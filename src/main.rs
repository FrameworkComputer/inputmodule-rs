//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
mod lotus_led_hal;
use lotus_led_hal as bsp;
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
use usbd_serial::SerialPort;

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

pub mod lotus;
use lotus::LotusLedMatrix;

pub mod mapping;

pub mod patterns;
use patterns::*;

mod control;
use control::*;

#[entry]
fn main() -> ! {
    info!("Program start");
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

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, 0x0020))
        .manufacturer("Framework")
        .product("Lotus LED Matrix")
        .serial_number("FRAKDE??0000000000")
        .device_class(2) // Communications and CDC Control. From: https://www.usb.org/defined-class-codes
        .build();

    // Enable LED controller
    // SDB
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_high().unwrap();
    // INTB. Currently ignoring
    pins.intb.into_floating_input();

    let sleep = pins.sleep.into_pull_down_input();
    let _sleeping = sleep.is_low().unwrap();

    let i2c = bsp::hal::I2C::i2c1(
        pac.I2C1,
        pins.gpio26.into_mode::<bsp::hal::gpio::FunctionI2C>(),
        pins.gpio27.into_mode::<bsp::hal::gpio::FunctionI2C>(),
        1000.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut matrix = LotusLedMatrix::configure(i2c);
    matrix
        .setup(&mut delay)
        .expect("failed to setup rgb controller");

    matrix.set_scaling(150).expect("failed to set scaling");

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut said_hello = false;

    let rotate = false;
    // Default on
    let mut grid = percentage(100);
    fill_grid(grid, &mut matrix);

    let mut prev_timer = timer.get_counter().ticks();

    loop {
        if timer.get_counter().ticks() > prev_timer + 20_000 {
            //fill_grid(grid, &mut matrix);
            if rotate {
                for x in 0..9 {
                    grid[x].rotate_right(1);
                }
            }
            prev_timer = timer.get_counter().ticks();
        }

        // A welcome message at the beginning
        if !said_hello && timer.get_counter().ticks() >= 2_000_000 {
            said_hello = true;
            let _ = serial.write(b"Hello, World!\r\n");

            let time = timer.get_counter();
            let mut text: String<64> = String::new();
            writeln!(&mut text, "Current timer ticks: {}", time).unwrap();

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
                        handle_command(command, &mut grid, &mut matrix);
                        fill_grid(grid, &mut matrix);
                    }
                }
            }
        }
    }
}

// End of file
