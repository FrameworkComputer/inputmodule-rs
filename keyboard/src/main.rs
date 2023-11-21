//! NOT THE OFFICIAL Keyboard firmware
//! Just experimental reference code for building keyboard firmare in Rust
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

// TODO:
// - [x] Basic keyscan
// - [x] Send UP/LEFT/RIGHT/CAPS HID reports, DOWN to go into bootloader
// - [x] Can go into D2 (tested on Linux)
// - [x] Can wake host (remote wakeup)
// - [ ] Both serial and HID keyboard as composite device
// - [ ] Key Debouncing
// - [ ] 1-Zone PWM backlight
// - [x] RGB backlight (needs new/modified Rust driver)
//   - [x] Working (all white)
//   - [ ] Map all LEDs
// - [ ] Separate builds for different keyboard variants
// - [ ] Measure and optimize scan frequency
// - [ ] Implement full key matrix with all keys
// - [ ] Implement second layer for FN (including FN lock)
// - [ ] Persist brightness setting and FN lock through reset
// - [ ] Media keys

use cortex_m::delay::Delay;
//use defmt::*;
use crate::rgb_matrix::{LedMatrix, DVT2_CALC_PIXEL};
use defmt_rtt as _;
use embedded_hal::adc::OneShot;
use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};
use rp2040_hal::gpio::bank0::Gpio28;
use rp2040_hal::gpio::{self, Input, PullUp};
use usbd_human_interface_device::device::keyboard::BootKeyboardInterface;
use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::prelude::UsbHidClassBuilder;
use usbd_human_interface_device::UsbHidError;

mod rgb_matrix;

use core::fmt::Display;
use core::fmt::{self, Formatter};

