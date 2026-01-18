// Adafruit QT Py RP2040 BSP
// Based on https://learn.adafruit.com/adafruit-qt-py-2040/pinouts
// Until https://github.com/rp-rs/rp-hal-boards/pull/99 is merged

pub extern crate rp2040_hal as hal;

extern crate cortex_m_rt;
pub use hal::entry;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_GD25Q64CS;

pub use hal::pac;

hal::bsp_pins!(
    /// GPIO 0 - UART0 TX / I2C0 SDA / SPI0 RX
    Gpio0 {
        name: tx,
        aliases: {
            FunctionUart, PullNone: Gp0Uart0Tx,
            FunctionI2C, PullUp: Gp0I2C0Sda,
            FunctionSpi, PullNone: Gp0Spi0Rx,
            FunctionPio0, PullNone: Gp0Pio0,
            FunctionPio1, PullNone: Gp0Pio1
        }
    },

    /// GPIO 1 - UART0 RX / I2C0 SCL / SPI0 CSn
    Gpio1 {
        name: rx,
        aliases: {
            FunctionUart, PullNone: Gp1Uart0Rx,
            FunctionI2C, PullUp: Gp1I2C0Scl,
            FunctionSpi, PullNone: Gp1Spi0Csn,
            FunctionPio0, PullNone: Gp1Pio0,
            FunctionPio1, PullNone: Gp1Pio1
        }
    },

    /// GPIO 2 - SPI0 SCK / I2C1 SDA
    Gpio2 {
        name: sck0,
        aliases: {
            FunctionSpi, PullNone: Gp2Spi0Sck,
            FunctionI2C, PullUp: Gp2I2C1Sda,
            FunctionPio0, PullNone: Gp2Pio0,
            FunctionPio1, PullNone: Gp2Pio1
        }
    },

    /// GPIO 3 - SPI0 TX (MOSI) / I2C1 SCL
    Gpio3 {
        name: mosi0,
        aliases: {
            FunctionSpi, PullNone: Gp3Spi0Tx,
            FunctionI2C, PullUp: Gp3I2C1Scl,
            FunctionPio0, PullNone: Gp3Pio0,
            FunctionPio1, PullNone: Gp3Pio1
        }
    },

    /// GPIO 4 - SPI0 RX (MISO)
    Gpio4 {
        name: miso0,
        aliases: {
            FunctionSpi, PullNone: Gp4Spi0Rx,
            FunctionPio0, PullNone: Gp4Pio0,
            FunctionPio1, PullNone: Gp4Pio1
        }
    },

    /// GPIO 5 - A0
    Gpio5 {
        name: a0,
    },

    /// GPIO 6 - A1
    Gpio6 {
        name: a1,
    },

    /// GPIO 7 - A2
    Gpio7 {
        name: a2,
    },

    /// GPIO 8 - A3
    Gpio8 {
        name: a3,
    },

    /// GPIO 11 - NeoPixel Power (directly active high enable)
    Gpio11 {
        name: neopixel_power,
    },

    /// GPIO 12 - NeoPixel Data
    Gpio12 {
        name: neopixel_data,
        aliases: {
            FunctionPio0, PullNone: Gp12Pio0,
            FunctionPio1, PullNone: Gp12Pio1
        }
    },

    /// GPIO 20 - I2C0 SDA (STEMMA QT)
    Gpio20 {
        name: sda,
        aliases: {
            FunctionI2C, PullUp: Gp20I2C0Sda,
            FunctionPio0, PullNone: Gp20Pio0,
            FunctionPio1, PullNone: Gp20Pio1
        }
    },

    /// GPIO 21 - I2C0 SCL (STEMMA QT)
    Gpio21 {
        name: scl,
        aliases: {
            FunctionI2C, PullUp: Gp21I2C0Scl,
            FunctionPio0, PullNone: Gp21Pio0,
            FunctionPio1, PullNone: Gp21Pio1
        }
    },

    /// GPIO 24 - SPI1 RX (MISO)
    Gpio24 {
        name: miso1,
        aliases: {
            FunctionSpi, PullNone: Gp24Spi1Rx,
            FunctionPio0, PullNone: Gp24Pio0,
            FunctionPio1, PullNone: Gp24Pio1
        }
    },

    /// GPIO 25 - SPI1 CSn
    Gpio25 {
        name: cs1,
        aliases: {
            FunctionSpi, PullNone: Gp25Spi1Csn,
            FunctionPio0, PullNone: Gp25Pio0,
            FunctionPio1, PullNone: Gp25Pio1
        }
    },

    /// GPIO 26 - SPI1 SCK
    Gpio26 {
        name: sck1,
        aliases: {
            FunctionSpi, PullNone: Gp26Spi1Sck,
            FunctionPio0, PullNone: Gp26Pio0,
            FunctionPio1, PullNone: Gp26Pio1
        }
    },

    /// GPIO 27 - SPI1 TX (MOSI)
    Gpio27 {
        name: mosi1,
        aliases: {
            FunctionSpi, PullNone: Gp27Spi1Tx,
            FunctionPio0, PullNone: Gp27Pio0,
            FunctionPio1, PullNone: Gp27Pio1
        }
    },

    /// GPIO 29 - A3 (also used internally)
    Gpio29 {
        name: a3_alt,
    },
);

// QT Py RP2040 uses a 12 MHz crystal
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
