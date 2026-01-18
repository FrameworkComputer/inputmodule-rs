//! QT PY RP2040 with Framework 16 Input Module Firmware
//!
//! Neopixel/WS2812 compatible RGB LED is connected to GPIO12.
//! This pin doesn't support SPI TX.
//! It does support UART TX, but that output would have to be inverted.
//! So instead we use PIO to drive the LED.
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use bsp::entry;
use cortex_m::delay::Delay;
use defmt_rtt as _;

use rp2040_hal::gpio::bank0::Gpio12;
use rp2040_hal::gpio::{FunctionPio0, Pin, PullDown};
use rp2040_hal::pio::PIOExt;
//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

// Use local BSP from fl16-inputmodules
use fl16_inputmodules::qtpy_hal as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::PinState,
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
    Pin<Gpio12, FunctionPio0, PullDown>,
>;

use fl16_inputmodules::control::*;
use fl16_inputmodules::serialnum::device_release;

const FRAMEWORK_VID: u16 = 0x32AC;
const COMMUNITY_PID: u16 = 0x001F;

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

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(FRAMEWORK_VID, COMMUNITY_PID))
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer("Adafruit")
            .product("QT PY - Framework 16 Inputmodule FW")])
        .unwrap()
        .max_power(500)
        .unwrap()
        .device_release(device_release())
        .device_class(USB_CLASS_CDC)
        .build();

    let mut state = C1MinimalState {
        sleeping: SimpleSleepState::Awake,
        color: colors::GREEN,
        brightness: 10,
    };

    let mut prev_timer = timer.get_counter().ticks();

    pins.neopixel_power
        .into_push_pull_output_in_state(PinState::High);
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws2812: Ws2812 = ws2812_pio::Ws2812::new(
        pins.neopixel_data.into_function(),
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
