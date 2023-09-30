// Taken from rp_pico hal and adjusted

pub extern crate rp2040_hal as hal;

extern crate cortex_m_rt;
pub use hal::entry;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

pub use hal::pac;

hal::bsp_pins!(
    /// GPIO 0 is connected to the SLEEP# pin of the EC
    Gpio0 { name: sleep },
    /// Mux selector A
    Gpio1 { name: mux_a },
    /// Mux selector B
    Gpio2 { name: mux_b },
    /// Mux selector C
    Gpio3 { name: mux_c },
    /// Mux enable
    Gpio4 { name: mux_enable },
    /// Pull low when firmware has started to turn off bootloader logic
    Gpio5 { name: boot_done },
    /// Connected to KSI5 but unused, should use high-Z
    Gpio6 {
        name: ksi5_reserved
    },
    /// Connected to KSI5 but unused, should use high-Z
    Gpio7 {
        name: ksi6_reserved
    },
    /// Keyboard column drive
    Gpio8 { name: kso0 },
    /// Keyboard column drive
    Gpio9 { name: kso1 },
    /// Keyboard column drive
    Gpio10 { name: kso2 },
    /// Keyboard column drive
    Gpio11 { name: kso3 },
    /// Keyboard column drive
    Gpio12 { name: kso4 },
    /// Keyboard column drive
    Gpio13 { name: kso5 },
    /// Keyboard column drive
    Gpio14 { name: kso6 },
    /// Keyboard column drive
    Gpio15 { name: kso7 },
    /// Keyboard column drive
    Gpio16 { name: kso13 },
    /// Keyboard column drive
    Gpio17 { name: kso12 },
    /// Keyboard column drive
    Gpio18 { name: kso11 },
    /// Keyboard column drive
    Gpio19 { name: kso10 },
    /// Keyboard column drive
    Gpio20 { name: kso9 },
    /// Keyboard column drive
    Gpio21 { name: kso8 },
    /// Keyboard column drive
    Gpio22 { name: kso15 },
    /// Keyboard column drive
    Gpio23 { name: kso14 },
    /// Capslock LED
    Gpio24 { name: caps_led },
    /// Single zone backlight (unused on RGB keyboard)
    Gpio25 { name: backlight },
    /// GPIO 26 is connected to I2C SDA of the LED controller
    Gpio26 {
        name: gpio26,
        aliases: {
            /// I2C Function alias for pin [crate::Pins::gpio26].
            FunctionI2C: Gp26I2C1Sda
        }
    },
    /// GPIO 27 is connected to I2C SCL of the LED controller
    Gpio27 {
        name: gpio27,
        aliases: {
            /// I2C Function alias for pin [crate::Pins::gpio27].
            FunctionI2C: Gp27I2C1Scl
        }
    },
    /// Analog IN from mux
    Gpio28 { name: analog_in },
    /// GPIO 29 is connected to the SDB pin of the LED controller
    Gpio29 { name: sdb },
);

// External crystal frequency, same as Raspberry Pi Pico
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
