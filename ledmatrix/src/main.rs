//! LED Matrix Module
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

#[derive(PartialEq, Eq)]
#[allow(dead_code)]
enum SleepMode {
    /// Instantly go to sleep ant
    Instant,
    /// Fade brightness out and in slowly when sleeping/waking-up
    Fading,
    // Display "SLEEP" when sleeping, instead of turning LEDs off
    Debug,
}

/// Static configuration whether sleep shohld instantly turn all LEDs on/off or
/// slowly fade themm on/off
const SLEEP_MODE: SleepMode = SleepMode::Fading;

const STARTUP_ANIMATION: bool = true;

/// Go to sleep after 60s awake
const SLEEP_TIMEOUT: u64 = 60_000_000;

/// List maximum current as 500mA in the USB descriptor
const MAX_CURRENT: usize = 500;

/// Maximum brightness out of 255
/// On HW Rev 1 from BizLink set to 94 to have just below 500mA current draw.
/// BizLink HW Rev 2 has a larger current limiting resistor
#[cfg(feature = "10k")]
const MAX_BRIGHTNESS: u8 = 94;
#[cfg(not(feature = "10k"))]
const MAX_BRIGHTNESS: u8 = 255;

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
//    let mut matrix = LedMatrix::configure(i2c);
//    matrix
//        .setup(&mut delay)
//        .expect("failed to setup rgb controller");
//
//    set_brightness(state, 255, &mut matrix);
//    let grid = display_panic();
//    fill_grid_pixels(state, &mut matrix);
//
//    loop {}
//}

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use bsp::entry;
#[cfg(not(feature = "evt"))]
use fl16_inputmodules::fl16::DVT2_CALC_PIXEL;
#[cfg(feature = "evt")]
use fl16_inputmodules::fl16::EVT_CALC_PIXEL;
use fl16_inputmodules::{games::game_of_life, led_hal as bsp};
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

use fl16_inputmodules::control::*;
use fl16_inputmodules::fl16::LedMatrix;
use fl16_inputmodules::games::{pong, snake};
use fl16_inputmodules::matrix::*;
use fl16_inputmodules::patterns::*;
use fl16_inputmodules::serialnum::{device_release, get_serialnum};

