//! NOT THE OFFICIAL Keyboard firmware
//! Just experimental reference code for building keyboard firmare in Rust
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

// TODO:
// - [x] Basic keyscan
// - [x] Basic HID keymaps for all five variants
// - [x] Can go into D2 (tested on Linux)
// - [x] Can wake host (remote wakeup)
// - [ ] Both serial and HID keyboard as composite device
// - [x] Key Debouncing
// - [ ] 1-Zone PWM backlight
// - [x] RGB backlight (needs new/modified Rust driver)
//   - [x] Working (all white)
//   - [ ] Map all LEDs
// - [x] Separate builds for different keyboard variants
// - [ ] Measure and optimize scan frequency
// - [x] Implement full basic key matrix
// - [x] Implement second layer for FN (including volatile FN lock)
// - [ ] Persist brightness setting and FN lock through reset
// - [ ] Media keys

//use defmt::*;
use crate::rgb_matrix::LedMatrix;
use defmt_rtt as _;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_02::adc::OneShot;
use rp2040_hal::gpio::bank0::Gpio28;
use rp2040_hal::gpio::{self, FunctionSioInput, PullUp};
use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::prelude::UsbHidClassBuilder;
use usbd_human_interface_device::UsbHidError;

mod keymap;
mod rgb_matrix;
use keymap::{COLS as MATRIX_COLS, ROWS as MATRIX_ROWS};

const SCAN_COLS: usize = if cfg!(any(feature = "numpad", feature = "macropad")) {
    8
} else {
    16
};
const SCAN_ROWS: usize = if cfg!(any(feature = "numpad", feature = "macropad")) {
    4
} else {
    8
};

#[cfg(feature = "ansi")]
use keymap::layouts::ANSI as LAYOUT;
#[cfg(feature = "iso")]
use keymap::layouts::ISO as LAYOUT;
#[cfg(feature = "jis")]
use keymap::layouts::JIS as LAYOUT;
#[cfg(feature = "macropad")]
use keymap::layouts::MACROPAD as LAYOUT;
#[cfg(feature = "numpad")]
use keymap::layouts::NUMPAD as LAYOUT;

use rp2040_hal::{
    gpio::{
        bank0::{
            Gpio1, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16, Gpio17, Gpio18, Gpio19,
            Gpio2, Gpio20, Gpio21, Gpio22, Gpio23, Gpio3, Gpio8, Gpio9,
        },
        FunctionSioOutput, Pin, PinState, PullDown,
    },
    Adc, Timer,
};
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

use bsp::hal::{clocks::init_clocks_and_plls, pac, sio::Sio, usb, watchdog::Watchdog};
use fugit::RateExtU32;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

use fl16_inputmodules::serialnum::device_release;

struct Mux {
    a: Pin<Gpio1, FunctionSioOutput, PullDown>,
    b: Pin<Gpio2, FunctionSioOutput, PullDown>,
    c: Pin<Gpio3, FunctionSioOutput, PullDown>,
    // TODO
    // x: Pin<Gpio3, FunctionSioOutput, PullDown>,
}
impl Mux {
    pub fn select_row(&mut self, row: usize) {
        let index = match row {
            0 => 2,
            1 => 0,
            2 => 1,
            _ => row,
        };
        self.a.set_state(PinState::from(index & 0x01 != 0)).unwrap();
        self.b.set_state(PinState::from(index & 0x02 != 0)).unwrap();
        self.c.set_state(PinState::from(index & 0x04 != 0)).unwrap();
    }
}

type Kso = (
    Pin<Gpio8, FunctionSioOutput, PullDown>,
    Pin<Gpio9, FunctionSioOutput, PullDown>,
    Pin<Gpio10, FunctionSioOutput, PullDown>,
    Pin<Gpio11, FunctionSioOutput, PullDown>,
    Pin<Gpio12, FunctionSioOutput, PullDown>,
    Pin<Gpio13, FunctionSioOutput, PullDown>,
    Pin<Gpio14, FunctionSioOutput, PullDown>,
    Pin<Gpio15, FunctionSioOutput, PullDown>,
    Pin<Gpio21, FunctionSioOutput, PullDown>,
    Pin<Gpio20, FunctionSioOutput, PullDown>,
    Pin<Gpio19, FunctionSioOutput, PullDown>,
    Pin<Gpio18, FunctionSioOutput, PullDown>,
    Pin<Gpio17, FunctionSioOutput, PullDown>,
    Pin<Gpio16, FunctionSioOutput, PullDown>,
    Pin<Gpio23, FunctionSioOutput, PullDown>,
    Pin<Gpio22, FunctionSioOutput, PullDown>,
);

