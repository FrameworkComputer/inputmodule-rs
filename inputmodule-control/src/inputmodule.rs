use std::thread;
use std::time::Duration;

use chrono::Local;
use image::codecs::gif::GifDecoder;
use image::{io::Reader as ImageReader, Luma};
use image::{AnimationDecoder, DynamicImage, ImageBuffer};
use rand::prelude::*;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::b1display::{B1Pattern, Fps, PowerMode};
use crate::c1minimal::Color;
use crate::font::{convert_font, convert_symbol};
use crate::ledmatrix::{BenchmarkMode, Demo, Game, GameOfLifeStartParam, Pattern};

const FWK_MAGIC: &[u8] = &[0x32, 0xAC];
pub const FRAMEWORK_VID: u16 = 0x32AC;
pub const LED_MATRIX_PID: u16 = 0x0020;
pub const B1_LCD_PID: u16 = 0x0021;

type Brightness = u8;

// TODO: Use a shared enum with the firmware code
#[derive(Clone, Copy)]
#[repr(u8)]
enum Command {
    Brightness = 0x00,
    Pattern = 0x01,
    Bootloader = 0x02,
    Sleeping = 0x03,
    Animate = 0x04,
    Panic = 0x05,
    DisplayBwImage = 0x06,
    SendCol = 0x07,
    CommitCols = 0x08,
    _B1Reserved = 0x09,
    StartGame = 0x10,
    GameControl = 0x11,
    _GameStatus = 0x12,
    SetColor = 0x13,
    DisplayOn = 0x14,
    InvertScreen = 0x15,
    SetPixelColumn = 0x16,
    FlushFramebuffer = 0x17,
    ClearRam = 0x18,
    ScreenSaver = 0x19,
    Fps = 0x1A,
    PowerMode = 0x1B,
    AnimationPeriod = 0x1C,
    Gamma = 0x1D,
    PwmFreq = 0x1E,
    DebugMode = 0x1F,
    Version = 0x20,
    Stats = 0x21,
}

enum GameControlArg {
    _Up = 0,
    _Down = 1,
    _Left = 2,
    _Right = 3,
    Exit = 4,
    _SecondLeft = 5,
    _SecondRight = 6,
}

const WIDTH: usize = 9;
const HEIGHT: usize = 34;

const SERIAL_TIMEOUT: Duration = Duration::from_millis(20);

fn match_serialdevs(
    ports: &[SerialPortInfo],
    requested: &Option<String>,
    pid: Option<u16>,
) -> Vec<String> {
    if let Some(requested) = requested {
        for p in ports {
            if requested == &p.port_name {
                return vec![p.port_name.clone()];
            }
        }
        vec![]
    } else {
        let mut compatible_devs = vec![];
        let pids = if let Some(pid) = pid {
            vec![pid]
        } else {
            // By default accept any type
            vec![LED_MATRIX_PID, B1_LCD_PID, 0x22, 0xFF]
        };
        // Find all supported Framework devices
        for p in ports {
            if let SerialPortType::UsbPort(usbinfo) = &p.port_type {
                // macOS creates a /dev/cu.* and /dev/tty.* device.
                // The latter can only be used for reading, not writing, so we have to ignore it.
                #[cfg(target_os = "macos")]
                if !p.port_name.starts_with("/dev/tty.") {
                    continue;
                }
                if usbinfo.vid == FRAMEWORK_VID && pids.contains(&usbinfo.pid) {
                    compatible_devs.push(p.port_name.clone());
                }
            }
        }
        compatible_devs
    }
}

pub fn find_serialdevs(args: &crate::ClapCli, wait_for_device: bool) -> (Vec<String>, bool) {
    let mut serialdevs: Vec<String>;
    let mut waited = false;
    loop {
        let ports = serialport::available_ports().expect("No ports found!");
        if args.list || args.verbose {
            for p in &ports {
                match &p.port_type {
                    SerialPortType::UsbPort(usbinfo) => {
                        println!("{}", p.port_name);
                        println!("  VID     {:#06X}", usbinfo.vid);
                        println!("  PID     {:#06X}", usbinfo.pid);
                        if let Some(sn) = &usbinfo.serial_number {
                            println!("  SN      {}", sn);
                        }
                        if let Some(product) = &usbinfo.product {
                            // TODO: Seems to replace the spaces with underscore, not sure why
                            println!("  Product {}", product);
                        }
                    }
                    _ => {
                        //println!("{}", p.port_name);
                        //println!("  Unknown (PCI Port)");
                    }
                }
            }
        }
        serialdevs = match_serialdevs(
            &ports,
            &args.serial_dev,
            args.command.as_ref().map(|x| x.to_pid()),
        );
        if serialdevs.is_empty() {
            if wait_for_device {
                // Waited at least once, that means the device was not present
                // when the program started
                waited = true;

                // Try again after short wait
                thread::sleep(Duration::from_millis(100));
                continue;
            } else {
                return (vec![], waited);
            }
        } else {
            break;
        }
    }
    (serialdevs, waited)
}

