//! C1 Minimal Input Module
//!
//! Neopixel/WS2812 compatible RGB LED is connected to GPIO16.
//! This pin doesn't support SPI TX.
//! It does support UART TX, but that output would have to be inverted.
//! So instead we use PIO to drive the LED.
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use bsp::entry;
use cortex_m::delay::Delay;
use defmt_rtt as _;
use embedded_hal::digital::InputPin;

use rp2040_hal::gpio::bank0::Gpio16;
use rp2040_hal::gpio::{FunctionPio0, Pin, PullDown};
use rp2040_hal::pio::PIOExt;
//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use fl16_inputmodules::minimal_hal as bsp;
//use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    usb,
    watchdog::Watchdog,
    Timer,
};

// USB Device support
use usb_device::descriptor::lang_id::LangID;
use usb_device::device::StringDescriptors;
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Used to demonstrate writing formatted strings
// use core::fmt::Write;
// use heapless::String;

// RGB LED
use smart_leds::{colors, SmartLedsWrite, RGB8};
pub type Ws2812 = ws2812_pio::Ws2812<
    crate::pac::PIO0,
    rp2040_hal::pio::SM0,
    rp2040_hal::timer::CountDown,
    Pin<Gpio16, FunctionPio0, PullDown>,
>;

use fl16_inputmodules::control::*;
use fl16_inputmodules::serialnum::{device_release, get_serialnum};

//                            FRA                - Framwork
//                               000             - C1 Minimal Input Module (No assigned  value)
//                                  AM           - Atemitech
//                                    00         - Default Configuration
//                                      00000000 - Device Identifier
const DEFAULT_SERIAL: &str = "FRA000AM0000000000";

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Create timer before USB bus since USB bus moves clocks.usb_clock
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    let serialnum = if let Some(serialnum) = get_serialnum() {
        serialnum.serialnum
    } else {
        DEFAULT_SERIAL
    };

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, 0x0022))
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer("Framework Computer Inc")
            .product("C1 Minimal Input Module")
            .serial_number(serialnum)])
        .unwrap()
        .max_power(500) // TODO: Check how much
        .unwrap()
        .device_release(device_release())
        .device_class(USB_CLASS_CDC)
        .build();

    let mut sleep = pins.sleep.into_pull_down_input();

    let mut state = C1MinimalState {
        sleeping: SimpleSleepState::Awake,
        color: colors::GREEN,
        brightness: 10,
    };

    let mut prev_timer = timer.get_counter().ticks();

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws2812: Ws2812 = ws2812_pio::Ws2812::new(
        pins.rgb_led.into_function(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    ws2812
        .write(smart_leds::brightness(
            [state.color].iter().cloned(),
            state.brightness,
        ))
        .unwrap();

    loop {
        // Go to sleep if the host is sleeping
        let host_sleeping = sleep.is_low().unwrap();
        handle_sleep(host_sleeping, &mut state, &mut delay, &mut ws2812);

        // Handle period LED updates. Don't do it too often or USB will get stuck
        if timer.get_counter().ticks() > prev_timer + 20_000 {
            // TODO: Can do animations here
            prev_timer = timer.get_counter().ticks();
        }

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    if let Some(command) = parse_command(count, &buf) {
                        if let Command::Sleep(go_sleeping) = command {
                            handle_sleep(go_sleeping, &mut state, &mut delay, &mut ws2812);
                        } else if let SimpleSleepState::Awake = state.sleeping {
                            // While sleeping no command is handled, except waking up
                            if let Some(response) =
                                handle_command(&command, &mut state, &mut ws2812)
                            {
                                let _ = serial.write(&response);
                            };
                        }
                    }
                }
            }
        }
    }
}

fn handle_sleep(
    go_sleeping: bool,
    state: &mut C1MinimalState,
    _delay: &mut Delay,
    ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ()>,
) {
    match (state.sleeping.clone(), go_sleeping) {
        (SimpleSleepState::Awake, false) => (),
        (SimpleSleepState::Awake, true) => {
            state.sleeping = SimpleSleepState::Sleeping;

            // Turn off LED
            ws2812.write([colors::BLACK].iter().cloned()).unwrap();

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        (SimpleSleepState::Sleeping, true) => (),
        (SimpleSleepState::Sleeping, false) => {
            state.sleeping = SimpleSleepState::Awake;

            // Turn LED back on
            ws2812
                .write(smart_leds::brightness(
                    [state.color].iter().cloned(),
                    state.brightness,
                ))
                .unwrap();
        }
    }
}