//                            FRA                - Framwork
//                               KDE             - C1 LED Matrix
//                                  BZ           - BizLink
//                                    01         - SKU, Default Configuration
//                                      00000000 - Device Identifier
const DEFAULT_SERIAL: &str = "FRAKDEBZ0100000000";

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
        serialnum.serialnum
    } else {
        DEFAULT_SERIAL
    };

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x32ac, 0x0020))
        .manufacturer("Framework Computer Inc")
        .product("LED Matrix Input Module")
        .serial_number(serialnum)
        .max_power(MAX_CURRENT)
        .device_release(device_release())
        .device_class(USB_CLASS_CDC)
        .build();

    // Enable LED controller
    // SDB
    let mut led_enable = pins.sdb.into_push_pull_output();
    led_enable.set_high().unwrap();
    // INTB. Currently ignoring
    pins.intb.into_floating_input();

    let i2c = bsp::hal::I2C::i2c1(
        pac.I2C1,
        pins.gpio26.into_mode::<gpio::FunctionI2C>(),
        pins.gpio27.into_mode::<gpio::FunctionI2C>(),
        1000.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let dip1 = pins.dip1.into_pull_up_input();

    let mut state = LedmatrixState {
        grid: percentage(0),
        col_buffer: Grid::default(),
        animate: false,
        brightness: 51, // Default to 51/255 = 20% brightness
        sleeping: SleepState::Awake,
        game: None,
        animation_period: 31_250, // 31,250 us = 32 FPS
        pwm_freq: PwmFreqArg::P29k,
        debug_mode: false,
    };
    state.debug_mode = dip1.is_low().unwrap();
    if !show_startup_animation(&state) {
        // If no startup animation, render another pattern
        // Lighting up every second column is a good pattern to test for noise.
        state.grid = every_nth_col(2);
    };

    #[cfg(feature = "evt")]
    let mut matrix = LedMatrix::new(i2c, EVT_CALC_PIXEL);
    #[cfg(not(feature = "evt"))]
    let mut matrix = LedMatrix::new(i2c, DVT2_CALC_PIXEL);
    matrix
        .setup(&mut delay)
        .expect("failed to setup RGB controller");

    // EVT
    #[cfg(feature = "evt")]
    matrix
        .device
        .sw_enablement(is31fl3741::SwSetting::Sw1Sw9)
        .unwrap();
    // DVT
    #[cfg(not(feature = "evt"))]
    matrix
        .device
        .sw_enablement(is31fl3741::SwSetting::Sw1Sw8)
        .unwrap();

    matrix
        .set_scaling(MAX_BRIGHTNESS)
        .expect("failed to set scaling");

    matrix.device.set_pwm_freq(state.pwm_freq.into()).unwrap();

    fill_grid_pixels(&state, &mut matrix);

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut animation_timer = timer.get_counter().ticks();
    let mut game_timer = timer.get_counter().ticks();
    let mut sleep_timer = timer.get_counter().ticks();

    let mut startup_percentage = Some(0);

    // Detect whether the sleep pin is connected
    // Early revisions of the hardware didn't have it wired up, if that is the
    // case we have to ignore its state.
    let mut sleep_present = false;
    let sleep = pins.sleep.into_pull_up_input();
    if sleep.is_low().unwrap() {
        sleep_present = true;
    }
    let sleep = sleep.into_pull_down_input();
    if sleep.is_high().unwrap() {
        sleep_present = true;
    }

    let mut usb_initialized = false;
    let mut usb_suspended = false;
    let mut last_usb_suspended = usb_suspended;
    let mut sleep_reason: Option<SleepReason> = None;
    let mut last_sleep_reason: Option<SleepReason>;
    let mut last_host_sleep = sleep.is_low().unwrap();

    loop {
        last_sleep_reason = sleep_reason;

        state.debug_mode = dip1.is_low().unwrap();
        if sleep_present {
            // Go to sleep if the host is sleeping
            let host_sleeping = sleep.is_low().unwrap();
            let host_sleep_changed = host_sleeping != last_host_sleep;
            // Change sleep state either if SLEEP# has changed
            // Or if it currently sleeping. Don't change if not sleeping
            // because then sleep is controlled by timing or by API.
            if host_sleep_changed || host_sleeping {
                sleep_reason = assign_sleep_reason(
                    last_sleep_reason,
                    sleep_reason,
                    host_sleeping,
                    host_sleep_changed,
                    SleepReason::SleepPin,
                );
            }
            last_host_sleep = host_sleeping;
        }

        // Change sleep state either if SLEEP# has changed
        // Or if it currently sleeping. Don't change if not sleeping
        // because then sleep is controlled by timing or by API.
        let usb_suspended_changed = usb_suspended != last_usb_suspended;
        // Only if USB was previously initialized,
        // since the OS puts the device into suspend before it's fully
        // initialized for the first time. But we don't want to show the
        // sleep animation during startup.
        if usb_initialized && (usb_suspended_changed || usb_suspended) {
            sleep_reason = assign_sleep_reason(
                last_sleep_reason,
                sleep_reason,
                usb_suspended,
                usb_suspended_changed,
                SleepReason::UsbSuspend,
            );
        }
        last_usb_suspended = usb_suspended;

        // Go to sleep after the timer has run out
        if timer.get_counter().ticks() > sleep_timer + SLEEP_TIMEOUT && !state.debug_mode {
            sleep_reason = assign_sleep_reason(
                last_sleep_reason,
                sleep_reason,
                true,
                true,
                SleepReason::Timeout,
            );
        }
        // Constantly resetting timer during sleep is same as reset it once on waking up.
        // This means the timer ends up counting the time spent awake.
        if sleep_reason.is_some() {
            sleep_timer = timer.get_counter().ticks();
        }

        handle_sleep(
            sleep_reason,
            &mut state,
            &mut matrix,
            &mut delay,
            &mut led_enable,
        );

        // Handle period display updates. Don't do it too often
        let render_again = timer.get_counter().ticks() > animation_timer + state.animation_period;
        if matches!(state.sleeping, SleepState::Awake) && render_again {
            // On startup slowly turn the screen on - it's a pretty effect :)
            match startup_percentage {
                Some(p) if p <= 100 && show_startup_animation(&state) => {
                    state.grid = percentage(p);
                    startup_percentage = Some(p + 5);
                }
                _ => {}
            }

            fill_grid_pixels(&state, &mut matrix);
            if state.animate {
                for x in 0..WIDTH {
                    state.grid.0[x].rotate_right(1);
                }
            }
            animation_timer = timer.get_counter().ticks();
        }

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            match usb_dev.state() {
                // Default: Device has just been created or reset
                // Addressed: Device has received an address for the host
                UsbDeviceState::Default | UsbDeviceState::Addressed => {
                    usb_initialized = false;
                    usb_suspended = false;
                    // Must not display anything or windows cannot enumerate properly
                }
                // Configured and is fully operational
                UsbDeviceState::Configured => {
                    usb_initialized = true;
                    usb_suspended = false;
                }
                // Never occurs here. Only if poll() returns false
                UsbDeviceState::Suspend => {
                    panic!("Never occurs here. Only if poll() returns false")
                }
            }
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
                        // Handle bootloader command without any delay
                        // No need, it'll reset the device anyways
                        (Some(c @ Command::BootloaderReset), _) => {
                            handle_command(&c, &mut state, &mut matrix, random);
                        }
                        (Some(command), _) => {
                            if let Command::Sleep(go_sleeping) = command {
                                sleep_reason = assign_sleep_reason(
                                    last_sleep_reason,
                                    sleep_reason,
                                    go_sleeping,
                                    true,
                                    SleepReason::Command,
                                );
                            } else {
                                // If already sleeping, wake up.
                                // This means every command will wake the device up.
                                // Much more convenient than having to send the wakeup commmand.
                                sleep_reason = None;
                            }
                            // Make sure sleep animation only goes up to newly set brightness,
                            // if setting the brightness causes wakeup
                            if let SleepState::Sleeping((ref grid, _)) = state.sleeping {
                                if let Command::SetBrightness(new_brightness) = command {
                                    state.sleeping =
                                        SleepState::Sleeping((grid.clone(), new_brightness));
                                }
                            }
                            handle_sleep(
                                sleep_reason,
                                &mut state,
                                &mut matrix,
                                &mut delay,
                                &mut led_enable,
                            );

                            // If there's a very early command, cancel the startup animation
                            startup_percentage = None;

                            // Reset sleep timer when interacting with the device
                            // Very easy way to keep the device from going to sleep
                            sleep_timer = timer.get_counter().ticks();

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
                            // let _ = serial.write(text.as_bytes());

                            fill_grid_pixels(&state, &mut matrix);
                        }
                        (None, _) => {}
                    }
                }
            }
        } else {
            match usb_dev.state() {
                // No new data
                UsbDeviceState::Default | UsbDeviceState::Addressed => {
                    usb_initialized = false;
                    usb_suspended = false;
                }
                UsbDeviceState::Configured => {
                    usb_initialized = true;
                    usb_suspended = false;
                }
                UsbDeviceState::Suspend => {
                    usb_suspended = true;
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

fn dyn_sleep_mode(state: &LedmatrixState) -> SleepMode {
    if state.debug_mode {
        SleepMode::Debug
    } else {
        SLEEP_MODE
    }
}

fn debug_mode(state: &LedmatrixState) -> bool {
    dyn_sleep_mode(state) == SleepMode::Debug
}

fn show_startup_animation(state: &LedmatrixState) -> bool {
    // Show startup animation
    STARTUP_ANIMATION && !debug_mode(state)
}

fn assign_sleep_reason(
    previous: Option<SleepReason>,
    current: Option<SleepReason>,
    need_sleep: bool,
    // Whether the signal has actually changed in between firing
    signal_changed: bool,
    new: SleepReason,
) -> Option<SleepReason> {
    if !need_sleep {
        None
    } else if current.is_some() && (Some(new) == previous || !signal_changed) {
        current
    } else {
        Some(new)
    }
}

// Will do nothing if already in the right state
fn handle_sleep(
    sleep_reason: Option<SleepReason>,
    state: &mut LedmatrixState,
    matrix: &mut Foo,
    delay: &mut Delay,
    led_enable: &mut gpio::Pin<Gpio29, gpio::Output<gpio::PushPull>>,
) {
    match (state.sleeping.clone(), sleep_reason) {
        // Awake and staying awake
        (SleepState::Awake, None) => (),
        (SleepState::Awake, Some(sleep_reason)) => {
            state.sleeping = SleepState::Sleeping((state.grid.clone(), state.brightness));
            // Slowly decrease brightness
            if dyn_sleep_mode(state) == SleepMode::Fading {
                let mut brightness = state.brightness;
                loop {
                    delay.delay_ms(100);
                    brightness = if brightness <= 5 { 0 } else { brightness - 5 };
                    set_brightness(state, brightness, matrix);
                    if brightness == 0 {
                        break;
                    }
                }
            }

            if debug_mode(state) {
                state.grid = display_sleep_reason(sleep_reason);
                fill_grid_pixels(state, matrix);
            } else {
                // Turn LED controller off to save power
                led_enable.set_low().unwrap();
            }

            // TODO: Set up SLEEP# pin as interrupt and wfi
            //cortex_m::asm::wfi();
        }
        // Already sleeping and new sleep reason => just keep sleeping
        (SleepState::Sleeping(_), Some(sleep_reason)) => {
            // If debug mode is enabled, then make sure the latest sleep reason is displayed
            if debug_mode(state) {
                state.grid = display_sleep_reason(sleep_reason);
                fill_grid_pixels(state, matrix);
            }
        }
        // Sleeping and need to wake up
        (SleepState::Sleeping((old_grid, old_brightness)), None) => {
            // Restore back grid before sleeping
            state.sleeping = SleepState::Awake;
            state.grid = old_grid;
            fill_grid_pixels(state, matrix);

            // Power LED controller back on
            if !debug_mode(state) {
                led_enable.set_high().unwrap();
            }

            // Slowly increase brightness
            if dyn_sleep_mode(state) == SleepMode::Fading {
                let mut brightness = 0;
                loop {
                    delay.delay_ms(100);
                    brightness = if brightness >= old_brightness - 5 {
                        old_brightness
                    } else {
                        brightness + 5
                    };
                    set_brightness(state, brightness, matrix);
                    if brightness == old_brightness {
                        break;
                    }
                }
            }
        }
    }
}