/// Commands that interact with serial devices
pub fn serial_commands(args: &crate::ClapCli) {
    let (serialdevs, waited): (Vec<String>, bool) = find_serialdevs(args, args.wait_for_device);
    if serialdevs.is_empty() {
        println!("Failed to find serial device. Please manually specify with --serial-dev");
        return;
    } else if args.wait_for_device && !waited {
        println!("Device already present. No need to wait. Not executing command. Sleep 1s");
        thread::sleep(Duration::from_millis(1000));
        return;
    }

    match &args.command {
        // TODO: Handle generic commands without code deduplication
        Some(crate::Commands::LedMatrix(ledmatrix_args)) => {
            for serialdev in &serialdevs {
                if args.verbose {
                    println!("Selected serialdev: {:?}", serialdev);
                }

                if ledmatrix_args.bootloader {
                    bootloader_cmd(serialdev);
                }
                if let Some(sleeping_arg) = ledmatrix_args.sleeping {
                    sleeping_cmd(serialdev, sleeping_arg);
                }
                if let Some(brightness_arg) = ledmatrix_args.brightness {
                    brightness_cmd(serialdev, brightness_arg);
                }
                if let Some(percentage) = ledmatrix_args.percentage {
                    assert!(percentage <= 100);
                    percentage_cmd(serialdev, percentage);
                }
                if let Some(animate_arg) = ledmatrix_args.animate {
                    animate_cmd(serialdev, animate_arg);
                }
                if let Some(pattern) = ledmatrix_args.pattern {
                    pattern_cmd(serialdev, pattern);
                }
                if ledmatrix_args.all_brightnesses {
                    all_brightnesses_cmd(serialdev);
                }
                if ledmatrix_args.panic {
                    simple_cmd(serialdev, Command::Panic, &[0x00]);
                }
                if let Some(image_path) = &ledmatrix_args.image_bw {
                    display_bw_image_cmd(serialdev, image_path);
                }

                if let Some(image_path) = &ledmatrix_args.image_gray {
                    display_gray_image_cmd(serialdev, image_path);
                }

                if let Some(values) = &ledmatrix_args.eq {
                    eq_cmd(serialdev, values);
                }

                if let Some(s) = &ledmatrix_args.string {
                    show_string(serialdev, s);
                }

                if let Some(symbols) = &ledmatrix_args.symbols {
                    show_symbols(serialdev, symbols);
                }

                if let Some(game) = ledmatrix_args.start_game {
                    start_game_cmd(serialdev, game, ledmatrix_args.game_param);
                }

                if let Some(fps) = ledmatrix_args.animation_fps {
                    animation_fps_cmd(serialdev, fps);
                }

                if let Some(freq) = ledmatrix_args.pwm_freq {
                    pwm_freq_cmd(serialdev, freq);
                }
                if let Some(debug_mode) = ledmatrix_args.debug_mode {
                    debug_mode_cmd(serialdev, debug_mode);
                }
                if let Some(gamma) = ledmatrix_args.gamma {
                    gamma_cmd(serialdev, gamma);
                }

                if ledmatrix_args.stop_game {
                    simple_cmd(
                        serialdev,
                        Command::GameControl,
                        &[GameControlArg::Exit as u8],
                    );
                }
                if ledmatrix_args.version {
                    get_device_version(serialdev);
                }
                if let Some(secs) = ledmatrix_args.benchmark {
                    benchmark_cmd(
                        serialdev,
                        Duration::from_secs(secs),
                        ledmatrix_args.benchmark_mode,
                    );
                }
            }
            // Commands that block and need manual looping
            if ledmatrix_args.blinking {
                blinking_cmd(&serialdevs);
            }
            if ledmatrix_args.breathing {
                breathing_cmd(&serialdevs);
            }

            if ledmatrix_args.random_eq {
                random_eq_cmd(&serialdevs);
            }

            #[cfg(feature = "audio-visualizations")]
            if ledmatrix_args.input_eq {
                input_eq_cmd(&serialdevs);
            }

            if ledmatrix_args.clock {
                clock_cmd(&serialdevs);
            }

            if let Some(path) = &ledmatrix_args.video {
                video_cmd(&serialdevs, path, ledmatrix_args.fps);
            }

            if let Some(device) = &ledmatrix_args.camera {
                camera_cmd(&serialdevs, device.as_deref(), ledmatrix_args.fps);
            }

            if let Some(demo) = ledmatrix_args.demo {
                demo_cmd(
                    &serialdevs,
                    demo,
                    &ledmatrix_args.demo_text,
                    ledmatrix_args.fps,
                );
            }
        }
        Some(crate::Commands::B1Display(b1display_args)) => {
            for serialdev in &serialdevs {
                if args.verbose {
                    println!("Selected serialdev: {:?}", serialdev);
                }

                if b1display_args.bootloader {
                    bootloader_cmd(serialdev);
                }
                if let Some(sleeping_arg) = b1display_args.sleeping {
                    sleeping_cmd(serialdev, sleeping_arg);
                }
                if b1display_args.panic {
                    simple_cmd(serialdev, Command::Panic, &[0x00]);
                }
                if b1display_args.version {
                    get_device_version(serialdev);
                }
                if let Some(display_on) = b1display_args.display_on {
                    display_on_cmd(serialdev, display_on);
                }
                if let Some(invert_screen) = b1display_args.invert_screen {
                    invert_screen_cmd(serialdev, invert_screen);
                }
                if let Some(screensaver_on) = b1display_args.screen_saver {
                    screensaver_cmd(serialdev, screensaver_on);
                }
                if let Some(fps) = b1display_args.fps {
                    fps_cmd(serialdev, fps);
                }
                if let Some(power_mode) = b1display_args.power_mode {
                    power_mode_cmd(serialdev, power_mode);
                }
                if let Some(fps) = b1display_args.animation_fps {
                    animation_fps_cmd(serialdev, fps);
                }
                if let Some(image_path) = &b1display_args.image {
                    b1display_bw_image_cmd(serialdev, image_path);
                }
                if let Some(image_path) = &b1display_args.animated_gif {
                    gif_cmd(serialdev, image_path);
                }
                if b1display_args.clear_ram {
                    simple_cmd(serialdev, Command::ClearRam, &[0x00]);
                }
                if let Some(pattern) = b1display_args.pattern {
                    b1_display_pattern(serialdev, pattern);
                }
            }
        }
        Some(crate::Commands::C1Minimal(c1minimal_args)) => {
            for serialdev in &serialdevs {
                if args.verbose {
                    println!("Selected serialdev: {:?}", serialdev);
                }

                if c1minimal_args.bootloader {
                    bootloader_cmd(serialdev);
                }
                if let Some(sleeping_arg) = c1minimal_args.sleeping {
                    sleeping_cmd(serialdev, sleeping_arg);
                }
                if c1minimal_args.panic {
                    simple_cmd(serialdev, Command::Panic, &[0x00]);
                }
                if c1minimal_args.version {
                    get_device_version(serialdev);
                }
                if let Some(color) = c1minimal_args.set_color {
                    set_color_cmd(serialdev, color);
                }
            }
        }
        _ => {}
    }
}

fn get_device_version(serialdev: &str) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    simple_cmd_port(&mut port, Command::Version, &[]);

    let mut response: Vec<u8> = vec![0; 32];
    port.read_exact(response.as_mut_slice())
        .expect("Found no data!");

    let major = response[0];
    let minor = (response[1] & 0xF0) >> 4;
    let patch = response[1] & 0x0F;
    let pre_release = response[2] == 1;
    print!("Device Version: {major}.{minor}.{patch}");
    if pre_release {
        print!(" (Pre-Release)");
    }
    println!();
}

fn bootloader_cmd(serialdev: &str) {
    simple_cmd(serialdev, Command::Bootloader, &[0x00]);
}

fn percentage_cmd(serialdev: &str, arg: u8) {
    simple_cmd(
        serialdev,
        Command::Pattern,
        &[Pattern::Percentage as u8, arg],
    );
}

fn pattern_cmd(serialdev: &str, arg: Pattern) {
    simple_cmd(serialdev, Command::Pattern, &[arg as u8]);
}

fn start_game_cmd(serialdev: &str, game: Game, param: Option<GameOfLifeStartParam>) {
    match (game, param) {
        (Game::GameOfLife, Some(param)) => {
            simple_cmd(serialdev, Command::StartGame, &[game as u8, param as u8])
        }
        (Game::GameOfLife, None) => {
            println!("To start Game of Life, provide a --game-param");
        }
        (_, _) => simple_cmd(serialdev, Command::StartGame, &[game as u8]),
    }
}

fn simple_cmd_multiple(serialdevs: &Vec<String>, command: Command, args: &[u8]) {
    for serialdev in serialdevs {
        simple_cmd(serialdev, command, args);
    }
}

fn simple_cmd(serialdev: &str, command: Command, args: &[u8]) {
    let port_result = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open();

    match port_result {
        Ok(mut port) => simple_cmd_port(&mut port, command, args),
        Err(error) => match error.kind {
            serialport::ErrorKind::Io(std::io::ErrorKind::PermissionDenied) => panic!("Permission denied, couldn't access inputmodule serialport. Ensure that you have permission, for example using a udev rule or sudo."),
            other_error => panic!("Couldn't open port: {:?}", other_error)
        }
    };
}

fn open_serialport(serialdev: &str) -> Box<dyn SerialPort> {
    serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port")
}

fn simple_open_cmd(serialport: &mut Box<dyn SerialPort>, command: Command, args: &[u8]) {
    simple_cmd_port(serialport, command, args);
}

fn simple_cmd_port(port: &mut Box<dyn SerialPort>, command: Command, args: &[u8]) {
    let mut buffer: [u8; 64] = [0; 64];
    buffer[..2].copy_from_slice(FWK_MAGIC);
    buffer[2] = command as u8;
    buffer[3..3 + args.len()].copy_from_slice(args);
    port.write_all(&buffer[..3 + args.len()])
        .expect("Write failed!");
}