struct Scanner {
    kso: Kso,
    mux: Mux,
    adc: (
        Adc,
        rp2040_hal::adc::AdcPin<Pin<Gpio28, FunctionSioInput, PullUp>>,
    ),
}
impl Scanner {
    fn drive_col(&mut self, col: usize, state: PinState) {
        match col {
            0 => self.kso.0.set_state(state).unwrap(),
            1 => self.kso.1.set_state(state).unwrap(),
            2 => self.kso.2.set_state(state).unwrap(),
            3 => self.kso.3.set_state(state).unwrap(),
            4 => self.kso.4.set_state(state).unwrap(),
            5 => self.kso.5.set_state(state).unwrap(),
            6 => self.kso.6.set_state(state).unwrap(),
            7 => self.kso.7.set_state(state).unwrap(),
            8 => self.kso.8.set_state(state).unwrap(),
            9 => self.kso.9.set_state(state).unwrap(),
            10 => self.kso.10.set_state(state).unwrap(),
            11 => self.kso.11.set_state(state).unwrap(),
            12 => self.kso.12.set_state(state).unwrap(),
            13 => self.kso.13.set_state(state).unwrap(),
            14 => self.kso.14.set_state(state).unwrap(),
            15 => self.kso.15.set_state(state).unwrap(),
            _ => unreachable!(),
        }
    }
    /// Scan one column at a time so USB is serviced between columns.
    fn scan_column(&mut self, col: usize) -> [u16; MATRIX_ROWS] {
        self.drive_col(col, PinState::Low);
        let mut voltages = [3300; MATRIX_ROWS];
        for (row, voltage) in voltages.iter_mut().enumerate().take(SCAN_ROWS) {
            self.mux.select_row(row);
            // Retain the settling time used by the original single-key reader.
            cortex_m::asm::delay(2000);
            let sample: u16 = self.adc.0.read(&mut self.adc.1).unwrap();
            *voltage = ((sample as u32 * 3300) / 4096) as u16;
        }
        self.drive_col(col, PinState::High);
        voltages
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
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

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

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
    //let mut serial = SerialPort::new(&usb_bus);
    let mut keyboard_hid = UsbHidClassBuilder::new()
        .add_device(usbd_human_interface_device::device::keyboard::BootKeyboardConfig::default())
        .build(&usb_bus);

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
        .strings(&[usb_device::device::StringDescriptors::new(LangID::EN_US)
            .manufacturer("Framework Computer Inc")
            .product(product)
            .serial_number("testing")])
        .unwrap()
        .supports_remote_wakeup(true)
        .device_class(0)
        .device_sub_class(0)
        .device_protocol(0)
        .max_power(MAX_CURRENT)
        .unwrap()
        .device_release(device_release())
        .build();

    // Disable bootloader circuitry
    let mut boot_done = pins.boot_done.into_push_pull_output();
    boot_done.set_low().unwrap();

    // pins.gp26 // SDA
    // pins.gp27 // SCL

    let mut caps_led = pins.caps_led.into_push_pull_output();
    let mut _backlight = pins.backlight.into_push_pull_output();

    // Pull low to enable mux
    let mut mux_enable = pins.mux_enable.into_push_pull_output();
    mux_enable.set_low().unwrap();
    let mux_a = pins.mux_a.into_push_pull_output();
    let mux_b = pins.mux_b.into_push_pull_output();
    let mux_c = pins.mux_c.into_push_pull_output();

    // KS0 - KSO7 for Keyboard and Numpad
    let kso0 = pins.kso0.into_push_pull_output();
    let kso1 = pins.kso1.into_push_pull_output();
    let kso2 = pins.kso2.into_push_pull_output();
    let kso3 = pins.kso3.into_push_pull_output();
    let kso4 = pins.kso4.into_push_pull_output();
    let kso5 = pins.kso5.into_push_pull_output();
    let kso6 = pins.kso6.into_push_pull_output();
    let kso7 = pins.kso7.into_push_pull_output();
    // KS08 - KS015 for Keyboard only
    let kso8 = pins.kso8.into_push_pull_output();
    let kso9 = pins.kso9.into_push_pull_output();
    let kso10 = pins.kso10.into_push_pull_output();
    let kso11 = pins.kso11.into_push_pull_output();
    let kso12 = pins.kso12.into_push_pull_output();
    let kso13 = pins.kso13.into_push_pull_output();
    let kso14 = pins.kso14.into_push_pull_output();
    let kso15 = pins.kso15.into_push_pull_output();
    // Set unused pins to input to avoid interfering. They're hooked up to rows 5 and 6
    let _ = pins.ksi5_reserved.into_floating_input();
    let _ = pins.ksi6_reserved.into_floating_input();

    let mut sleep = pins.sleep.into_floating_input();

    // Enable LED controller
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_high().unwrap();

    let i2c = bsp::hal::I2C::i2c1(
        pac.I2C1,
        pins.gpio26
            .into_function::<gpio::FunctionI2C>()
            .into_pull_type::<PullUp>(),
        pins.gpio27
            .into_function::<gpio::FunctionI2C>()
            .into_pull_type::<PullUp>(),
        1000.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut matrix = LedMatrix::new(i2c, |_, _| 0x00);

    cfg_if::cfg_if! {
        if #[cfg(any(feature = "ansi", feature = "macropad"))] {
            matrix.set_address(0b0100000);
            matrix
                .setup(&mut timer)
                .expect("failed to setup RGB controller");
            matrix.set_scaling(0xFF).expect("failed to set scaling");
            matrix.device.fill(0xFF);
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "ansi")] {
            matrix.set_address(0b0100011);
            matrix
                .setup(&mut timer)
                .expect("failed to setup RGB controller");
            matrix.set_scaling(0xFF).expect("failed to set scaling");
            matrix.device.fill(0xFF);
        }
    }

