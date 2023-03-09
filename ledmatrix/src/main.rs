//! Lotus LED Matrix Module
#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

use cortex_m::delay::Delay;
//use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use rp2040_hal::{
    gpio::bank0::Gpio29,
    rosc::{Enabled, RingOscillator},
};
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
use bsp::entry;
use lotus_inputmodules::{games::game_of_life, lotus_led_hal as bsp};
//use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
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

use lotus_inputmodules::control::*;
use lotus_inputmodules::games::{pong, snake};
use lotus_inputmodules::lotus::LotusLedMatrix;
use lotus_inputmodules::matrix::*;
use lotus_inputmodules::patterns::*;
use lotus_inputmodules::serialnum::{device_release, get_serialnum};

//                            FRA                - Framwork
//                               KDE             - Lotus C2 LED Matrix
//                                  AM           - Atemitech
//                                    00         - Default Configuration
//                                      00000000 - Device Identifier
const DEFAULT_SERIAL: &str = "FRAKDEAM0000000000";

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
    //rp2040_pac::rosc::RANDOMBIT::read(&self)
    let rosc = rp2040_hal::rosc::RingOscillator::new(pac.ROSC);
    let rosc = rosc.initialize();

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
        .max_power(200) // Device uses roughly 164mW when all LEDs are at full brightness
        .device_release(device_release())
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
        pins.gpio26.into_mode::<gpio::FunctionI2C>(),
        pins.gpio27.into_mode::<gpio::FunctionI2C>(),
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
        game: None,
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
    let mut game_timer = timer.get_counter().ticks();

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
                    let random = get_random_byte(&rosc);
                    match (parse_command(count, &buf), &state.sleeping) {
                        (Some(Command::Sleep(go_sleeping)), _) => {
                            handle_sleep(
                                go_sleeping,
                                &mut state,
                                &mut matrix,
                                &mut delay,
                                &mut led_enable,
                            );
                        }
                        (Some(c @ Command::BootloaderReset), _)
                        | (Some(c @ Command::IsSleeping), _) => {
                            if let Some(response) =
                                handle_command(&c, &mut state, &mut matrix, random)
                            {
                                let _ = serial.write(&response);
                            };
                        }
                        (Some(command), SleepState::Awake) => {
                            // While sleeping no command is handled, except waking up
                            if let Some(response) =
                                handle_command(&command, &mut state, &mut matrix, random)
                            {
                                let _ = serial.write(&response);
                            };
                            // Must write AFTER writing response, otherwise the
                            // client interprets this debug message as the response
                            let mut text: String<64> = String::new();
                            write!(
                                &mut text,
                                "Handled command {}:{}:{}:{}\r\n",
                                buf[0], buf[1], buf[2], buf[3]
                            )
                            .unwrap();
                            fill_grid_pixels(&state.grid, &mut matrix);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Handle game state
        let game_step_diff = match state.game {
            Some(GameState::Pong(ref pong_state)) => 100_000 - 5_000 * pong_state.speed,
            Some(GameState::Snake(_)) => 500_000,
            Some(GameState::GameOfLife(_)) => 500_000,
            _ => 500_000,
        };
        if timer.get_counter().ticks() > game_timer + game_step_diff {
            let random = get_random_byte(&rosc);
            match state.game {
                Some(GameState::GameOfLife(_)) => {
                    let _ = serial.write(b"GOL Game step\r\n");
                    game_of_life::game_step(&mut state, random);
                }
                Some(GameState::Pong(_)) => {
                    let _ = serial.write(b"Pong Game step\r\n");
                    pong::game_step(&mut state, random);
                }
                Some(GameState::Snake(_)) => {
                    let _ = serial.write(b"Snake Game step\r\n");
                    let (direction, game_over, points, (x, y)) =
                        snake::game_step(&mut state, random);

                    if game_over {
                        // TODO: Show score
                    } else {
                        let mut text: String<64> = String::new();
                        write!(
                            &mut text,
                            "Dir: {:?} Status: {}, Points: {}, Head: ({},{})\r\n",
                            direction, game_over, points, x, y
                        )
                        .unwrap();
                        let _ = serial.write(text.as_bytes());
                    }
                }
                None => {}
            }
            game_timer = timer.get_counter().ticks();
        }
    }
}

fn get_random_byte(rosc: &RingOscillator<Enabled>) -> u8 {
    let mut byte = 0;
    for i in 0..8 {
        byte += (rosc.get_random_bit() as u8) << i;
    }
    byte
}

fn handle_sleep(
    go_sleeping: bool,
    state: &mut State,
    matrix: &mut Foo,
    delay: &mut Delay,
    led_enable: &mut gpio::Pin<Gpio29, gpio::Output<gpio::PushPull>>,
) {
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

            // Turn LED controller off to save power
            led_enable.set_low().unwrap();

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        (SleepState::Sleeping(_), true) => (),
        (SleepState::Sleeping(old_grid), false) => {
            // Restore back grid before sleeping
            state.sleeping = SleepState::Awake;
            state.grid = old_grid;
            fill_grid_pixels(&state.grid, matrix);

            // Power LED controller back on
            led_enable.set_high().unwrap();

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
