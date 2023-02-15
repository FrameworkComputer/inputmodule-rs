//! Lotus LED Matrix Module
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use bsp::entry;
use cortex_m::delay::Delay;
//use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};

//#[cfg(debug_assertions)]
//use panic_probe as _;
use rp2040_panic_usb_boot as _;

// TODO: Doesn't work yet, unless I panic right at the beginning of main
//#[cfg(not(debug_assertions))]
//use core::panic::PanicInfo;
//#[cfg(not(debug_assertions))]
//#[panic_handler]
//fn panic(_info: &PanicInfo) -> ! {
//    let mut pac = pac::Peripherals::take().unwrap();
//    let core = pac::CorePeripherals::take().unwrap();
//    let mut watchdog = Watchdog::new(pac.WATCHDOG);
//    let sio = Sio::new(pac.SIO);
//
//    let clocks = init_clocks_and_plls(
//        bsp::XOSC_CRYSTAL_FREQ,
//        pac.XOSC,
//        pac.CLOCKS,
//        pac.PLL_SYS,
//        pac.PLL_USB,
//        &mut pac.RESETS,
//        &mut watchdog,
//    )
//    .ok()
//    .unwrap();
//
//    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
//
//    let pins = bsp::Pins::new(
//        pac.IO_BANK0,
//        pac.PADS_BANK0,
//        sio.gpio_bank0,
//        &mut pac.RESETS,
//    );
//
//    let mut led_enable = pins.sdb.into_push_pull_output();
//    led_enable.set_high().unwrap();
//
//    let i2c = bsp::hal::I2C::i2c1(
//        pac.I2C1,
//        pins.gpio26.into_mode::<bsp::hal::gpio::FunctionI2C>(),
//        pins.gpio27.into_mode::<bsp::hal::gpio::FunctionI2C>(),
//        1000.kHz(),
//        &mut pac.RESETS,
//        &clocks.peripheral_clock,
//    );
//
//    let mut matrix = LotusLedMatrix::configure(i2c);
//    matrix
//        .setup(&mut delay)
//        .expect("failed to setup rgb controller");
//
//    matrix.set_scaling(100).expect("failed to set scaling");
//    let grid = display_panic();
//    fill_grid_pixels(grid, &mut matrix);
//
//    loop {}
//}

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
use usbd_serial::{SerialPort, USB_CLASS_CDC};

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

//                            FRA                - Framwork
//                               KDE             - Lotus C2 LED Matrix
//                                  AM           - Atemitech
//                                    00         - Default Configuration
//                                      00000000 - Device Identifier
const DEFAULT_SERIAL: &str = "FRAKDEAM0000000000";
// Get serial number from last 4K block of the first 1M
const FLASH_OFFSET: usize = 0x10000000;
const LAST_4K_BLOCK: usize = 0xff000;
const SERIALNUM_LEN: usize = 18;

fn get_serialnum() -> Option<&'static str> {
    // Flash is mapped into memory, just read it from there
    let ptr: *const u8 = (FLASH_OFFSET + LAST_4K_BLOCK) as *const u8;
    unsafe {
        let slice: &[u8] = core::slice::from_raw_parts(ptr, SERIALNUM_LEN);
        if slice[0] == 0xFF || slice[0] == 0x00 {
            return None;
        }
        core::str::from_utf8(slice).ok()
    }
}

#[derive(Clone)]
pub struct Grid([[u8; HEIGHT]; WIDTH]);
impl Default for Grid {
    fn default() -> Self {
        Grid([[0; HEIGHT]; WIDTH])
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum SleepState {
    Awake,
    Sleeping(Grid),
}

pub struct State {
    grid: Grid,
    col_buffer: Grid,
    animate: bool,
    brightness: u8,
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

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, 0x0020))
        .manufacturer("Framework")
        .product("Lotus LED Matrix")
        .serial_number(serialnum)
        .device_class(USB_CLASS_CDC)
        .build();

    // Enable LED controller
    // SDB
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_high().unwrap();
    // INTB. Currently ignoring
    pins.intb.into_floating_input();

    let sleep = pins.sleep.into_pull_down_input();

    let i2c = bsp::hal::I2C::i2c1(
        pac.I2C1,
        pins.gpio26.into_mode::<bsp::hal::gpio::FunctionI2C>(),
        pins.gpio27.into_mode::<bsp::hal::gpio::FunctionI2C>(),
        1000.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut state = State {
        grid: percentage(100),
        col_buffer: Grid::default(),
        animate: false,
        brightness: 120,
        sleeping: SleepState::Awake,
    };

    let mut matrix = LotusLedMatrix::configure(i2c);
    matrix
        .setup(&mut delay)
        .expect("failed to setup rgb controller");

    matrix
        .set_scaling(state.brightness)
        .expect("failed to set scaling");

    let mut said_hello = false;

    fill_grid_pixels(&state.grid, &mut matrix);

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut prev_timer = timer.get_counter().ticks();

    loop {
        // TODO: Current hardware revision does not have the sleep pin wired up :(
        // Go to sleep if the host is sleeping
        let _host_sleeping = sleep.is_low().unwrap();
        //handle_sleep(host_sleeping, &mut state, &mut matrix, &mut delay);

        // Handle period display updates. Don't do it too often
        if timer.get_counter().ticks() > prev_timer + 20_000 {
            fill_grid_pixels(&state.grid, &mut matrix);
            if state.animate {
                for x in 0..WIDTH {
                    state.grid.0[x].rotate_right(1);
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
                        handle_command(&command, &mut state, &mut matrix);

                        if let Command::Sleep(go_sleeping) = command {
                            handle_sleep(go_sleeping, &mut state, &mut matrix, &mut delay);
                        }

                        fill_grid_pixels(&state.grid, &mut matrix);
                    }
                }
            }
        }
    }
}

fn handle_sleep(go_sleeping: bool, state: &mut State, matrix: &mut Foo, delay: &mut Delay) {
    match (state.sleeping.clone(), go_sleeping) {
        (SleepState::Awake, false) => (),
        (SleepState::Awake, true) => {
            state.sleeping = SleepState::Sleeping(state.grid.clone());
            //state.grid = display_sleep();
            fill_grid_pixels(&state.grid, matrix);

            // Slowly decrease brightness
            delay.delay_ms(1000);
            let mut brightness = state.brightness;
            loop {
                delay.delay_ms(100);
                brightness = if brightness <= 5 { 0 } else { brightness - 5 };
                matrix
                    .set_scaling(brightness)
                    .expect("failed to set scaling");
                if brightness == 0 {
                    break;
                }
            }

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        (SleepState::Sleeping(_), true) => (),
        (SleepState::Sleeping(old_grid), false) => {
            // Restore back grid before sleeping
            state.sleeping = SleepState::Awake;
            state.grid = old_grid;
            fill_grid_pixels(&state.grid, matrix);

            // Slowly increase brightness
            delay.delay_ms(1000);
            let mut brightness = 0;
            loop {
                delay.delay_ms(100);
                brightness = if brightness >= state.brightness - 5 {
                    state.brightness
                } else {
                    brightness + 5
                };
                matrix
                    .set_scaling(brightness)
                    .expect("failed to set scaling");
                if brightness == state.brightness {
                    break;
                }
            }
        }
    }
}