fn sleeping_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(goto_sleep) = arg {
        simple_cmd_port(&mut port, Command::Sleeping, &[u8::from(goto_sleep)]);
    } else {
        simple_cmd_port(&mut port, Command::Sleeping, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let sleeping: bool = response[0] == 1;
        println!("Currently sleeping: {sleeping}");
    }
}

fn debug_mode_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(enable_debug) = arg {
        simple_cmd_port(&mut port, Command::DebugMode, &[u8::from(enable_debug)]);
    } else {
        simple_cmd_port(&mut port, Command::DebugMode, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let debug_mode: bool = response[0] == 1;
        println!("Debug Mode enabled: {debug_mode}");
    }
}

fn gamma_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(enable) = arg {
        simple_cmd_port(&mut port, Command::Gamma, &[u8::from(enable)]);
    } else {
        simple_cmd_port(&mut port, Command::Gamma, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let gamma: bool = response[0] == 1;
        println!("Gamma correction enabled: {gamma}");
    }
}

fn brightness_cmd(serialdev: &str, arg: Option<u8>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(brightness) = arg {
        simple_cmd_port(&mut port, Command::Brightness, &[brightness]);
    } else {
        simple_cmd_port(&mut port, Command::Brightness, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let brightness: u8 = response[0];
        println!("Current brightness: {brightness}");
    }
}

fn animate_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(animate) = arg {
        simple_cmd_port(&mut port, Command::Animate, &[animate as u8]);
    } else {
        simple_cmd_port(&mut port, Command::Animate, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let animating = response[0] == 1;
        println!("Currently animating: {animating}");
    }
}

/// Stage greyscale values for a single column. Must be committed with commit_cols()
fn send_col(port: &mut Box<dyn SerialPort>, x: u8, vals: &[u8]) {
    let mut buffer: [u8; 64] = [0; 64];
    buffer[0] = x;
    buffer[1..vals.len() + 1].copy_from_slice(vals);
    simple_cmd_port(port, Command::SendCol, &buffer[0..vals.len() + 1]);
}

/// Commit the changes from sending individual cols with send_col(), displaying the matrix.
/// This makes sure that the matrix isn't partially updated.
fn commit_cols(port: &mut Box<dyn SerialPort>) {
    simple_cmd_port(port, Command::CommitCols, &[]);
}

/// Performance counters of the LED matrix firmware, see Stats in matrix_sync.rs
#[derive(Debug, Clone, Copy)]
struct DeviceStats {
    bytes_received: u32,
    cmds_parsed: u32,
    cmds_rejected: u32,
    frames_committed: u32,
    flushes: u32,
    flush_us_total: u32,
    flush_us_max: u32,
    loop_us_max: u32,
    loops: u32,
    idle_loops: u32,
    idle_us_total: u32,
    busy_us_total: u32,
    calcs: u32,
    calc_us_total: u32,
    calc_us_max: u32,
    wait_us_total: u32,
    led_power_ons: u32,
    led_off_us_total: u32,
    scale_flushes: u32,
}

/// Read one page of performance counters. None if the firmware doesn't support it.
fn get_stats_page(port: &mut Box<dyn SerialPort>, page: u8) -> Option<[u32; 8]> {
    simple_cmd_port(port, Command::Stats, &[page]);

    let mut response = [0; 32];
    port.read_exact(&mut response).ok()?;
    let mut vals = [0; 8];
    for (i, val) in vals.iter_mut().enumerate() {
        *val = u32::from_le_bytes(response[i * 4..i * 4 + 4].try_into().unwrap());
    }
    Some(vals)
}

/// Read the performance counters. None if the firmware doesn't support it.
/// Also acts as a barrier, the firmware only responds after processing all previous commands.
/// Sends three commands of 4 bytes each.
fn get_stats(port: &mut Box<dyn SerialPort>) -> Option<DeviceStats> {
    let page0 = get_stats_page(port, 0)?;
    let page1 = get_stats_page(port, 1)?;
    let page2 = get_stats_page(port, 2)?;
    Some(DeviceStats {
        bytes_received: page0[0],
        cmds_parsed: page0[1],
        cmds_rejected: page0[2],
        frames_committed: page0[3],
        flushes: page0[4],
        flush_us_total: page0[5],
        flush_us_max: page0[6],
        loop_us_max: page0[7],
        loops: page1[0],
        idle_loops: page1[1],
        idle_us_total: page1[2],
        busy_us_total: page1[3],
        calcs: page1[4],
        calc_us_total: page1[5],
        calc_us_max: page1[6],
        wait_us_total: page1[7],
        led_power_ons: page2[0],
        led_off_us_total: page2[1],
        scale_flushes: page2[2],
    })
}

/// Wake up the LED matrix and wait until it's done fading in.
/// While fading in, the firmware doesn't service USB, so sending frames would stall.
/// The port needs a long enough timeout.
fn wake_and_wait(port: &mut Box<dyn SerialPort>) {
    simple_cmd_port(port, Command::Sleeping, &[0]);
    // Only responds after the fade-in, because commands are handled in order
    simple_cmd_port(port, Command::Version, &[]);
    let mut response = [0; 32];
    port.read_exact(&mut response)
        .expect("Device didn't respond to version request");
}

/// Stream changing greyscale frames as fast as possible and report the throughput
fn benchmark_cmd(serialdev: &str, duration: Duration, mode: BenchmarkMode) {
    // Long timeout, because the firmware doesn't service USB while fading in after waking up
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(Duration::from_secs(10))
        .open()
        .expect("Failed to open port");

    // So the fade-in doesn't count towards the benchmark
    wake_and_wait(&mut port);

    // Old firmware doesn't respond to the stats command, don't wait too long
    port.set_timeout(Duration::from_millis(500)).unwrap();
    let before = get_stats(&mut port);
    port.set_timeout(Duration::from_secs(10)).unwrap();
    if before.is_none() {
        println!("Firmware doesn't support stats, only measuring host side throughput");
        // Make sure the unknown command doesn't leave anything behind
        let _ = port.clear(serialport::ClearBuffer::All);
    }

    let mut frames: u32 = 0;
    let mut cmds: u32 = 0;
    let mut bytes: u32 = 0;
    // How long each write call blocked
    let mut write_times: Vec<Duration> = vec![];
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        // Bright row moving down, so that every frame is different
        let head = frames as usize % HEIGHT;
        match mode {
            BenchmarkMode::Grey => {
                // With a fading tail
                for x in 0..WIDTH {
                    let mut vals = [0; HEIGHT];
                    for (y, val) in vals.iter_mut().enumerate() {
                        let dist = (head + HEIGHT - y) % HEIGHT;
                        *val = 255usize.saturating_sub(dist * 32) as u8;
                    }
                    let write_start = std::time::Instant::now();
                    send_col(&mut port, x as u8, &vals);
                    write_times.push(write_start.elapsed());
                    bytes += 3 + 1 + HEIGHT as u32;
                }
                let write_start = std::time::Instant::now();
                commit_cols(&mut port);
                write_times.push(write_start.elapsed());
                bytes += 3;
                cmds += WIDTH as u32 + 1;
            }
            BenchmarkMode::Bw => {
                let mut vals = [0; 39];
                for x in 0..WIDTH {
                    let i = x + head * WIDTH;
                    vals[i / 8] |= 1 << (i % 8);
                }
                let write_start = std::time::Instant::now();
                simple_cmd_port(&mut port, Command::DisplayBwImage, &vals);
                write_times.push(write_start.elapsed());
                bytes += 3 + vals.len() as u32;
                cmds += 1;
            }
            BenchmarkMode::Idle => {
                thread::sleep(duration);
                break;
            }
        }
        frames += 1;
    }

    // Wait until the device processed everything
    let after = if before.is_some() {
        get_stats(&mut port)
    } else {
        simple_cmd_port(&mut port, Command::Version, &[]);
        let mut response = [0; 32];
        port.read_exact(&mut response)
            .expect("Device didn't respond after benchmark");
        None
    };
    let elapsed = start.elapsed().as_secs_f64();

    println!("Duration:          {:.2}s", elapsed);
    println!("Frames sent:       {}", frames);
    println!(
        "Host send rate:    {:.1} FPS ({:.1} KiB/s)",
        frames as f64 / elapsed,
        bytes as f64 / elapsed / 1024.0
    );

    write_times.sort();
    let write_total: Duration = write_times.iter().sum();
    if !write_times.is_empty() {
        let percentile = |p: usize| write_times[(write_times.len() - 1) * p / 100].as_micros();
        println!(
            "Host write time:   {}us avg, {}us p50, {}us p99, {}us max, {:.0}% of total time",
            write_total.as_micros() / write_times.len() as u128,
            percentile(50),
            percentile(99),
            percentile(100),
            write_total.as_secs_f64() / elapsed * 100.0
        );
    }

    let (Some(before), Some(after)) = (before, after) else {
        return;
    };
    let delta = |f: fn(&DeviceStats) -> u32| f(&after).wrapping_sub(f(&before));
    // The second and third request of the first get_stats and the first request
    // of the last get_stats are counted too, one command of 4 bytes each
    let bytes_received = delta(|s| s.bytes_received).wrapping_sub(12);
    let cmds_parsed = delta(|s| s.cmds_parsed).wrapping_sub(3);
    let cmds_rejected = delta(|s| s.cmds_rejected);
    let frames_committed = delta(|s| s.frames_committed);
    let flushes = delta(|s| s.flushes);
    let flush_us_total = delta(|s| s.flush_us_total);

    println!("Bytes received:    {} of {} sent", bytes_received, bytes);
    println!(
        "Commands:          {} parsed, {} rejected, {} lost",
        cmds_parsed,
        cmds_rejected,
        cmds.saturating_sub(cmds_parsed + cmds_rejected)
    );
    println!(
        "Frames committed:  {} of {} ({:.1} FPS)",
        frames_committed,
        frames,
        frames_committed as f64 / elapsed
    );
    println!(
        "I2C flushes:       {} ({:.1} FPS, {:.1} per frame)",
        flushes,
        flushes as f64 / elapsed,
        flushes as f64 / frames_committed.max(1) as f64
    );
    println!(
        "I2C flush time:    {:.0}us avg, {}us max, {:.0}% of total time",
        flush_us_total as f64 / flushes.max(1) as f64,
        after.flush_us_max,
        flush_us_total as f64 / 1e6 / elapsed * 100.0
    );
    let scale_flushes = delta(|s| s.scale_flushes);
    println!(
        "Scaling flushes:   {} ({:.0}% of I2C flushes)",
        scale_flushes,
        scale_flushes as f64 / flushes.max(1) as f64 * 100.0
    );
    println!("Main loop:         {}us max", after.loop_us_max);

    let loops = delta(|s| s.loops);
    let idle_loops = delta(|s| s.idle_loops);
    let busy_loops = loops - idle_loops;
    let idle_us = delta(|s| s.idle_us_total) as f64;
    let busy_us = delta(|s| s.busy_us_total) as f64;
    let device_us = (idle_us + busy_us).max(1.0);
    println!(
        "Device busy:       {} loops with data, {:.0}us avg, {:.0}% of device time",
        busy_loops,
        busy_us / busy_loops.max(1) as f64,
        busy_us / device_us * 100.0
    );
    println!(
        "Device idle:       {} loops without data, {:.0}us avg, {:.0}% of device time",
        idle_loops,
        idle_us / idle_loops.max(1) as f64,
        idle_us / device_us * 100.0
    );

    let calcs = delta(|s| s.calcs);
    let calc_us = delta(|s| s.calc_us_total) as f64;
    println!(
        "PWM calculation:   {} times ({:.1} per command), {:.0}us avg, {}us max, {:.0}% of device time",
        calcs,
        calcs as f64 / cmds_parsed.max(1) as f64,
        calc_us / calcs.max(1) as f64,
        after.calc_us_max,
        calc_us / device_us * 100.0
    );

    let wait_us = delta(|s| s.wait_us_total) as f64;
    println!(
        "CPU asleep:        {:.0}% of device time",
        wait_us / device_us * 100.0
    );

    let led_off_us = delta(|s| s.led_off_us_total) as f64;
    println!(
        "LED controller:    off {:.0}% of device time, powered on {} times",
        led_off_us / device_us * 100.0,
        delta(|s| s.led_power_ons)
    );
}