use rp2040_hal::{
    gpio::{
        bank0::{
            Gpio1, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16, Gpio17, Gpio18, Gpio19,
            Gpio2, Gpio20, Gpio21, Gpio22, Gpio23, Gpio3, Gpio8, Gpio9,
        },
        Output, Pin, PinState, PushPull,
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

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    usb,
    watchdog::Watchdog,
};
use fugit::RateExtU32;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

use fl16_inputmodules::serialnum::device_release;

const MATRIX_COLS: usize = 16;
const MATRIX_ROWS: usize = 8;
const ADC_THRESHOLD: usize = 2900;

struct Mux {
    a: Pin<Gpio1, Output<PushPull>>,
    b: Pin<Gpio2, Output<PushPull>>,
    c: Pin<Gpio3, Output<PushPull>>,
    // TODO
    // x: Pin<Gpio3, Output<PushPull>>,
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Col(u8);
#[derive(Debug, PartialEq, Eq, Clone, Default)]
struct Matrix([Col; MATRIX_COLS]);

impl Matrix {
    pub fn set(&mut self, row: usize, col: usize, val: bool) {
        let mask = 1 << row;

        self.0[col].0 = if val {
            self.0[col].0 | mask
        } else {
            self.0[col].0 & !mask
        };
    }
}

impl Display for Col {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for row in 0..MATRIX_ROWS {
            let val = (self.0 & (1 << row)) >> row;
            write!(f, "{:b}", val)?
        }
        Ok(())
    }
}

type Kso = (
    Pin<Gpio8, Output<PushPull>>,
    Pin<Gpio9, Output<PushPull>>,
    Pin<Gpio10, Output<PushPull>>,
    Pin<Gpio11, Output<PushPull>>,
    Pin<Gpio12, Output<PushPull>>,
    Pin<Gpio13, Output<PushPull>>,
    Pin<Gpio14, Output<PushPull>>,
    Pin<Gpio15, Output<PushPull>>,
    Pin<Gpio21, Output<PushPull>>,
    Pin<Gpio20, Output<PushPull>>,
    Pin<Gpio19, Output<PushPull>>,
    Pin<Gpio18, Output<PushPull>>,
    Pin<Gpio17, Output<PushPull>>,
    Pin<Gpio16, Output<PushPull>>,
    Pin<Gpio23, Output<PushPull>>,
    Pin<Gpio22, Output<PushPull>>,
);

struct Scanner {
    kso: Kso,
    mux: Mux,
    adc: (Adc, Pin<Gpio28, Input<PullUp>>),
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
    fn read_voltage(&mut self) -> usize {
        let _adc_read: u16 = self.adc.0.read(&mut self.adc.1).unwrap();
        33000
    }
    pub fn measure_key(&mut self, row: usize, col: usize) -> (usize, usize) {
        for col in 0..MATRIX_COLS {
            self.drive_col(col, PinState::High);
        }
        self.drive_col(col, PinState::Low);

        self.mux.select_row(row);
        // Let column and mux settle a bit
        cortex_m::asm::delay(2000);
        let adc_read: u16 = self.adc.0.read(&mut self.adc.1).unwrap();

        self.drive_col(col, PinState::High);

        let voltage_10k = ((adc_read as usize) * 3300) / 4096;
        (voltage_10k / 1_000, voltage_10k % 1_000)
    }
    pub fn scan(&mut self) -> Matrix {
        let mut matrix = Matrix::default();

        // Initialize all cols as high
        for col in 0..MATRIX_COLS {
            self.drive_col(col, PinState::High);
        }

        for col in 0..MATRIX_COLS {
            self.drive_col(col, PinState::Low);

            for row in 0..MATRIX_ROWS {
                self.mux.select_row(row);

                if self.read_voltage() < ADC_THRESHOLD {
                    matrix.set(row, col, true);
                }
            }

            self.drive_col(col, PinState::High);
        }

        matrix.set(3, 4, true);
        matrix.set(0, 4, true);
        matrix
    }
}

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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

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
        .add_interface(BootKeyboardInterface::default_config())
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
        .manufacturer("Framework Computer Inc")
        .product(product)
        .supports_remote_wakeup(true)
        .device_class(0)
        .device_sub_class(0)
        .device_protocol(0)
        .max_power(MAX_CURRENT)
        .serial_number("testing")
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

    let sleep = pins.sleep.into_floating_input();

    // Enable LED controller
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_high().unwrap();

    let i2c = bsp::hal::I2C::i2c1(
        pac.I2C1,
        pins.gpio26.into_mode::<gpio::FunctionI2C>(),
        pins.gpio27.into_mode::<gpio::FunctionI2C>(),
        1000.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut matrix = LedMatrix::new(i2c, |_, _| 0x00);

    cfg_if::cfg_if! {
        if #[cfg(any(feature = "ansi", feature = "macropad"))] {
            matrix.set_address(0b0100000);
            matrix
                .setup(&mut delay)
                .expect("failed to setup RGB controller");
            matrix.set_scaling(0xFF).expect("failed to set scaling");
            matrix.device.fill(0xFF);
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "ansi")] {
            matrix.set_address(0b0100011);
            matrix
                .setup(&mut delay)
                .expect("failed to setup RGB controller");
            matrix.set_scaling(0xFF).expect("failed to set scaling");
            matrix.device.fill(0xFF);
        }
    }

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut scan_timer = timer.get_counter().ticks();
    let mut capslock_timer = timer.get_counter().ticks();
    caps_led.set_high().unwrap();

    let adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let adc_x = pins.analog_in.into_pull_up_input();

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

    let mut usb_initialized;
    let mut usb_suspended = false;
    loop {
        let _ = sleep.is_high();

        // Turn back on when not suspended anymore
        if !usb_suspended {
            if caps_led.is_set_low().unwrap() {
                caps_led.set_high().unwrap();
                #[cfg(any(feature = "ansi", feature = "macropad"))]
                matrix.device.fill(0xFF);
            }
        }

        // Blink when in USB Suspend
        // 500_000us = 500ms = 0.5s
        if usb_suspended && timer.get_counter().ticks() > capslock_timer + 500_000 {
            if caps_led.is_set_high().unwrap() {
                #[cfg(any(feature = "ansi", feature = "macropad"))]
                matrix.device.fill(0x00);
                caps_led.set_low().unwrap();
            } else {
                caps_led.set_high().unwrap();
                #[cfg(any(feature = "ansi", feature = "macropad"))]
                matrix.device.fill(0xFF);
            }
            capslock_timer = timer.get_counter().ticks();
        }

        let mut keycode: Option<Keyboard> = None;
        if timer.get_counter().ticks() > scan_timer + 250_000 {
            cfg_if::cfg_if! {
                if #[cfg(any(feature = "ansi", feature = "iso", feature = "jis"))] {
                    let left = scanner.measure_key(6, 11);
                    let up = scanner.measure_key(1, 13);
                    let down = scanner.measure_key(1, 8);
                    let right = scanner.measure_key(2, 15);
                    let caps = scanner.measure_key(4, 4);

                    let left_p = left.0 < 2 || (left.0 == 2 && left.1 < 290);
                    let right_p = right.0 < 2 || (right.0 == 2 && right.1 < 290);
                    let up_p = up.0 < 2 || (up.0 == 2 && up.1 < 290);
                    let down_p = down.0 < 2 || (down.0 == 2 && down.1 < 290);
                    let caps_p = caps.0 < 2 || (caps.0 == 2 && caps.1 < 290);

                    if left_p {
                        keycode = Some(Keyboard::LeftArrow);
                    } else if right_p {
                        keycode = Some(Keyboard::RightArrow);
                    } else if up_p {
                        keycode = Some(Keyboard::UpArrow);
                    } else if down_p {
                        keycode = None;
                        rp2040_hal::rom_data::reset_to_usb_boot(0, 0);
                    } else if caps_p {
                        keycode = Some(Keyboard::CapsLock);
                    } else {
                        keycode = None;
                    }
                }
            }

            cfg_if::cfg_if! {
                if #[cfg(any(feature = "numpad", feature = "macropad"))] {
                    let one = scanner.measure_key(0, 3);
                    let two = scanner.measure_key(0, 7);
                    let three = scanner.measure_key(1, 4);
                    let four = scanner.measure_key(2, 6);

                    let one_p = one.0 < 2 || (one.0 == 2 && one.1 < 290);
                    let two_p = two.0 < 2 || (two.0 == 2 && two.1 < 290);
                    let three_p = three.0 < 2 || (three.0 == 2 && three.1 < 290);
                    let four_p = four.0 < 2 || (four.0 == 2 && four.1 < 290);

                    if one_p {
                        keycode = Some(Keyboard::Keyboard1);
                    } else if two_p {
                        keycode = Some(Keyboard::Keyboard2);
                    } else if three_p {
                        keycode = Some(Keyboard::Keyboard3);
                    } else if four_p {
                        keycode = Some(Keyboard::Keyboard4);
                    } else {
                        keycode = None;
                    }
                }
            }

            scan_timer = timer.get_counter().ticks();
        }

        if !usb_suspended {
            let _ = keyboard_hid.interface().read_report();

            // Setup the report for the control channel
            let keycodes = if let Some(keycode) = keycode {
                [keycode]
            } else {
                [Keyboard::NoEventIndicated]
            };
            match keyboard_hid.interface().write_report(keycodes) {
                Err(UsbHidError::WouldBlock) | Err(UsbHidError::Duplicate) | Ok(_) => {}
                Err(e) => panic!("Failed to write keyboard report: {:?}", e),
            }
            match keyboard_hid.interface().tick() {
                Err(UsbHidError::WouldBlock) | Ok(_) => {}
                Err(e) => panic!("Failed to process keyboard tick: {:?}", e),
            }
        }

        // Wake the host.
        if keycode.is_some() && usb_suspended && usb_dev.remote_wakeup_enabled() {
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

            //let kb = Matrix::default();
            //let kb = scanner.scan();

            keyboard_hid.poll();

            // let mut buf = [0u8; 64];
            // match serial.read(&mut buf) {
            //     Err(_e) => {
            //         // Do nothing
            //     }
            //     Ok(0) => {
            //         // Do nothing
            //     }
            //     Ok(_count) => {
            //         match buf[0] {
            //             b'r' => rp2040_hal::rom_data::reset_to_usb_boot(0, 0),
            //             _ => (),
            //         }
            //         //let mut text: String<64> = String::new();
            //         //write!(&mut text, "    01234567\r\n").unwrap();
            //         //let _ = serial.write(text.as_bytes());

            //         //for col in 0..MATRIX_COLS {
            //         //    let mut text: String<64> = String::new();
            //         //    write!(&mut text, "{:2}: {}\r\n", col, kb.0[col]).unwrap();
            //         //    let _ = serial.write(text.as_bytes());
            //         //}
            //     }
            // }
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
