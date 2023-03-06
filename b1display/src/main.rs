//! Lotus LED Matrix Module
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use bsp::entry;
use cortex_m::delay::Delay;
//use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use rp2040_hal::gpio::bank0::Gpio18;
use rp2040_hal::gpio::{Output, Pin, PushPull};
//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_hal::blocking::spi;
use st7306_lcd::{Orientation, ST7306};

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use lotus_inputmodules::lotus_lcd_hal as bsp;
//use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    gpio,
    usb,
    watchdog::Watchdog,
    Timer,
};
use fugit::RateExtU32;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Used to demonstrate writing formatted strings
use core::fmt::{Debug, Write};
use heapless::String;

use lotus_inputmodules::control::*;
use lotus_inputmodules::graphics::*;
use lotus_inputmodules::serialnum::{device_release, get_serialnum};

//                            FRA                - Framwork
//                               KDE             - Lotus C2 LED Matrix
//                                  AM           - Atemitech
//                                    00         - Default Configuration
//                                      00000000 - Device Identifier
const DEFAULT_SERIAL: &str = "FRAKDEAM0000000000";

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum SleepState {
    Awake,
    Sleeping,
}

pub struct State {
    sleeping: SleepState,
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
    let mut serial = SerialPort::new(&usb_bus);

    let serialnum = if let Some(serialnum) = get_serialnum() {
        serialnum
    } else {
        DEFAULT_SERIAL
    };

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, 0x0021))
        .manufacturer("Framework")
        .product("Lotus B1 Display")
        .serial_number(serialnum)
        .max_power(500) // TODO: Check how much
        .device_release(device_release()) // TODO: Assign dynamically based on crate version
        .device_class(USB_CLASS_CDC)
        .build();

    // Display SPI pins
    let _spi_sclk = pins.scl.into_mode::<bsp::hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.sda.into_mode::<bsp::hal::gpio::FunctionSpi>();
    let _spi_miso = pins.miso.into_mode::<bsp::hal::gpio::FunctionSpi>();
    let spi = bsp::hal::Spi::<_, _, 8>::new(pac.SPI0);
    // Display control pins
    let dc = pins.dc.into_push_pull_output();
    //let mut lcd_led = pins.backlight.into_push_pull_output();
    let mut cs = pins.cs.into_push_pull_output();
    cs.set_low().unwrap();
    let rst = pins.rstb.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let mut disp: ST7306<
        rp2040_hal::Spi<rp2040_hal::spi::Enabled, pac::SPI0, 8>,
        Pin<gpio::bank0::Gpio20, Output<PushPull>>,
        Pin<gpio::bank0::Gpio17, Output<PushPull>>,
        Pin<gpio::bank0::Gpio21, Output<PushPull>>,
        25,
        200,
    > = ST7306::new(spi, dc, cs, rst, false, 300, 460);
    disp.init(&mut delay).unwrap();

    disp.clear(Rgb565::BLACK).unwrap();

    let logo_rect = draw_logo(&mut disp).unwrap();
    Rectangle::new(Point::new(10, 10), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut disp)
        .unwrap();
    Rectangle::new(Point::new(20, 20), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut disp)
        .unwrap();
    Rectangle::new(Point::new(30, 30), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut disp)
        .unwrap();
    Rectangle::new(Point::new(40, 40), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut disp)
        .unwrap();
    Rectangle::new(Point::new(50, 50), Size::new(10, 10))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut disp)
        .unwrap();
    draw_text(
        &mut disp,
        "Framework",
        Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y + logo_rect.size.height as i32),
    )
    .unwrap();

    let sleep = pins.sleep.into_pull_down_input();

    let mut state = State {
        sleeping: SleepState::Awake,
    };

    let mut said_hello = false;

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut prev_timer = timer.get_counter().ticks();

    loop {
        // Go to sleep if the host is sleeping
        let host_sleeping = sleep.is_low().unwrap();
        handle_sleep(host_sleeping, &mut state, &mut delay, &mut disp);

        // Handle period display updates. Don't do it too often
        if timer.get_counter().ticks() > prev_timer + 20_000 {
            // TODO: Update display
            prev_timer = timer.get_counter().ticks();
        }

        // A welcome message at the beginning
        if !said_hello && timer.get_counter().ticks() >= 2_000_000 {
            said_hello = true;
            let _ = serial.write(b"Hello, World!\r\n");

            let time = timer.get_counter();
            let mut text: String<64> = String::new();
            write!(&mut text, "Current timer ticks: {}\r\n", time).unwrap();

            // This only works reliably because the number of bytes written to
            // the serial port is smaller than the buffers available to the USB
            // peripheral. In general, the return value should be handled, so that
            // bytes not transferred yet don't get lost.
            let _ = serial.write(text.as_bytes());
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
                            handle_sleep(host_sleeping, &mut state, &mut delay, &mut disp);
                        } else if let SleepState::Awake = state.sleeping {
                            // While sleeping no command is handled, except waking up
                            //handle_command(&command, &mut disp, logo_rect);
                            if let Some(response) = handle_command(&command, &mut disp, logo_rect) {
                                let _ = serial.write(&response);
                            };
                        }
                    }
                }
            }
        }
    }
}

fn handle_sleep<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize>(
    go_sleeping: bool,
    state: &mut State,
    _delay: &mut Delay,
    disp: &mut ST7306<SPI, DC, CS, RST, COLS, ROWS>,
) where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    <SPI as spi::Write<u8>>::Error: Debug,
{
    match (state.sleeping.clone(), go_sleeping) {
        (SleepState::Awake, false) => (),
        (SleepState::Awake, true) => {
            state.sleeping = SleepState::Sleeping;
            //state.grid = display_sleep();

            // Turn off backlight
            disp.on_off(false);

            // TODO: Power Display controller down

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        (SleepState::Sleeping, true) => (),
        (SleepState::Sleeping, false) => {
            // Restore back grid before sleeping
            state.sleeping = SleepState::Awake;

            // TODO: Power display controller back on
            disp.on_off(true);
        }
    }
}