///Increase the brightness with each pixel.
///Only 0-255 available, so it can't fill all 306 LEDs
fn all_brightnesses_cmd(serialdev: &str) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    for x in 0..WIDTH {
        let mut vals: [u8; HEIGHT] = [0; HEIGHT];

        for y in 0..HEIGHT {
            let brightness = x + WIDTH * y;
            vals[y] = if brightness > 255 { 0 } else { brightness } as u8;
        }

        send_col(&mut port, x as u8, &vals);
    }
    commit_cols(&mut port);
}

fn blinking_cmd(serialdevs: &Vec<String>) {
    let duration = Duration::from_millis(500);
    loop {
        simple_cmd_multiple(serialdevs, Command::Brightness, &[0]);
        thread::sleep(duration);
        simple_cmd_multiple(serialdevs, Command::Brightness, &[200]);
        thread::sleep(duration);
    }
}

fn breathing_cmd(serialdevs: &Vec<String>) {
    loop {
        // Go quickly from 250 to 50
        for i in 0..40 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[250 - i * 5]);
            thread::sleep(Duration::from_millis(25));
        }

        // Go slowly from 50 to 0
        for i in 0..50 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[50 - i]);
            thread::sleep(Duration::from_millis(10));
        }

        // Go slowly from 0 to 50
        for i in 0..50 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[i]);
            thread::sleep(Duration::from_millis(10));
        }

        // Go quickly from 50 to 250
        for i in 0..40 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[50 + i * 5]);
            thread::sleep(Duration::from_millis(25));
        }
    }
}

