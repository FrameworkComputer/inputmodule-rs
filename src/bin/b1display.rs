//! Lotus LED Matrix Module
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use bsp::entry;
use cortex_m::delay::Delay;
//use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use mipidsi::Orientation;
use rp2040_hal::gpio::bank0::Gpio18;
use rp2040_hal::gpio::{Output, Pin, PushPull};
//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use lotus_input::lotus_lcd_hal as bsp;
//use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
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
use core::fmt::Write;
use heapless::String;

use lotus_input::control::*;
use lotus_input::graphics::*;
use lotus_input::serialnum::get_serialnum;

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
        .device_release(0x0010) // TODO: Assign dynamically based on crate version
        .device_class(USB_CLASS_CDC)
        .build();

    // Display SPI pins
    let _spi_sclk = pins.scl.into_mode::<bsp::hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.sda.into_mode::<bsp::hal::gpio::FunctionSpi>();
    let _spi_miso = pins.miso.into_mode::<bsp::hal::gpio::FunctionSpi>();
    let spi = bsp::hal::Spi::<_, _, 8>::new(pac.SPI1);
    // Display control pins
    let dc = pins.dc.into_push_pull_output();
    let mut lcd_led = pins.backlight.into_push_pull_output();
    let rst = pins.rstb.into_push_pull_output();

    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    // Create a DisplayInterface from SPI and DC pin, with no manual CS control
    let di = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);
    let mut disp = mipidsi::Builder::st7735s(di)
        .with_invert_colors(true) // Looks cooler. TODO: Should invert image not entire screen
        .with_orientation(Orientation::PortraitInverted(false))
        .init(&mut delay, Some(rst))
        .unwrap();
    disp.clear(Rgb565::WHITE).unwrap();

    let logo_rect = draw_logo(&mut disp).unwrap();
    draw_text(
        &mut disp,
        "Framework",
        Point::new(0, LOGO_OFFSET + logo_rect.size.height as i32),
    )
    .unwrap();

    // Wait until the background and image have been rendered otherwise
    // the screen will show random pixels for a brief moment
    lcd_led.set_high().unwrap();

    let sleep = pins.sleep.into_pull_down_input();

    let mut state = State {
        sleeping: SleepState::Awake,
    };

    let mut said_hello = false;

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut prev_timer = timer.get_counter().ticks();

    loop {
        // TODO: Current hardware revision does not have the sleep pin wired up :(
        // Go to sleep if the host is sleeping
        let _host_sleeping = sleep.is_low().unwrap();
        //handle_sleep(host_sleeping, &mut state, &mut matrix, &mut delay);

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
                            handle_sleep(go_sleeping, &mut state, &mut delay, &mut lcd_led);
                        } else if let SleepState::Awake = state.sleeping {
                            // While sleeping no command is handled, except waking up
                            handle_command(&command, &mut disp, logo_rect);
                        }
                    }
                }
            }
        }
    }
}

fn handle_sleep(
    go_sleeping: bool,
    state: &mut State,
    _delay: &mut Delay,
    lcd_led: &mut Pin<Gpio18, Output<PushPull>>,
) {
    match (state.sleeping.clone(), go_sleeping) {
        (SleepState::Awake, false) => (),
        (SleepState::Awake, true) => {
            state.sleeping = SleepState::Sleeping;
            //state.grid = display_sleep();

            // Turn off backlight
            lcd_led.set_low().unwrap();

            // TODO: Power Display controller down

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        (SleepState::Sleeping, true) => (),
        (SleepState::Sleeping, false) => {
            // Restore back grid before sleeping
            state.sleeping = SleepState::Awake;

            // TODO: Power display controller back on

            // Turn backlight back on
            lcd_led.set_high().unwrap();
        }
    }
}
