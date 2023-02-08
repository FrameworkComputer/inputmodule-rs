//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
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
    rom_data::reset_to_usb_boot,
    sio::Sio,
    usb,
    watchdog::Watchdog,
    Timer,
};
use fugit::RateExtU32;

use rp2040_hal::{
    gpio::{
        bank0::{Gpio26, Gpio27},
        Function, Pin, I2C,
    },
    pac::I2C1,
};
// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

// Used to demonstrate writing formatted strings
use core::fmt::Write;
use heapless::String;

pub mod lotus;
use lotus::LotusLedMatrix;
mod mapping;
use mapping::*;

type Foo = LotusLedMatrix<
    bsp::hal::I2C<
        I2C1,
        (
            bsp::hal::gpio::Pin<Gpio26, bsp::hal::gpio::Function<bsp::hal::gpio::I2C>>,
            bsp::hal::gpio::Pin<Gpio27, bsp::hal::gpio::Function<bsp::hal::gpio::I2C>>,
        ),
    >,
>;

type Grid = [[u8; 34]; 9];

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

    // Enable LED controller
    // SDB
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_high().unwrap();
    // INTB. Currently ignoring
    pins.intb.into_floating_input();

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

    matrix.set_scaling(0xA0).expect("failed to set scaling");

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
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut said_hello = false;

    let rotate = false;
    let mut grid = display_letters();
    //full_brightness(&mut matrix);
    //let mut grid = gradient();
    //let mut grid = zigzag();
    //let mut grid = double_gradient();
    let update_percentage = false;
    let mut p = 10;
    //let mut grid = percentage(p);

    fill_grid(grid, &mut matrix);
    let mut prev_timer = timer.get_counter();

    loop {
        if timer.get_counter() > prev_timer + 20_000 {
            fill_grid(grid, &mut matrix);
            if update_percentage {}

            if rotate {
                for x in 0..9 {
                    grid[x].rotate_right(1);
                }
            }
            prev_timer = timer.get_counter();
        }

        // A welcome message at the beginning
        if !said_hello && timer.get_counter() >= 2_000_000 {
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
                    // Convert to upper case
                    if count == 1 {
                        match buf[0] as char {
                            'r' => reset_to_usb_boot(0, 0),
                            '0' => grid = percentage(0),
                            '1' => grid = percentage(10),
                            '2' => grid = percentage(20),
                            '3' => grid = percentage(30),
                            '4' => grid = percentage(40),
                            '5' => grid = percentage(50),
                            '6' => grid = percentage(60),
                            '7' => grid = percentage(70),
                            '8' => grid = percentage(80),
                            '9' => grid = percentage(90),
                            _ => {}
                        }
                    }
                    buf.iter_mut().take(count).for_each(|b| {
                        b.make_ascii_uppercase();
                    });
                    // Send back to the host
                    let mut wr_ptr = &buf[..count];
                    while !wr_ptr.is_empty() {
                        match serial.write(wr_ptr) {
                            Ok(len) => wr_ptr = &wr_ptr[len..],
                            // On error, just drop unwritten data.
                            // One possible error is Err(WouldBlock), meaning the USB
                            // write buffer is full.
                            Err(_) => break,
                        };
                    }
                }
            }
        }
    }
}

fn display_letters() -> Grid {
    let mut grid: Grid = [[0; 34]; 9];

    display_letter(26, &mut grid, CAP_L);
    display_letter(20, &mut grid, CAP_O);
    display_letter(12, &mut grid, CAP_T);
    display_letter(0, &mut grid, CAP_S);
    display_letter(5, &mut grid, CAP_U);

    grid
}

fn display_letter(pos: usize, grid: &mut Grid, letter: SingleDisplayData) {
    for x in 0..8 {
        for y in 0..8 {
            let val = if letter[x] & (1 << y) > 0 { 0xFF } else { 0 };
            grid[8 - x][y + pos] = val;
        }
    }
}

// Gradient getting brighter from top to bottom
fn gradient() -> Grid {
    let mut grid: Grid = [[0; 34]; 9];
    for y in 0..34 {
        for x in 0..9 {
            grid[x][y] = (1 * (y + 1)) as u8;
        }
    }
    grid
}

// Fill a percentage of the rows from the bottom up
fn percentage(percentage: u16) -> Grid {
    let mut grid: Grid = [[0; 34]; 9];
    let first_row = 34 * percentage / 100;
    for y in (34 - first_row)..34 {
        for x in 0..9 {
            grid[x][y as usize] = 0xFF;
        }
    }
    grid
}

// Double sided gradient, bright in the middle, dim top and bottom
fn double_gradient() -> Grid {
    let mut grid: Grid = [[0; 34]; 9];
    for y in 0..(34 / 2) {
        for x in 0..9 {
            grid[x][y] = (1 * (y + 1)) as u8;
        }
    }
    for y in (34 / 2)..34 {
        for x in 0..9 {
            grid[x][y] = 34 - (1 * (y + 1)) as u8;
        }
    }
    grid
}

fn fill_grid(grid: Grid, matrix: &mut Foo) {
    for y in 0..34 {
        for x in 0..9 {
            matrix.device.pixel(x, y, grid[x as usize][y as usize]);
        }
    }
}

fn fill_grid_pixels(grid: Grid, matrix: &mut Foo) {
    for y in 0..34 {
        for x in 0..9 {
            matrix.device.pixel(x, y, grid[x as usize][y as usize]);
        }
    }
}

fn full_brightness(matrix: &mut Foo) {
    // Fills every pixel individually
    matrix.fill_brightness(0xFF).unwrap();

    // Fills full page at once
    //matrix.device.fill(0xFF).unwrap();
}

fn zigzag() -> Grid {
    let mut grid: Grid = [[0; 34]; 9];
    // 1st Right to left
    for i in 0..9 {
        grid[i][i] = 0xFF;
    }
    // 1st Left to right
    for i in 0..9 {
        grid[8 - i][9 + i] = 0xFF;
    }
    // 2nd right to left
    for i in 0..9 {
        grid[i][18 + i] = 0xFF;
    }
    // 2nd left to right
    for i in 0..9 {
        if 27 + i < 34 {
            grid[8 - i][27 + i] = 0xFF;
        }
    }
    grid[1][33] = 0xFF;
    grid
}

// End of file