/// Display an image in black and white
/// Confirmed working with PNG and GIF.
/// Must be 9x34 in size.
/// Sends everything in a single command
fn display_bw_image_cmd(serialdev: &str, image_path: &str) {
    let mut vals: [u8; 39] = [0; 39];

    let img = ImageReader::open(image_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_luma8();
    let width = img.width();
    let height = img.height();
    assert!(width == 9);
    assert!(height == 34);
    for (x, y, pixel) in img.enumerate_pixels() {
        let brightness = pixel.0[0];
        if brightness > 0xFF / 2 {
            let i = (x as usize) + (y as usize) * WIDTH;
            vals[i / 8] |= 1 << (i % 8);
        }
    }

    simple_cmd(serialdev, Command::DisplayBwImage, &vals);
}

// Calculate pixel brightness from an RGB triple
fn pixel_to_brightness(pixel: &Luma<u8>) -> u8 {
    let brightness = pixel.0[0];
    // Poor man's scaling to make the greyscale pop better.
    // Should find a good function.
    if brightness > 200 {
        brightness
    } else if brightness > 150 {
        ((brightness as u32) * 10 / 8) as u8
    } else if brightness > 100 {
        brightness / 2
    } else if brightness > 50 {
        brightness
    } else {
        brightness * 2
    }
}

/// Display an image in greyscale
/// Sends each 1x34 column and then commits => 10 commands
fn display_gray_image_cmd(serialdev: &str, image_path: &str) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    let img = ImageReader::open(image_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_luma8();
    let width = img.width();
    let height = img.height();
    assert!(width == 9);
    assert!(height == 34);
    for x in 0..WIDTH {
        let mut vals: [u8; HEIGHT] = [0; HEIGHT];

        for y in 0..HEIGHT {
            let pixel = img.get_pixel(x as u32, y as u32);
            vals[y] = pixel_to_brightness(pixel);
        }

        send_col(&mut port, x as u8, &vals)
    }
    commit_cols(&mut port);
}

type GreyGrid = [[u8; HEIGHT]; WIDTH];

/// Make dim values dimmer, so the LEDs show more contrast
fn gamma(brightness: f32) -> u8 {
    let brightness = brightness.clamp(0.0, 1.0);
    (brightness * brightness * 255.0) as u8
}

/// Overlapping sine waves, depends only on the time, not on the frame rate
fn plasma_frame(t: f32) -> GreyGrid {
    let mut grid = [[0; HEIGHT]; WIDTH];
    // Center of the circular wave moves around
    let cx = 4.0 + (t * 0.5).sin() * 3.0;
    let cy = 17.0 + (t * 0.3).cos() * 12.0;
    for (x, col) in grid.iter_mut().enumerate() {
        for (y, val) in col.iter_mut().enumerate() {
            let (fx, fy) = (x as f32, y as f32);
            let dist = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            let v = (fx * 0.7 + t * 1.1).sin()
                + (fy * 0.23 - t * 1.7).sin()
                + (fx * 0.5 + fy * 0.15 + t * 0.9).sin()
                + (dist * 0.35 - t * 2.0).sin();
            // v is in -4..4
            *val = gamma(0.5 + 0.5 * (v * std::f32::consts::FRAC_PI_2).sin());
        }
    }
    grid
}

/// Fire simulation steps per second, independent of the frame rate
const FIRE_STEPS_PER_SEC: f32 = 30.0;

/// Fire effect like in PSX Doom: heat rises from the bottom row, cools down and drifts sideways
struct Fire {
    /// Heat from 0.0 to 1.0 after the previous and the next simulation step
    prev: [[f32; HEIGHT]; WIDTH],
    next: [[f32; HEIGHT]; WIDTH],
    steps: u64,
    rng: ThreadRng,
}

impl Fire {
    fn new() -> Self {
        Self {
            prev: [[0.0; HEIGHT]; WIDTH],
            next: [[0.0; HEIGHT]; WIDTH],
            steps: 0,
            rng: thread_rng(),
        }
    }

    fn step(&mut self) {
        self.prev = self.next;
        let heat = &mut self.next;
        // Flickering source of heat in the bottom row
        for col in heat.iter_mut() {
            col[HEIGHT - 1] = self.rng.gen_range(0.6..1.0);
        }
        // Each pixel takes the heat of the pixel below, cooled down and moved sideways.
        // Top to bottom, so every row only moves up by one per step.
        for y in 0..HEIGHT - 1 {
            for x in 0..WIDTH {
                let drift: isize = self.rng.gen_range(-1..=1);
                let to_x = (x as isize + drift).clamp(0, WIDTH as isize - 1) as usize;
                let cooling = self.rng.gen_range(0.0..0.09);
                heat[to_x][y] = (heat[x][y + 1] - cooling).max(0.0);
            }
        }
        self.steps += 1;
    }

    /// Blend between simulation steps, so it's smooth at frame rates above the step rate
    fn frame(&mut self, t: f32) -> GreyGrid {
        let pos = t * FIRE_STEPS_PER_SEC;
        while self.steps <= pos as u64 + 1 {
            self.step();
        }
        let blend = pos - (self.steps - 2) as f32;
        let mut grid = [[0; HEIGHT]; WIDTH];
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let heat = self.prev[x][y] + (self.next[x][y] - self.prev[x][y]) * blend;
                grid[x][y] = gamma(heat);
            }
        }
        grid
    }
}

/// Rows per second that the text moves up
const TEXT_ROWS_PER_SEC: f32 = 12.0;

/// Text stacked vertically, scrolling up by fractions of a pixel
struct ScrollingText {
    /// Brightness from 0.0 to 1.0 of each row of the rendered text, starting with an empty screen
    rows: Vec<[f32; WIDTH]>,
}

impl ScrollingText {
    fn new(text: &str) -> Self {
        // Start with an empty screen, so the text scrolls in from the bottom and out at the top
        let mut rows = vec![[0.0; WIDTH]; HEIGHT];
        // Same layout as show_font: 5x6 pixel characters, two columns from the left, one row apart
        for c in text.to_uppercase().chars() {
            let pixels = convert_font(c);
            for y in 0..6 {
                let mut row = [0.0; WIDTH];
                for x in 0..5 {
                    row[2 + x] = pixels[x + y * 5] as f32;
                }
                rows.push(row);
            }
            rows.push([0.0; WIDTH]);
        }
        Self { rows }
    }

    /// Blend between neighboring rows, so the text glides instead of jumping by whole pixels
    fn frame(&self, t: f32) -> GreyGrid {
        let len = self.rows.len();
        let offset = (t * TEXT_ROWS_PER_SEC) % len as f32;
        let mut grid = [[0; HEIGHT]; WIDTH];
        for y in 0..HEIGHT {
            let pos = offset + y as f32;
            let blend = pos.fract();
            let upper = &self.rows[pos as usize % len];
            let lower = &self.rows[(pos as usize + 1) % len];
            for x in 0..WIDTH {
                // Linear instead of gamma, so the total light stays the same while moving
                let brightness = upper[x] * (1.0 - blend) + lower[x] * blend;
                grid[x][y] = (brightness * 255.0) as u8;
            }
        }
        grid
    }
}

/// Open ports for streaming frames and wake up the LED matrices
fn open_stream_ports(serialdevs: &[String]) -> Vec<Box<dyn SerialPort>> {
    serialdevs
        .iter()
        .map(|serialdev| {
            // Long timeout, for waiting until the LED matrix is done fading in
            let mut port = serialport::new(serialdev, 115_200)
                .timeout(Duration::from_secs(10))
                .open()
                .expect("Failed to open port");
            wake_and_wait(&mut port);
            port
        })
        .collect()
}

/// Send a greyscale frame to every port
fn send_grey_frame(ports: &mut [Box<dyn SerialPort>], grid: &GreyGrid) {
    for port in ports {
        for (x, col) in grid.iter().enumerate() {
            send_col(port, x as u8, col);
        }
        commit_cols(port);
    }
}

/// Prints the achieved frame rate once per second
struct FpsReporter {
    /// When the first frame of the current second was sent
    start: Option<std::time::Instant>,
    /// Frames sent since then
    frames: u64,
}

impl FpsReporter {
    fn new() -> Self {
        Self {
            start: None,
            frames: 0,
        }
    }

