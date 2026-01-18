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
    // Pins not connected:
    // - Gpio5
    // - Gpio14
    // - Gpio15
    // - Gpio17
    // - Gpio21
    // - Gpio22
    // - Gpio23

    /// GPIO 0 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 RX`    | [crate::Gp0Spi0Rx]          |
    /// | `UART0 TX`   | [crate::Gp0Uart0Tx]         |
    /// | `I2C0 SDA`   | [crate::Gp0I2C0Sda]         |
    /// | `PWM0 A`     | [crate::Gp0Pwm0A]           |
    /// | `PIO0`       | [crate::Gp0Pio0]            |
    /// | `PIO1`       | [crate::Gp0Pio1]            |
    Gpio0 {
        name: tx,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio0].
            FunctionUart, PullNone: Gp0Uart0Tx,
            /// SPI Function alias for pin [crate::Pins::gpio0].
            FunctionSpi, PullNone: Gp0Spi0Rx,
            /// I2C Function alias for pin [crate::Pins::gpio0].
            FunctionI2C, PullUp: Gp0I2C0Sda,
            /// PWM Function alias for pin [crate::Pins::gpio0].
            FunctionPwm, PullNone: Gp0Pwm0A,
            /// PIO0 Function alias for pin [crate::Pins::gpio0].
            FunctionPio0, PullNone: Gp0Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio0].
            FunctionPio1, PullNone: Gp0Pio1
        }
    },

    /// GPIO 1 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 CSn`   | [crate::Gp1Spi0Csn]         |
    /// | `UART0 RX`   | [crate::Gp1Uart0Rx]         |
    /// | `I2C0 SCL`   | [crate::Gp1I2C0Scl]         |
    /// | `PWM0 B`     | [crate::Gp1Pwm0B]           |
    /// | `PIO0`       | [crate::Gp1Pio0]            |
    /// | `PIO1`       | [crate::Gp1Pio1]            |
    Gpio1 {
        name: rx,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio1].
            FunctionUart, PullNone: Gp1Uart0Rx,
            /// SPI Function alias for pin [crate::Pins::gpio1].
            FunctionSpi, PullNone: Gp1Spi0Csn,
            /// I2C Function alias for pin [crate::Pins::gpio1].
            FunctionI2C, PullUp: Gp1I2C0Scl,
            /// PWM Function alias for pin [crate::Pins::gpio1].
            FunctionPwm, PullNone: Gp1Pwm0B,
            /// PIO0 Function alias for pin [crate::Pins::gpio1].
            FunctionPio0, PullNone: Gp1Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio1].
            FunctionPio1, PullNone: Gp1Pio1
        }
    },

    /// GPIO 2 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 SCK`   | [crate::Gp2Spi0Sck]         |
    /// | `UART0 CTS`  | [crate::Gp2Uart0Cts]        |
    /// | `I2C1 SDA`   | [crate::Gp2I2C1Sda]         |
    /// | `PWM1 A`     | [crate::Gp2Pwm1A]           |
    /// | `PIO0`       | [crate::Gp2Pio0]            |
    /// | `PIO1`       | [crate::Gp2Pio1]            |
    Gpio2 {
        name: sda,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio2].
            FunctionUart, PullNone: Gp2Uart0Cts,
            /// SPI Function alias for pin [crate::Pins::gpio2].
            FunctionSpi, PullNone: Gp2Spi0Sck,
            /// I2C Function alias for pin [crate::Pins::gpio2].
            FunctionI2C, PullUp: Gp2I2C1Sda,
            /// PWM Function alias for pin [crate::Pins::gpio2].
            FunctionPwm, PullNone: Gp2Pwm1A,
            /// PIO0 Function alias for pin [crate::Pins::gpio2].
            FunctionPio0, PullNone: Gp2Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio2].
            FunctionPio1, PullNone: Gp2Pio1
        }
    },

    /// GPIO 3 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 TX`    | [crate::Gp3Spi0Tx]          |
    /// | `UART0 RTS`  | [crate::Gp3Uart0Rts]        |
    /// | `I2C1 SCL`   | [crate::Gp3I2C1Scl]         |
    /// | `PWM1 B`     | [crate::Gp3Pwm1B]           |
    /// | `PIO0`       | [crate::Gp3Pio0]            |
    /// | `PIO1`       | [crate::Gp3Pio1]            |
    Gpio3 {
        name: scl,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio3].
            FunctionUart, PullNone: Gp3Uart0Rts,
            /// SPI Function alias for pin [crate::Pins::gpio3].
            FunctionSpi, PullNone: Gp3Spi0Tx,
            /// I2C Function alias for pin [crate::Pins::gpio3].
            FunctionI2C, PullUp: Gp3I2C1Scl,
            /// PWM Function alias for pin [crate::Pins::gpio3].
            FunctionPwm, PullNone: Gp3Pwm1B,
            /// PIO0 Function alias for pin [crate::Pins::gpio3].
            FunctionPio0, PullNone: Gp3Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio3].
            FunctionPio1, PullNone: Gp3Pio1
        }
    },

    /// GPIO 4 is connected to the sleep pin. Low when host is asleep
    Gpio4 {
        name: sleep,
    },

    /// GPIO 6 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 SCK`   | [crate::Gp6Spi0Sck]         |
    /// | `UART1 CTS`  | [crate::Gp6Uart1Cts]        |
    /// | `I2C1 SDA`   | [crate::Gp6I2C1Sda]         |
    /// | `PWM3 A`     | [crate::Gp6Pwm3A]           |
    /// | `PIO0`       | [crate::Gp6Pio0]            |
    /// | `PIO1`       | [crate::Gp6Pio1]            |
    Gpio6 {
        name: d4,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio6].
            FunctionUart, PullNone: Gp6Uart1Cts,
            /// SPI Function alias for pin [crate::Pins::gpio6].
            FunctionSpi, PullNone: Gp6Spi0Sck,
            /// I2C Function alias for pin [crate::Pins::gpio6].
            FunctionI2C, PullUp: Gp6I2C1Sda,
            /// PWM Function alias for pin [crate::Pins::gpio6].
            FunctionPwm, PullNone: Gp6Pwm3A,
            /// PIO0 Function alias for pin [crate::Pins::gpio6].
            FunctionPio0, PullNone: Gp6Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio6].
            FunctionPio1, PullNone: Gp6Pio1
        }
    },

    /// GPIO 7 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 TX`    | [crate::Gp7Spi0Tx]          |
    /// | `UART1 RTS`  | [crate::Gp7Uart1Rts]        |
    /// | `I2C1 SCL`   | [crate::Gp7I2C1Scl]         |
    /// | `PWM3 B`     | [crate::Gp7Pwm3B]           |
    /// | `PIO0`       | [crate::Gp7Pio0]            |
    /// | `PIO1`       | [crate::Gp7Pio1]            |
    Gpio7 {
        name: d5,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio7].
            FunctionUart, PullNone: Gp7Uart1Rts,
            /// SPI Function alias for pin [crate::Pins::gpio7].
            FunctionSpi, PullNone: Gp7Spi0Tx,
            /// I2C Function alias for pin [crate::Pins::gpio7].
            FunctionI2C, PullUp: Gp7I2C1Scl,
            /// PWM Function alias for pin [crate::Pins::gpio7].
            FunctionPwm, PullNone: Gp7Pwm3B,
            /// PIO0 Function alias for pin [crate::Pins::gpio7].
            FunctionPio0, PullNone: Gp7Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio7].
            FunctionPio1, PullNone: Gp7Pio1
        }
    },

    /// GPIO 8 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 RX`    | [crate::Gp8Spi1Rx]          |
    /// | `UART1 TX`   | [crate::Gp8Uart1Tx]         |
    /// | `I2C0 SDA`   | [crate::Gp8I2C0Sda]         |
    /// | `PWM4 A`     | [crate::Gp8Pwm4A]           |
    /// | `PIO0`       | [crate::Gp8Pio0]            |
    /// | `PIO1`       | [crate::Gp8Pio1]            |
    Gpio8 {
        name: d6,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio8].
            FunctionUart, PullNone: Gp8Uart1Tx,
            /// SPI Function alias for pin [crate::Pins::gpio8].
            FunctionSpi, PullNone: Gp8Spi1Rx,
            /// I2C Function alias for pin [crate::Pins::gpio8].
            FunctionI2C, PullUp: Gp8I2C0Sda,
            /// PWM Function alias for pin [crate::Pins::gpio8].
            FunctionPwm, PullNone: Gp8Pwm4A,
            /// PIO0 Function alias for pin [crate::Pins::gpio8].
            FunctionPio0, PullNone: Gp8Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio8].
            FunctionPio1, PullNone: Gp8Pio1
        }
    },

    /// GPIO 9 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 CSn`   | [crate::Gp9Spi1Csn]         |
    /// | `UART1 RX`   | [crate::Gp9Uart1Rx]         |
    /// | `I2C0 SCL`   | [crate::Gp9I2C0Scl]         |
    /// | `PWM4 B`     | [crate::Gp9Pwm4B]           |
    /// | `PIO0`       | [crate::Gp9Pio0]            |
    /// | `PIO1`       | [crate::Gp9Pio1]            |
    Gpio9 {
        name: d9,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio9].
            FunctionUart, PullNone: Gp9Uart1Rx,
            /// SPI Function alias for pin [crate::Pins::gpio9].
            FunctionSpi, PullNone: Gp9Spi1Csn,
            /// I2C Function alias for pin [crate::Pins::gpio9].
            FunctionI2C, PullUp: Gp9I2C0Scl,
            /// PWM Function alias for pin [crate::Pins::gpio9].
            FunctionPwm, PullNone: Gp9Pwm4B,
            /// PIO0 Function alias for pin [crate::Pins::gpio9].
            FunctionPio0, PullNone: Gp9Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio9].
            FunctionPio1, PullNone: Gp9Pio1
        }
    },

    /// GPIO 10 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 SCK`   | [crate::Gp10Spi1Sck]        |
    /// | `UART1 CTS`  | [crate::Gp10Uart1Cts]       |
    /// | `I2C1 SDA`   | [crate::Gp10I2C1Sda]        |
    /// | `PWM5 A`     | [crate::Gp10Pwm5A]          |
    /// | `PIO0`       | [crate::Gp10Pio0]           |
    /// | `PIO1`       | [crate::Gp10Pio1]           |
    Gpio10 {
        name: d10,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio10].
            FunctionUart, PullNone: Gp10Uart1Cts,
            /// SPI Function alias for pin [crate::Pins::gpio10].
            FunctionSpi, PullNone: Gp10Spi1Sck,
            /// I2C Function alias for pin [crate::Pins::gpio10].
            FunctionI2C, PullUp: Gp10I2C1Sda,
            /// PWM Function alias for pin [crate::Pins::gpio10].
            FunctionPwm, PullNone: Gp10Pwm5A,
            /// PIO0 Function alias for pin [crate::Pins::gpio10].
            FunctionPio0, PullNone: Gp10Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio10].
            FunctionPio1, PullNone: Gp10Pio1
        }
    },

    /// GPIO 11 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 TX`    | [crate::Gp11Spi1Tx]         |
    /// | `UART1 RTS`  | [crate::Gp11Uart1Rts]       |
    /// | `I2C1 SCL`   | [crate::Gp11I2C1Scl]        |
    /// | `PWM5 B`     | [crate::Gp11Pwm5B]          |
    /// | `PIO0`       | [crate::Gp11Pio0]           |
    /// | `PIO1`       | [crate::Gp11Pio1]           |
    Gpio11 {
        name: d11,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio11].
            FunctionUart, PullNone: Gp11Uart1Rts,
            /// SPI Function alias for pin [crate::Pins::gpio11].
            FunctionSpi, PullNone: Gp11Spi1Tx,
            /// I2C Function alias for pin [crate::Pins::gpio11].
            FunctionI2C, PullUp: Gp11I2C1Scl,
            /// PWM Function alias for pin [crate::Pins::gpio11].
            FunctionPwm, PullNone: Gp11Pwm5B,
            /// PIO0 Function alias for pin [crate::Pins::gpio11].
            FunctionPio0, PullNone: Gp11Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio11].
            FunctionPio1, PullNone: Gp11Pio1
        }
    },

    /// GPIO 12 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 RX`    | [crate::Gp12Spi1Rx]         |
    /// | `UART0 TX`   | [crate::Gp12Uart0Tx]        |
    /// | `I2C0 SDA`   | [crate::Gp12I2C0Sda]        |
    /// | `PWM6 A`     | [crate::Gp12Pwm6A]          |
    /// | `PIO0`       | [crate::Gp12Pio0]           |
    /// | `PIO1`       | [crate::Gp12Pio1]           |
    Gpio12 {
        name: d12,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio12].
            FunctionUart, PullNone: Gp12Uart0Tx,
            /// SPI Function alias for pin [crate::Pins::gpio12].
            FunctionSpi, PullNone: Gp12Spi1Rx,
            /// I2C Function alias for pin [crate::Pins::gpio12].
            FunctionI2C, PullUp: Gp12I2C0Sda,
            /// PWM Function alias for pin [crate::Pins::gpio12].
            FunctionPwm, PullNone: Gp12Pwm6A,
            /// PIO0 Function alias for pin [crate::Pins::gpio12].
            FunctionPio0, PullNone: Gp12Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio12].
            FunctionPio1, PullNone: Gp12Pio1
        }
    },

    /// GPIO 13 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 CSn`   | [crate::Gp13Spi1Csn]        |
    /// | `UART0 RX`   | [crate::Gp13Uart0Rx]        |
    /// | `I2C0 SCL`   | [crate::Gp13I2C0Scl]        |
    /// | `PWM6 B`     | [crate::Gp13Pwm6B]          |
    /// | `PIO0`       | [crate::Gp13Pio0]           |
    /// | `PIO1`       | [crate::Gp13Pio1]           |
    Gpio13 {
        name: d13,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio13].
            FunctionUart, PullNone: Gp13Uart0Rx,
            /// SPI Function alias for pin [crate::Pins::gpio13].
            FunctionSpi, PullNone: Gp13Spi1Csn,
            /// I2C Function alias for pin [crate::Pins::gpio13].
            FunctionI2C, PullUp: Gp13I2C0Scl,
            /// PWM Function alias for pin [crate::Pins::gpio13].
            FunctionPwm, PullNone: Gp13Pwm6B,
            /// PIO0 Function alias for pin [crate::Pins::gpio13].
            FunctionPio0, PullNone: Gp13Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio13].
            FunctionPio1, PullNone: Gp13Pio1
        }
    },

    /// GPIO 16 is connected to RGB led
    Gpio16 {
        name: rgb_led,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio16].
            FunctionUart, PullNone: Gp16Uart0Tx,
            /// SPI Function alias for pin [crate::Pins::gpio16].
            FunctionSpi, PullNone: Gp16Spi0Rx,
            /// I2C Function alias for pin [crate::Pins::gpio16].
            FunctionI2C, PullUp: Gp16I2C0Sda,
            /// PWM Function alias for pin [crate::Pins::gpio16].
            FunctionPwm, PullNone: Gp16Pwm0A,
            /// PIO0 Function alias for pin [crate::Pins::gpio16].
            FunctionPio0, PullNone: Gp16Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio16].
            FunctionPio1, PullNone: Gp16Pio1
        }
    },

    /// GPIO 18 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 SCK`   | [crate::Gp18Spi0Sck]        |
    /// | `UART0 CTS`  | [crate::Gp18Uart0Cts]       |
    /// | `I2C1 SDA`   | [crate::Gp18I2C1Sda]        |
    /// | `PWM1 A`     | [crate::Gp18Pwm1A]          |
    /// | `PIO0`       | [crate::Gp18Pio0]           |
    /// | `PIO1`       | [crate::Gp18Pio1]           |
    Gpio18 {
        name: sck,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio18].
            FunctionUart, PullNone: Gp18Uart0Cts,
            /// SPI Function alias for pin [crate::Pins::gpio18].
            FunctionSpi, PullNone: Gp18Spi0Sck,
            /// I2C Function alias for pin [crate::Pins::gpio18].
            FunctionI2C, PullUp: Gp18I2C1Sda,
            /// PWM Function alias for pin [crate::Pins::gpio18].
            FunctionPwm, PullNone: Gp18Pwm1A,
            /// PIO0 Function alias for pin [crate::Pins::gpio18].
            FunctionPio0, PullNone: Gp18Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio18].
            FunctionPio1, PullNone: Gp18Pio1
        }
    },

    /// GPIO 19 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 TX`    | [crate::Gp19Spi0Tx]         |
    /// | `UART0 RTS`  | [crate::Gp19Uart0Rts]       |
    /// | `I2C1 SCL`   | [crate::Gp19I2C1Scl]        |
    /// | `PWM1 B`     | [crate::Gp19Pwm1B]          |
    /// | `PIO0`       | [crate::Gp19Pio0]           |
    /// | `PIO1`       | [crate::Gp19Pio1]           |
    Gpio19 {
        name: mosi,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio19].
            FunctionUart, PullNone: Gp19Uart0Rts,
            /// SPI Function alias for pin [crate::Pins::gpio19].
            FunctionSpi, PullNone: Gp19Spi0Tx,
            /// I2C Function alias for pin [crate::Pins::gpio19].
            FunctionI2C, PullUp: Gp19I2C1Scl,
            /// PWM Function alias for pin [crate::Pins::gpio19].
            FunctionPwm, PullNone: Gp19Pwm1B,
            /// PIO0 Function alias for pin [crate::Pins::gpio19].
            FunctionPio0, PullNone: Gp19Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio19].
            FunctionPio1, PullNone: Gp19Pio1
        }
    },

    /// GPIO 20 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI0 RX`    | [crate::Gp20Spi0Rx]         |
    /// | `UART1 TX`   | [crate::Gp20Uart1Tx]        |
    /// | `I2C0 SDA`   | [crate::Gp20I2C0Sda]        |
    /// | `PWM2 A`     | [crate::Gp20Pwm2A]          |
    /// | `PIO0`       | [crate::Gp20Pio0]           |
    /// | `PIO1`       | [crate::Gp20Pio1]           |
    Gpio20 {
        name: miso,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio20].
            FunctionUart, PullNone: Gp20Uart1Tx,
            /// SPI Function alias for pin [crate::Pins::gpio20].
            FunctionSpi, PullNone: Gp20Spi0Rx,
            /// I2C Function alias for pin [crate::Pins::gpio20].
            FunctionI2C, PullUp: Gp20I2C0Sda,
            /// PWM Function alias for pin [crate::Pins::gpio20].
            FunctionPwm, PullNone: Gp20Pwm2A,
            /// PIO0 Function alias for pin [crate::Pins::gpio20].
            FunctionPio0, PullNone: Gp20Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio20].
            FunctionPio1, PullNone: Gp20Pio1
        }
    },

    /// GPIO 24
    Gpio24 {
        name: d24,
        // TODO: Add aliases
    },

    /// GPIO 25
    Gpio25 {
        name: d25,
        // TODO: Add aliases
    },

    /// GPIO 26 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 SCK`   | [crate::Gp26Spi1Sck]        |
    /// | `UART1 CTS`  | [crate::Gp26Uart1Cts]       |
    /// | `I2C1 SDA`   | [crate::Gp26I2C1Sda]        |
    /// | `PWM5 A`     | [crate::Gp26Pwm5A]          |
    /// | `PIO0`       | [crate::Gp26Pio0]           |
    /// | `PIO1`       | [crate::Gp26Pio1]           |
    Gpio26 {
        name: a0,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio26].
            FunctionUart, PullNone: Gp26Uart1Cts,
            /// SPI Function alias for pin [crate::Pins::gpio26].
            FunctionSpi, PullNone: Gp26Spi1Sck,
            /// I2C Function alias for pin [crate::Pins::gpio26].
            FunctionI2C, PullUp: Gp26I2C1Sda,
            /// PWM Function alias for pin [crate::Pins::gpio26].
            FunctionPwm, PullNone: Gp26Pwm5A,
            /// PIO0 Function alias for pin [crate::Pins::gpio26].
            FunctionPio0, PullNone: Gp26Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio26].
            FunctionPio1, PullNone: Gp26Pio1
        }
    },

    /// GPIO 27 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 TX`    | [crate::Gp27Spi1Tx]         |
    /// | `UART1 RTS`  | [crate::Gp27Uart1Rts]       |
    /// | `I2C1 SCL`   | [crate::Gp27I2C1Scl]        |
    /// | `PWM5 B`     | [crate::Gp27Pwm5B]          |
    /// | `PIO0`       | [crate::Gp27Pio0]           |
    /// | `PIO1`       | [crate::Gp27Pio1]           |
    Gpio27 {
        name: a1,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio27].
            FunctionUart, PullNone: Gp27Uart1Rts,
            /// SPI Function alias for pin [crate::Pins::gpio27].
            FunctionSpi, PullNone: Gp27Spi1Tx,
            /// I2C Function alias for pin [crate::Pins::gpio27].
            FunctionI2C, PullUp: Gp27I2C1Scl,
            /// PWM Function alias for pin [crate::Pins::gpio27].
            FunctionPwm, PullNone: Gp27Pwm5B,
            /// PIO0 Function alias for pin [crate::Pins::gpio27].
            FunctionPio0, PullNone: Gp27Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio27].
            FunctionPio1, PullNone: Gp27Pio1
        }
    },

    /// GPIO 28 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    /// | `SPI1 RX`    | [crate::Gp28Spi1Rx]         |
    /// | `UART0 TX`   | [crate::Gp28Uart0Tx]        |
    /// | `I2C0 SDA`   | [crate::Gp28I2C0Sda]        |
    /// | `PWM6 A`     | [crate::Gp28Pwm6A]          |
    /// | `PIO0`       | [crate::Gp28Pio0]           |
    /// | `PIO1`       | [crate::Gp28Pio1]           |
    Gpio28 {
        name: a2,
        aliases: {
            /// UART Function alias for pin [crate::Pins::gpio28].
            FunctionUart, PullNone: Gp28Uart0Tx,
            /// SPI Function alias for pin [crate::Pins::gpio28].
            FunctionSpi, PullNone: Gp28Spi1Rx,
            /// I2C Function alias for pin [crate::Pins::gpio28].
            FunctionI2C, PullUp: Gp28I2C0Sda,
            /// PWM Function alias for pin [crate::Pins::gpio28].
            FunctionPwm, PullNone: Gp28Pwm6A,
            /// PIO0 Function alias for pin [crate::Pins::gpio28].
            FunctionPio0, PullNone: Gp28Pio0,
            /// PIO1 Function alias for pin [crate::Pins::gpio28].
            FunctionPio1, PullNone: Gp28Pio1
        }
    },

    /// GPIO 29 supports following functions:
    ///
    /// | Function     | Alias with applied function |
    /// |--------------|-----------------------------|
    Gpio29 {
        name: a3,
        // TODO: Add aliases
    },
);

// External crystal frequency, same as Raspberry Pi Pico
pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
