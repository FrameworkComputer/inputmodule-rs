//! NOT THE OFFICIAL Keyboard firmware
//! Just experimental reference code for building keyboard firmare in Rust
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

//use cortex_m::delay::Delay;
//use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

use rp2040_hal::Timer;
//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

/// List maximum current as 500mA in the USB descriptor
const MAX_CURRENT: usize = 500;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use bsp::entry;
use fl16_inputmodules::keyboard_hal as bsp;
//use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    usb,
    watchdog::Watchdog,
};

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

use fl16_inputmodules::serialnum::device_release;

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

    let rosc = rp2040_hal::rosc::RingOscillator::new(pac.ROSC);
    let _rosc = rosc.initialize();

    let mut _delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

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

    #[cfg(feature = "macropad")]
    let pid = 0x013;
    #[cfg(feature = "ansi")]
    let pid = 0x012;
    #[cfg(feature = "numpad")]
    let pid = 0x014;
    #[cfg(feature = "iso")]
    let pid = 0x018;
    #[cfg(feature = "jis")]
    let pid = 0x019;

    #[cfg(feature = "macropad")]
    let product = "Rust Macropad";
    #[cfg(feature = "ansi")]
    let product = "Rust ANSI Keyboard";
    #[cfg(feature = "numpad")]
    let product = "Rust Numpad";
    #[cfg(feature = "iso")]
    let product = "Rust ISO Keyboard";
    #[cfg(feature = "jis")]
    let product = "Rust JIS Keyboard";

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, pid))
        .manufacturer("Framework Computer Inc")
        .product(product)
        //.supports_remote_wakeup(true)
        .device_class(USB_CLASS_CDC)
        .max_power(MAX_CURRENT)
        .serial_number("testing")
        .device_release(device_release())
        .build();

    // Disable bootloader circuitry
    let mut boot_done = pins.boot_done.into_push_pull_output();
    boot_done.set_high().unwrap();

    // pins.gp26 // SDA
    // pins.gp27 // SCL
    // pins.analog_in

    let mut caps_led = pins.caps_led.into_push_pull_output();
    let mut _backlight = pins.backlight.into_push_pull_output();

    // Pull low to enable mux
    let mut mux_enable = pins.mux_enable.into_push_pull_output();
    mux_enable.set_low().unwrap();
    let mut _mux_a = pins.mux_a.into_push_pull_output();
    let mut _mux_b = pins.mux_b.into_push_pull_output();
    let mut _mux_c = pins.mux_c.into_push_pull_output();

    // KS0 - KSO7 for Keyboard and Numpad
    let mut _kso0 = pins.kso0.into_push_pull_output();
    let mut _kso1 = pins.kso1.into_push_pull_output();
    let mut _kso2 = pins.kso2.into_push_pull_output();
    let mut _kso3 = pins.kso3.into_push_pull_output();
    let mut _kso4 = pins.kso4.into_push_pull_output();
    let mut _kso5 = pins.kso5.into_push_pull_output();
    let mut _kso6 = pins.kso6.into_push_pull_output();
    let mut _kso7 = pins.kso7.into_push_pull_output();
    // KS08 - KS015 for Keyboard only
    let mut _kso8 = pins.kso8.into_push_pull_output();
    let mut _kso9 = pins.kso9.into_push_pull_output();
    let mut _kso10 = pins.kso10.into_push_pull_output();
    let mut _kso11 = pins.kso11.into_push_pull_output();
    let mut _kso12 = pins.kso12.into_push_pull_output();
    let mut _kso13 = pins.kso13.into_push_pull_output();
    let mut _kso14 = pins.kso14.into_push_pull_output();
    let mut _kso15 = pins.kso15.into_push_pull_output();
    // Set unused pins to input to avoid interfering. They're hooked up to rows 5 and 6
    let _ = pins.ksi5_reserved.into_floating_input();
    let _ = pins.ksi6_reserved.into_floating_input();

    let sleep = pins.sleep.into_floating_input();

    // Enable LED controller
    // SDB
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_low().unwrap();
    //    led_enable.set_high().unwrap();
    //
    //    let i2c = bsp::hal::I2C::i2c1(
    //        pac.I2C1,
    //        pins.gpio26.into_mode::<gpio::FunctionI2C>(),
    //        pins.gpio27.into_mode::<gpio::FunctionI2C>(),
    //        1000.kHz(),
    //        &mut pac.RESETS,
    //        &clocks.peripheral_clock,
    //    );
    //
    //    let mut matrix = LedMatrix::new(i2c, DVT2_CALC_PIXEL);
    //    matrix
    //        .setup(&mut delay)
    //        .expect("failed to setup RGB controller");
    //
    //    matrix
    //        .set_scaling(MAX_BRIGHTNESS)
    //        .expect("failed to set scaling");
    //
    //    fill_grid_pixels(&state, &mut matrix);

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut animation_timer = timer.get_counter().ticks();
    caps_led.set_high().unwrap();

    let mut usb_initialized;
    let mut usb_suspended = false;
    loop {
        let _ = sleep.is_high();

        // Turn back on when not suspended anymore
        if !usb_suspended {
            if caps_led.is_set_low().unwrap() {
                caps_led.set_high().unwrap();
            }
        }

        // Blink when in USB Suspend
        // 500_000us = 500ms = 0.5s
        if usb_suspended && timer.get_counter().ticks() > animation_timer + 500_000 {
            if caps_led.is_set_high().unwrap() {
                caps_led.set_low().unwrap();
            } else {
                caps_led.set_high().unwrap();
            }
            animation_timer = timer.get_counter().ticks();
        }

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            match usb_dev.state() {
                // Default: Device has just been created or reset
                // Addressed: Device has received an address for the host
                UsbDeviceState::Default | UsbDeviceState::Addressed => {
                    usb_initialized = false;
                    usb_suspended = false;
                    // Must not display anything or windows cannot enumerate properly
                }
                // Configured and is fully operational
                UsbDeviceState::Configured => {
                    usb_initialized = true;
                    usb_suspended = false;
                }
                // Never occurs here. Only if poll() returns false
                UsbDeviceState::Suspend => {
                    panic!("Never occurs here. Only if poll() returns false")
                }
            }

            // Ignore unused
            let _ = usb_initialized;
            let _ = usb_suspended;

            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(count) => {
                    let mut text: String<64> = String::new();
                    write!(
                        &mut text,
                        "Hello World: Usb Suspended: {}. C: {}\r\n",
                        usb_suspended, count
                    )
                    .unwrap();
                    let _ = serial.write(text.as_bytes());
                }
            }
        } else {
            match usb_dev.state() {
                // No new data
                UsbDeviceState::Default | UsbDeviceState::Addressed => {
                    usb_initialized = false;
                    usb_suspended = false;
                }
                UsbDeviceState::Configured => {
                    usb_initialized = true;
                    usb_suspended = false;
                }
                UsbDeviceState::Suspend => {
                    usb_suspended = true;
                }
            }
            // Ignore unused
            let _ = usb_initialized;
            let _ = usb_suspended;
        }
    }
}