    fn frame_sent(&mut self) {
        // Start at the first frame, so startup time, like opening a camera, doesn't count
        let Some(start) = self.start else {
            self.start = Some(std::time::Instant::now());
            return;
        };
        self.frames += 1;
        if start.elapsed() >= Duration::from_secs(1) {
            let rate = self.frames as f64 / start.elapsed().as_secs_f64();
            print!("\r{:.1} FPS ", rate);
            let _ = std::io::Write::flush(&mut std::io::stdout());
            self.start = Some(std::time::Instant::now());
            self.frames = 0;
        }
    }
}

/// Frame rate of the first video stream, using ffprobe
fn video_frame_rate(path: &str) -> Option<f64> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0"])
        .args(["-show_entries", "stream=avg_frame_rate"])
        .args(["-of", "default=noprint_wrappers=1:nokey=1", path])
        .output()
        .ok()?;
    // For example "30000/1001"
    let rate = String::from_utf8(output.stdout).ok()?;
    let (num, den) = rate.trim().split_once('/')?;
    let rate = num.parse::<f64>().ok()? / den.parse::<f64>().ok()?;
    (rate.is_finite() && rate > 0.0).then_some(rate)
}

/// Whether ffmpeg can be run. Prints a message if not
fn ffmpeg_available() -> bool {
    let available = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok();
    if !available {
        println!("ffmpeg not found. Please install it and make sure it's in PATH");
    }
    available
}

/// Play a video in greyscale at its own frame rate, or resampled to fps
fn video_cmd(serialdevs: &[String], path: &str, fps: Option<u32>) {
    if !ffmpeg_available() {
        return;
    }
    let frame_rate = match fps {
        Some(fps) => fps as f64,
        None => video_frame_rate(path).unwrap_or_else(|| {
            println!("Couldn't get the video's frame rate with ffprobe, using 30 FPS");
            30.0
        }),
    };
    ffmpeg_stream(serialdevs, &["-i", path], fps, Some(frame_rate));
}

/// Name of the first DirectShow video device, like "Laptop Webcam Module (2nd Gen)"
#[cfg(target_os = "windows")]
fn first_dshow_camera() -> Option<String> {
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .output()
        .ok()?;
    // Devices are listed in stderr, like: [in#0 @ 0000020c351e1b40] "Laptop Webcam Module (2nd Gen)" (video)
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter(|line| line.ends_with("(video)"))
        .find_map(|line| line.split('"').nth(1).map(String::from))
}

/// Show a live view from the webcam in greyscale.
/// Uses the first camera by default, or the device name or path.
fn camera_cmd(serialdevs: &[String], device: Option<&str>, fps: Option<u32>) {
    if !ffmpeg_available() {
        return;
    }

    #[cfg(target_os = "windows")]
    let input = {
        let Some(name) = device.map(String::from).or_else(first_dshow_camera) else {
            println!("No camera found");
            return;
        };
        [
            "-f".to_string(),
            "dshow".into(),
            "-i".into(),
            format!("video={name}"),
        ]
    };
    #[cfg(target_os = "linux")]
    let input = [
        "-f".to_string(),
        "v4l2".into(),
        "-i".into(),
        device.unwrap_or("/dev/video0").into(),
    ];
    #[cfg(target_os = "macos")]
    let input = [
        "-f".to_string(),
        "avfoundation".into(),
        "-i".into(),
        device.unwrap_or("0").into(),
    ];

    let input: Vec<&str> = input.iter().map(String::as_str).collect();
    // Frames come in live, no need to pace them
    ffmpeg_stream(serialdevs, &input, fps, None);
}