    let mut tick_timer = timer.get_counter().ticks();
    let mut backlight_timer = timer.get_counter().ticks();
    let mut backlight_on = true;
    let mut caps_lock = false;
    caps_led.set_low().unwrap();

    let adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let adc_x = rp2040_hal::adc::AdcPin::new(pins.analog_in.into_pull_up_input()).unwrap();

    let mut scanner = Scanner {
        kso: (
            kso0, kso1, kso2, kso3, kso4, kso5, kso6, kso7, kso8, kso9, kso10, kso11, kso12, kso13,
            kso14, kso15,
        ),
        mux: Mux {
            a: mux_a,
            b: mux_b,
            c: mux_c,
        },
        adc: (adc, adc_x),
    };

    for col in 0..MATRIX_COLS {
        scanner.drive_col(col, PinState::High);
    }
    let mut keys = keymap::Keys::new();
    let mut voltages = [[3300; MATRIX_COLS]; MATRIX_ROWS];
    let mut scan_col = 0;
    let mut usb_initialized;
    let mut usb_suspended = false;
    loop {
        let _ = sleep.is_high();

        // Keep the RGB status animation independent of the Caps Lock indicator.
        if !usb_suspended && !backlight_on {
            #[cfg(any(feature = "ansi", feature = "macropad"))]
            let _ = matrix.device.fill(0xFF);
            backlight_on = true;
        }

        // Blink when in USB Suspend
        // 500_000us = 500ms = 0.5s
        if usb_suspended && timer.get_counter().ticks() > backlight_timer + 500_000 {
            if backlight_on {
                #[cfg(any(feature = "ansi", feature = "macropad"))]
                let _ = matrix.device.fill(0x00);
            } else {
                #[cfg(any(feature = "ansi", feature = "macropad"))]
                let _ = matrix.device.fill(0xFF);
            }
            backlight_on = !backlight_on;
            backlight_timer = timer.get_counter().ticks();
        }

        let readings = scanner.scan_column(scan_col);
        for row in 0..MATRIX_ROWS {
            voltages[row][scan_col] = readings[row];
        }
        scan_col += 1;
        if scan_col == SCAN_COLS {
            let pressed = if sleep.is_high().unwrap() {
                keymap::interpret(&voltages)
            } else {
                [[false; MATRIX_COLS]; MATRIX_ROWS]
            };
            keys.update(pressed, timer.get_counter().ticks(), &LAYOUT);
            scan_col = 0;
        }

        if !usb_suspended {
            if let Ok(leds) = keyboard_hid.device().read_report() {
                caps_lock = leds.caps_lock;
            }

            // Retry the full held state if the USB endpoint was busy.
            match keyboard_hid
                .device()
                .write_report(keys.pressed().map(Keyboard::from))
            {
                Err(UsbHidError::WouldBlock) | Err(UsbHidError::Duplicate) | Ok(_) => {}
                Err(e) => panic!("Failed to write keyboard report: {:?}", e),
            }
        }
        // The HID idle timer expects a 1 ms tick, independently of scan speed.
        let now = timer.get_counter().ticks();
        while now - tick_timer >= 1_000 {
            tick_timer += 1_000;
            if !usb_suspended {
                match keyboard_hid.tick() {
                    Err(UsbHidError::WouldBlock) | Ok(_) => {}
                    Err(e) => panic!("Failed to process keyboard tick: {:?}", e),
                }
            }
        }

        if keys.any_pressed()
            && sleep.is_high().unwrap()
            && usb_suspended
            && usb_dev.remote_wakeup_enabled()
        {
            usb_dev.bus().remote_wakeup();
        }

        // Check for new data
        if usb_dev.poll(&mut [&mut keyboard_hid]) {
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

            keyboard_hid.poll();
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
        // The host owns lock state, including changes from another keyboard.
        // Clear stale state on USB reset, and blank the indicator in suspend.
        if matches!(
            usb_dev.state(),
            UsbDeviceState::Default | UsbDeviceState::Addressed
        ) {
            caps_lock = false;
        }
        caps_led
            .set_state(PinState::from(caps_lock && !usb_suspended))
            .unwrap();
    }
}
