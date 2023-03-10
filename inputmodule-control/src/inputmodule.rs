use std::thread;
use std::time::Duration;

use chrono::Local;
use image::{io::Reader as ImageReader, Luma};
use rand::prelude::*;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::c1minimal::Color;
use crate::font::{convert_font, convert_symbol};
use crate::ledmatrix::{Game, GameOfLifeStartParam, Pattern};

const FWK_MAGIC: &[u8] = &[0x32, 0xAC];
const FRAMEWORK_VID: u16 = 0x32AC;
const LED_MATRIX_PID: u16 = 0x0020;
const B1_LCD_PID: u16 = 0x0021;

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
    Version = 0x20,
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

fn match_serialdevs(ports: &[SerialPortInfo], requested: &Option<String>) -> Vec<String> {
    if let Some(requested) = requested {
        for p in ports {
            if requested == &p.port_name {
                return vec![p.port_name.clone()];
            }
        }
        vec![]
    } else {
        let mut compatible_devs = vec![];
        // Find all supported Framework devices
        for p in ports {
            if let SerialPortType::UsbPort(usbinfo) = &p.port_type {
                if usbinfo.vid == FRAMEWORK_VID
                    && [LED_MATRIX_PID, B1_LCD_PID].contains(&usbinfo.pid)
                {
                    compatible_devs.push(p.port_name.clone());
                }
            }
        }
        compatible_devs
    }
}

pub fn find_serialdevs(args: &crate::ClapCli, wait_for_device: bool) -> Vec<String> {
    let mut serialdevs: Vec<String>;
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
        serialdevs = match &args.command {
            // TODO: Must be the correct device type
            Some(crate::Commands::LedMatrix(ledmatrix_args)) => {
                match_serialdevs(&ports, &ledmatrix_args.serial_dev)
            }
            Some(crate::Commands::B1Display(ledmatrix_args)) => {
                match_serialdevs(&ports, &ledmatrix_args.serial_dev)
            }
            Some(crate::Commands::C1Minimal(c1minimal_args)) => {
                match_serialdevs(&ports, &c1minimal_args.serial_dev)
            }
            None => vec![],
        };
        if serialdevs.is_empty() {
            if wait_for_device {
                // Try again after short wait
                thread::sleep(Duration::from_millis(100));
                continue;
            } else {
                return Vec::new();
            }
        } else {
            break;
        }
    }
    serialdevs
}

/// Commands that interact with serial devices
pub fn serial_commands(args: &crate::ClapCli) {
    let serialdevs: Vec<String> = find_serialdevs(args, args.wait_for_device);
    if serialdevs.is_empty() {
        println!("Failed to find serial devivce. Please manually specify with --serial-dev");
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

            if ledmatrix_args.clock {
                clock_cmd(&serialdevs);
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
                if let Some(image_path) = &b1display_args.image_bw {
                    b1display_bw_image_cmd(serialdev, image_path);
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
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    simple_cmd_port(&mut port, command, args);
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
        for i in 0..10 {
            thread::sleep(Duration::from_millis(30));
            simple_cmd_multiple(serialdevs, Command::Brightness, &[250 - i * 20]);
        }

        // Go slowly from 50 to 0
        for i in 0..10 {
            thread::sleep(Duration::from_millis(60));
            simple_cmd_multiple(serialdevs, Command::Brightness, &[50 - i * 5]);
        }

        // Go slowly from 0 to 50
        for i in 0..10 {
            thread::sleep(Duration::from_millis(60));
            simple_cmd_multiple(serialdevs, Command::Brightness, &[i * 5]);
        }

        // Go quickly from 50 to 250
        for i in 0..10 {
            thread::sleep(Duration::from_millis(30));
            simple_cmd_multiple(serialdevs, Command::Brightness, &[50 + i * 20]);
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

/// Display 9 values in equalizer diagram starting from the middle, going up and down
/// TODO: Implement a commandline parameter for this
fn eq_cmd(serialdev: &str, vals: &[u8]) {
    assert!(vals.len() <= WIDTH);
    let mut matrix: [[u8; 34]; 9] = [[0; 34]; 9];

    for (col, val) in vals[..9].iter().enumerate() {
        let row: usize = 34 / 2;
        let above: usize = (*val as usize) / 2;
        let below = (*val as usize) - above;

        for i in 0..above {
            matrix[col][row + i] = 0xFF;
        }
        for i in 0..below {
            matrix[col][row - 1 - i] = 0xFF;
        }
    }

    render_matrix(serialdev, &matrix);
}

/// Show a black/white matrix
/// Send everything in a single command
fn render_matrix(serialdev: &str, matrix: &[[u8; 34]; 9]) {
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

/// Display an image in black and white
/// Confirmed working with PNG and GIF.
/// Must be 300x400 in size.
/// Sends one 400px column in a single commands and a flush at the end
fn b1display_bw_image_cmd(serialdev: &str, image_path: &str) {
    let mut serialport = open_serialport(serialdev);
    let img = ImageReader::open(image_path)
        .unwrap()
        .decode()
        .unwrap()
        .to_luma8();
    let width = img.width();
    let height = img.height();
    assert!(width == 300);
    assert!(height == 400);

    for x in 0..300 {
        let mut vals: [u8; 2 + 50] = [0; 2 + 50];
        let column = (x as u16).to_le_bytes();
        vals[0] = column[0];
        vals[1] = column[1];

        let mut byte: u8 = 0;
        for y in 0..400usize {
            let pixel = img.get_pixel(x, y as u32);
            let brightness = pixel.0[0];
            let black = brightness < 0xFF / 2;

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

        simple_open_cmd(&mut serialport, Command::SetPixelColumn, &vals);
    }

    simple_open_cmd(&mut serialport, Command::FlushFramebuffer, &[]);
}
