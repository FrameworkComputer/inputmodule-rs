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

// Previously prototyping with ST7735 on Raspberry Pi
// LCD Module Input Module has a different pin mapping
// |  FN  | Pico | LCD Input Module |
// |  SCL | GP14 |            GP18  |
// |  SDA | GP15 |            GP19  |
// |   RX | GP12 |            GP16  |
// | DC/A0| GP13 |            GP20  |
// |   BL | GP18 |            GP??  |
// | RSTB | GP16 |            GP21  |
hal::bsp_pins!(
    /// GPIO 0 is connected to the SLEEP# pin of the EC
    Gpio0 { name: sleep },
    Gpio18 {
        name: scl,
        aliases: {
            /// SPI Function alias for pin [crate::Pins::gpio14].
            FunctionSpi, PullNone: Gp18Spi1Sck
        }
    },
    Gpio19 {
        name: sda,
        aliases: {
            /// SPI Function alias for pin [crate::Pins::gpio15].
            FunctionSpi, PullNone: Gp19Spi1Tx
        }
    },
    Gpio16 {
        name: miso,
        aliases: {
            /// SPI Function alias for pin [crate::Pins::gpio12].
            FunctionSpi, PullNone: Gp16Spi1Rx
        }
    },
    Gpio20 { name: dc },
    //Gpio18 { name: backlight },
    Gpio21 { name: rstb },
    Gpio17 { name: cs },
);

// External crystal frequency, same as Raspberry Pi Pico
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
