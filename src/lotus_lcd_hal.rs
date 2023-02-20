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

// Current mapping is prototyping with ST7735 on Raspberry Pi
// Lotus LCD Module will have different pin mapping
// |  FN  | Pico | Lotus |
// |  SCL | GP14 | GP18  |
// |  SDA | GP15 | GP16  |
// |   RX | GP12 | GP19  |
// | DC/A0| GP13 | GP20  |
// |   BL | GP18 | GP??  |
// | RSTB | GP16 | GP21  |
hal::bsp_pins!(
    /// GPIO 0 is connected to the SLEEP# pin of the EC
    Gpio0 { name: sleep },
    Gpio14 {
        name: scl,
        aliases: {
            /// SPI Function alias for pin [crate::Pins::gpio14].
            FunctionSpi: Gp14Spi1Sck
        }
    },
    Gpio15 {
        name: sda,
        aliases: {
            /// SPI Function alias for pin [crate::Pins::gpio15].
            FunctionSpi: Gp15Spi1Tx
        }
    },
    Gpio12 {
        name: miso,
        aliases: {
            /// SPI Function alias for pin [crate::Pins::gpio12].
            FunctionSpi: Gp12Spi1Rx
        }
    },
    Gpio13 { name: dc },
    Gpio18 { name: backlight },
    Gpio16 { name: rstb },
);

// External crystal frequency, same as Raspberry Pi Pico
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