/// Stream greyscale frames from ffmpeg to the LED matrices until the input ends.
/// ffmpeg decodes, scales and crops the input, which is given as ffmpeg arguments.
/// If frame_rate is set, frames are sent at that rate, otherwise as soon as they are decoded.
fn ffmpeg_stream(serialdevs: &[String], input: &[&str], fps: Option<u32>, frame_rate: Option<f64>) {
    // Before starting ffmpeg, so a camera doesn't have to buffer frames while the matrix wakes up
    let mut ports = open_stream_ports(serialdevs);

    // Scale to the height of the matrix and crop the middle, like the Python version.
    // Videos narrower than the matrix get padded.
    let mut filter = format!(
        "scale=-2:{HEIGHT}:flags=area,crop=w=min({WIDTH}\\,iw):h={HEIGHT},pad={WIDTH}:{HEIGHT}:(ow-iw)/2:0,format=gray"
    );
    if let Some(fps) = fps {
        filter = format!("fps={fps},{filter}");
    }
    let mut ffmpeg = std::process::Command::new("ffmpeg")
        .args(["-loglevel", "error", "-nostdin"])
        .args(input)
        .args(["-vf", &filter, "-f", "rawvideo", "-"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to run ffmpeg");
    let mut video = ffmpeg.stdout.take().unwrap();

    let mut reporter = FpsReporter::new();
    // One byte per pixel, row by row
    let mut pixels = [0; WIDTH * HEIGHT];
    let mut start = None;
    let mut frames: u64 = 0;
    while std::io::Read::read_exact(&mut video, &mut pixels).is_ok() {
        if let Some(frame_rate) = frame_rate {
            // ffmpeg decodes faster than real time, so wait until the frame is due.
            // Start the clock at the first frame, so ffmpeg's startup time doesn't count.
            let start = *start.get_or_insert_with(std::time::Instant::now);
            let due = start + Duration::from_secs_f64(frames as f64 / frame_rate);
            if let Some(wait) = due.checked_duration_since(std::time::Instant::now()) {
                thread::sleep(wait);
            }
            frames += 1;
        }

        let mut grid = [[0; HEIGHT]; WIDTH];
        for (x, col) in grid.iter_mut().enumerate() {
            for (y, val) in col.iter_mut().enumerate() {
                // Video brightness is gamma encoded, LED brightness is linear
                *val = gamma(pixels[x + y * WIDTH] as f32 / 255.0);
            }
        }
        send_grey_frame(&mut ports, &grid);
        reporter.frame_sent();
    }
    let _ = ffmpeg.wait();
    println!();
}

/// Run a greyscale demo forever, as fast as possible or limited to fps
fn demo_cmd(serialdevs: &[String], demo: Demo, text: &str, fps: Option<u32>) {
    let mut ports = open_stream_ports(serialdevs);

    let mut fire = Fire::new();
    let scrolling_text = ScrollingText::new(text);

    let start = std::time::Instant::now();
    let mut frames: u64 = 0;
    let mut reporter = FpsReporter::new();
    loop {
        let t = start.elapsed().as_secs_f32();
        let grid = match demo {
            Demo::Plasma => plasma_frame(t),
            Demo::Fire => fire.frame(t),
            Demo::Text => scrolling_text.frame(t),
        };
        send_grey_frame(&mut ports, &grid);
        frames += 1;
        reporter.frame_sent();

        if let Some(fps) = fps {
            let next_frame = start + Duration::from_secs_f64(frames as f64 / fps as f64);
            if let Some(wait) = next_frame.checked_duration_since(std::time::Instant::now()) {
                thread::sleep(wait);
            }
        }
    }
}

/// Display an equlizer looking animation with random values.
fn random_eq_cmd(serialdevs: &Vec<String>) {
    loop {
        // Lower values more likely, makes it look nicer
        //weights = [i*i for i in range(33, 0, -1)]
        let population: Vec<u8> = (1..34).collect();
        let mut rng = thread_rng();
        let vals = population
            .choose_multiple_weighted(&mut rng, 9, |item| (34 - item) ^ 2)
            .unwrap()
            .copied()
            .collect::<Vec<_>>();
        for serialdev in serialdevs {
            eq_cmd(serialdev, vals.as_slice());
        }
        thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(feature = "audio-visualizations")]
/// The data-type for storing analyzer results
#[derive(Debug, Clone)]
pub struct AnalyzerResult {
    spectrum: vis_core::analyzer::Spectrum<Vec<f32>>,
    volume: f32,
    beat: f32,
}

#[cfg(feature = "audio-visualizations")]
// Equalizer-like animation that expands as volume goes up and retracts as it goes down
fn input_eq_cmd(serialdevs: &Vec<String>) {
    // Example from https://github.com/Rahix/visualizer2/blob/canon/README.md

    // Initialize the logger.  Take a look at the sources if you want to customize
    // the logger.
    vis_core::default_log();

    // Load the default config source.  More about config later on.  You can also
    // do this manually if you have special requirements.
    vis_core::default_config();

    // Initialize some analyzer-tools.  These will be moved into the analyzer closure
    // later on.
    let mut analyzer = vis_core::analyzer::FourierBuilder::new()
        .length(512)
        .window(vis_core::analyzer::window::nuttall)
        .plan();

    let spectrum = vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets()], 0.0, 1.0);

    let mut frames = vis_core::Visualizer::new(
        AnalyzerResult {
            spectrum,
            volume: 0.0,
            beat: 0.0,
        },
        // This closure is the "analyzer".  It will be executed in a loop to always
        // have the latest data available.
        move |info, samples| {
            analyzer.analyze(samples);

            info.spectrum.fill_from(&analyzer.average());
            info.volume = samples.volume(0.3) * 400.0;
            info.beat = info.spectrum.slice(50.0, 100.0).max() * 0.01;
            info
        },
    )
    // Build the frame iterator which is the base of your loop later on
    .frames();

    for frame in frames.iter() {
        // This is just a primitive example, your vis core belongs here

        frame.info(|info| {
            let sampled_volume = info.volume;
            let limited_volume = sampled_volume.min(34.0);

            let display_max_widths = [10.0, 14.0, 20.0, 28.0, 34.0, 28.0, 20.0, 14.0, 10.0];

            let volumes_to_display = display_max_widths
                .iter()
                .map(|x| {
                    let computed_width = (limited_volume / 34.0) * x;
                    let next_lowest_odd = computed_width - (computed_width % 2.0) - 1.0;
                    next_lowest_odd as u8
                })
                .collect::<Vec<_>>();

            for serialdev in serialdevs {
                eq_cmd(serialdev, volumes_to_display.as_slice())
            }
        });
        thread::sleep(Duration::from_millis(30));
    }
}

/// Display 9 values in equalizer diagram starting from the middle, going up and down
/// TODO: Implement a commandline parameter for this
fn eq_cmd(serialdev: &str, vals: &[u8]) {
    assert!(vals.len() <= WIDTH);
    let mut matrix: [[Brightness; 34]; 9] = [[0; 34]; 9];

    for (col, val) in vals[..9].iter().enumerate() {
        let row: usize = 34 / 2;
        let above: usize = (*val as usize) / 2;
        let below = (*val as usize) - above;

        for i in 0..above {
            matrix[col][row + i] = 0xFF; // Set this LED to full brightness
        }
        for i in 0..below {
            matrix[col][row - 1 - i] = 0xFF; // Set this LED to full brightness
        }
    }

    render_matrix(serialdev, &matrix);
}

/// Show a black/white matrix
/// Send everything in a single command
fn render_matrix(serialdev: &str, matrix: &[[u8; 34]; 9]) {
    // One bit for each LED, on or off
    // 39 = ceil(34 * 9 / 8)
    let mut vals: [u8; 39] = [0x00; 39];

    for x in 0..9 {
        for y in 0..34 {
            let i = x + 9 * y;
            if matrix[x][y] == 0xFF {
                vals[i / 8] |= 1 << (i % 8);
            }
        }
    }

    simple_cmd(serialdev, Command::DisplayBwImage, &vals);
}

/// Render the current time and display.
/// Loops forever, updating every second
fn clock_cmd(serialdevs: &Vec<String>) {
    loop {
        let date = Local::now();
        let current_time = date.format("%H:%M").to_string();
        println!("Current Time = {current_time}");

        for serialdev in serialdevs {
            show_string(serialdev, &current_time);
        }
        thread::sleep(Duration::from_millis(1000));
    }
}

/// Render a string with up to five letters
fn show_string(serialdev: &str, s: &str) {
    let items: Vec<Vec<u8>> = s.chars().take(5).map(convert_font).collect();
    show_font(serialdev, &items);
}

/// Render up to five 5x6 pixel font items
fn show_font(serialdev: &str, font_items: &[Vec<u8>]) {
    let mut vals: [u8; 39] = [0x00; 39];

    for (digit_i, digit_pixels) in font_items.iter().enumerate() {
        let offset = digit_i * 7;
        for pixel_x in 0..5 {
            for pixel_y in 0..6 {
                let pixel_value = digit_pixels[pixel_x + pixel_y * 5];
                let i = (2 + pixel_x) + (9 * (pixel_y + offset));
                if pixel_value == 1 {
                    vals[i / 8] |= 1 << (i % 8);
                }
            }
        }
    }

    simple_cmd(serialdev, Command::DisplayBwImage, &vals);
}

/// Render a list of up to five symbols
/// Can use letters/numbers or symbol names, like 'sun', ':)'
fn show_symbols(serialdev: &str, symbols: &Vec<String>) {
    println!("Symbols: {symbols:?}");
    let font_items: Vec<Vec<u8>> = symbols.iter().map(|x| convert_symbol(x)).collect();
    show_font(serialdev, &font_items);
}

fn display_on_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(display_on) = arg {
        simple_cmd_port(&mut port, Command::DisplayOn, &[display_on as u8]);
    } else {
        simple_cmd_port(&mut port, Command::DisplayOn, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let on = response[0] == 1;
        println!("Currently on: {on}");
    }
}

fn invert_screen_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(invert_on) = arg {
        simple_cmd_port(&mut port, Command::InvertScreen, &[invert_on as u8]);
    } else {
        simple_cmd_port(&mut port, Command::InvertScreen, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let inverted = response[0] == 1;
        println!("Currently inverted: {inverted}");
    }
}

fn screensaver_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(display_on) = arg {
        simple_cmd_port(&mut port, Command::ScreenSaver, &[display_on as u8]);
    } else {
        simple_cmd_port(&mut port, Command::ScreenSaver, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let on = response[0] == 1;
        println!("Currently on: {on}");
    }
}

fn fps_cmd(serialdev: &str, arg: Option<Fps>) {
    const HIGH_FPS_MASK: u8 = 0b00010000;
    const LOW_FPS_MASK: u8 = 0b00000111;
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    simple_cmd_port(&mut port, Command::Fps, &[]);
    let mut response: Vec<u8> = vec![0; 32];
    port.read_exact(response.as_mut_slice())
        .expect("Found no data!");
    let current_fps = response[0];

    if let Some(fps) = arg {
        let power_mode = match fps {
            Fps::Sixteen | Fps::ThirtyTwo => PowerMode::High,
            _ => PowerMode::Low,
        };
        let fps_bits = match fps {
            Fps::Quarter => current_fps & !LOW_FPS_MASK,
            Fps::Half => (current_fps & !LOW_FPS_MASK) | 0b001,
            Fps::One => (current_fps & !LOW_FPS_MASK) | 0b010,
            Fps::Two => (current_fps & !LOW_FPS_MASK) | 0b011,
            Fps::Four => (current_fps & !LOW_FPS_MASK) | 0b100,
            Fps::Eight => (current_fps & !LOW_FPS_MASK) | 0b101,
            Fps::Sixteen => current_fps & !HIGH_FPS_MASK,
            Fps::ThirtyTwo => (current_fps & !HIGH_FPS_MASK) | 0b00010000,
        };
        set_power_mode(&mut port, power_mode);
        simple_cmd_port(&mut port, Command::Fps, &[fps_bits]);
    } else {
        simple_cmd_port(&mut port, Command::PowerMode, &[]);
        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");
        let high = response[0] == 1;

        let fps = if high {
            if current_fps & HIGH_FPS_MASK == 0 {
                16.0
            } else {
                32.0
            }
        } else {
            let current_fps = current_fps & LOW_FPS_MASK;
            if current_fps == 0 {
                0.25
            } else if current_fps == 1 {
                0.5
            } else {
                (1 << (current_fps - 2)) as f32
            }
        };

        println!("Current FPS: {fps}");
    }
}

fn power_mode_cmd(serialdev: &str, arg: Option<PowerMode>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(mode) = arg {
        set_power_mode(&mut port, mode);
    } else {
        simple_cmd_port(&mut port, Command::PowerMode, &[]);
        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");
        let high = response[0] == 1;

        if high {
            println!("Current Power Mode: High");
        } else {
            println!("Current Power Mode: Low");
        }
    }
}

fn set_power_mode(port: &mut Box<dyn SerialPort>, mode: PowerMode) {
    match mode {
        PowerMode::Low => simple_cmd_port(port, Command::PowerMode, &[0]),
        PowerMode::High => simple_cmd_port(port, Command::PowerMode, &[1]),
    }
}

fn animation_fps_cmd(serialdev: &str, arg: Option<u16>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(fps) = arg {
        const MS: u16 = 1000;
        if fps < MS {
            // It would need to set the animation period lower than 1ms
            println!("Unable to set FPS over 1000");
            return;
        }
        let period = (MS / fps).to_le_bytes();
        simple_cmd_port(&mut port, Command::AnimationPeriod, &[period[0], period[1]]);
    } else {
        simple_cmd_port(&mut port, Command::AnimationPeriod, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let period = u16::from_le_bytes([response[0], response[1]]);
        println!("Animation Frequency: {}ms / {}Hz", period, 1_000 / period);
    }
}

fn pwm_freq_cmd(serialdev: &str, arg: Option<u16>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(freq) = arg {
        let hz = match freq {
            29000 => 0,
            3600 => 1,
            1800 => 2,
            900 => 3,
            _ => panic!("Invalid frequency"),
        };
        simple_cmd_port(&mut port, Command::PwmFreq, &[hz]);
    } else {
        simple_cmd_port(&mut port, Command::PwmFreq, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let hz = match response[0] {
            0 => 29000,
            1 => 3600,
            2 => 1800,
            3 => 900,
            _ => panic!("Invalid frequency"),
        };
        println!("Animation Frequency: {}Hz", hz);
    }
}

fn set_color_cmd(serialdev: &str, color: Color) {
    let args = match color {
        Color::White => &[0xFF, 0xFF, 0xFF],
        Color::Black => &[0x00, 0x00, 0x00],
        Color::Red => &[0xFF, 0x00, 0x00],
        Color::Green => &[0x00, 0xFF, 0x00],
        Color::Blue => &[0x00, 0x00, 0xFF],
        Color::Yellow => &[0xFF, 0xFF, 0x00],
        Color::Cyan => &[0x00, 0xFF, 0xFF],
        Color::Purple => &[0xFF, 0x00, 0xFF],
    };
    simple_cmd(serialdev, Command::SetColor, args);
}

fn gif_cmd(serialdev: &str, image_path: &str) {
    let mut serialport = open_serialport(serialdev);

    loop {
        let img = std::fs::File::open(image_path).unwrap();
        let gif = GifDecoder::new(img).unwrap();
        let frames = gif.into_frames();
        for frame in frames {
            let frame = frame.unwrap();
            //let delay = frame.delay();
            //println!("  Delay: {:?}", Duration::from(delay));
            let frame_img = frame.into_buffer();
            let frame_img = DynamicImage::from(frame_img);
            let frame_img = frame_img.resize(300, 400, image::imageops::FilterType::Gaussian);
            let frame_img = frame_img.into_luma8();
            display_img(&mut serialport, &frame_img);
            // Not delaying any further. Current transmission delay is big enough
            //thread::sleep(delay.into());
        }
    }
}

/// Display an image in black and white
/// Confirmed working with PNG and GIF.
/// Must be 300x400 in size.
/// Sends one 400px column in a single commands and a flush at the end
fn generic_img_cmd(serialdev: &str, image_path: &str) {
    let mut serialport = open_serialport(serialdev);
    let img = ImageReader::open(image_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_luma8();
    display_img(&mut serialport, &img);
}

fn b1display_bw_image_cmd(serialdev: &str, image_path: &str) {
    generic_img_cmd(serialdev, image_path);
}

fn display_img(serialport: &mut Box<dyn SerialPort>, img: &ImageBuffer<Luma<u8>, Vec<u8>>) {
    let width = img.width();
    let height = img.height();
    assert!(width == 300);
    assert!(height == 400);

    let (brightest, darkest) = img
        .pixels()
        .fold((0xFF, 0x00), |(brightest, darkest), pixel| {
            let br = pixel.0[0];
            let brightest = if br > brightest { br } else { brightest };
            let darkest = if br < darkest { br } else { darkest };
            (brightest, darkest)
        });
    let bright_diff = brightest - darkest;
    // Anything brighter than 90% between darkest and brightest counts as white
    // Just a heuristic. Don't use greyscale images! Use black and white instead
    let threshold = darkest + (bright_diff / 10) * 9;

    for x in 0..300 {
        let mut vals: [u8; 2 + 50] = [0; 2 + 50];
        let column = (x as u16).to_le_bytes();
        vals[0] = column[0];
        vals[1] = column[1];

        let mut byte: u8 = 0;
        for y in 0..400usize {
            let pixel = img.get_pixel(x, y as u32);
            let brightness = pixel.0[0];
            let black = brightness < threshold;

            let bit = y % 8;
            if bit == 0 {
                byte = 0;
            }
            if black {
                byte |= 1 << bit;
            }
            if bit == 7 {
                vals[2 + y / 8] = byte;
            }
        }

        simple_open_cmd(serialport, Command::SetPixelColumn, &vals);
    }

    simple_open_cmd(serialport, Command::FlushFramebuffer, &[]);
}

fn b1_display_color(serialdev: &str, black: bool) {
    let mut serialport = open_serialport(serialdev);
    for x in 0..300 {
        let byte = if black { 0xFF } else { 0x00 };
        let mut vals: [u8; 2 + 50] = [byte; 2 + 50];
        let column = (x as u16).to_le_bytes();
        vals[0] = column[0];
        vals[1] = column[1];
        simple_open_cmd(&mut serialport, Command::SetPixelColumn, &vals);
    }
    simple_open_cmd(&mut serialport, Command::FlushFramebuffer, &[]);
}

fn b1_display_pattern(serialdev: &str, pattern: B1Pattern) {
    match pattern {
        B1Pattern::Black => b1_display_color(serialdev, true),
        B1Pattern::White => b1_display_color(serialdev, false),
    }
}
