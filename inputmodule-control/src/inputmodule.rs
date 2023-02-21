use std::time::Duration;

use clap::Parser;
use serialport::{SerialPort, SerialPortInfo};

const EXPECTED_SERIAL_DEVICES: &[&str] = &["/dev/ttyACM0", "/dev/ttyACM1", "COM0", "COM1"];
const FWK_MAGIC: &[u8] = &[0x32, 0xAC];

const BRIGHTNESS: u8 = 0x00;
const PERCENTAGE: u8 = 0x01;
const BOOTLOADER: u8 = 0x02;
const SLEEPING: u8 = 0x03;
const ANIMATE: u8 = 0x04;
const PATTERN: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
enum Pattern {
    // Percentage = 0
    Gradient = 1,
    DoubleGradient = 2,
    LotusSideways = 3,
    Zigzag = 4,
    AllOn = 5,
    Panic = 6,
    LotusTopDown = 7,
    //AllBrightnesses
}

/// LED Matrix
#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct LedMatrixSubcommand {
    /// Set LED max brightness percentage or get, if no value provided
    #[arg(long)]
    brightness: Option<Option<u8>>,

    /// Set sleep status or get, if no value provided
    #[arg(long)]
    sleeping: Option<Option<bool>>,

    /// Jump to the bootloader
    #[arg(long)]
    bootloader: bool,

    /// Display a percentage (0-100)
    #[arg(long)]
    percentage: Option<u8>,

    /// Start/stop animation
    #[arg(long)]
    animate: Option<Option<bool>>,

    /// Display a pattern
    #[arg(long)]
    #[clap(value_enum)]
    pattern: Option<Pattern>,

    /// Serial device, like /dev/ttyACM0 or COM0
    #[arg(long)]
    serial_dev: Option<String>,
}

/// B1 Display
#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct B1DisplaySubcommand {}

fn find_serialdev(ports: &[SerialPortInfo], requested: &Option<String>) -> Option<String> {
    if let Some(requested) = requested {
        for p in ports {
            if requested == &p.port_name {
                return Some(p.port_name.clone());
            }
        }
    } else {
        // If nothing requested, fall back to a generic one
        for p in ports {
            if EXPECTED_SERIAL_DEVICES.contains(&p.port_name.as_str()) {
                return Some(p.port_name.clone());
            }
        }
    }
    None
}

/// Commands that interact with serial devices
pub fn serial_commands(args: &crate::ClapCli) {
    let ports = serialport::available_ports().expect("No ports found!");
    if args.list || args.verbose {
        for p in &ports {
            println!("{}", p.port_name);
        }
    }
    let serialdev = match &args.command {
        Some(crate::Commands::LedMatrix(ledmatrix_args)) => {
            find_serialdev(&ports, &ledmatrix_args.serial_dev)
        }
        _ => None,
    };
    let serialdev = if let Some(serialdev) = serialdev {
        if args.verbose {
            println!("Selected serialdev: {serialdev:?}");
        }
        serialdev
    } else {
        println!("Failed to find serial devivce. Please manually specify with --serial-dev");
        return;
    };

    match &args.command {
        Some(crate::Commands::LedMatrix(ledmatrix_args)) => {
            if ledmatrix_args.bootloader {
                bootloader_cmd(&serialdev);
            }
            if let Some(sleeping_arg) = ledmatrix_args.sleeping {
                sleeping_cmd(&serialdev, sleeping_arg);
            }
            if let Some(brightness_arg) = ledmatrix_args.brightness {
                brightness_cmd(&serialdev, brightness_arg);
            }
            if let Some(percentage) = ledmatrix_args.percentage {
                assert!(percentage <= 100);
                percentage_cmd(&serialdev, percentage);
            }
            if let Some(animate_arg) = ledmatrix_args.animate {
                animate_cmd(&serialdev, animate_arg);
            }
            if let Some(pattern) = ledmatrix_args.pattern {
                pattern_cmd(&serialdev, pattern);
            }
        }
        Some(crate::Commands::B1Display(_b1display_args)) => {}
        _ => {}
    }
}

fn bootloader_cmd(serialdev: &str) {
    simple_cmd(serialdev, BOOTLOADER, &[0x00]);
}

fn percentage_cmd(serialdev: &str, arg: u8) {
    simple_cmd(serialdev, PERCENTAGE, &[arg]);
}

fn pattern_cmd(serialdev: &str, arg: Pattern) {
    simple_cmd(serialdev, PATTERN, &[arg as u8]);
}

fn simple_cmd(serialdev: &str, command: u8, args: &[u8]) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    simple_cmd_port(&mut port, command, args);
}

fn simple_cmd_port(port: &mut Box<dyn SerialPort>, command: u8, args: &[u8]) {
    let mut buffer: [u8; 64] = [0; 64];
    buffer[..2].copy_from_slice(FWK_MAGIC);
    buffer[2] = command;
    buffer[3..3 + args.len()].copy_from_slice(args);
    port.write_all(&buffer[..3 + args.len()])
        .expect("Write failed!");
}

fn sleeping_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    if let Some(goto_sleep) = arg {
        simple_cmd_port(&mut port, SLEEPING, &[if goto_sleep { 1 } else { 0 }]);
    } else {
        simple_cmd_port(&mut port, SLEEPING, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let sleeping: bool = response[0] == 1;
        println!("Currently sleeping: {sleeping}");
    }
}

fn brightness_cmd(serialdev: &str, arg: Option<u8>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    if let Some(brightness) = arg {
        simple_cmd_port(&mut port, BRIGHTNESS, &[brightness]);
    } else {
        simple_cmd_port(&mut port, BRIGHTNESS, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let brightness: u8 = response[0];
        println!("Current brightness: {brightness}");
    }
}

fn animate_cmd(serialdev: &str, arg: Option<bool>) {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    if let Some(animate) = arg {
        simple_cmd_port(&mut port, ANIMATE, &[animate as u8]);
    } else {
        simple_cmd_port(&mut port, ANIMATE, &[]);

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let animating = response[0] == 1;
        println!("Currently animating: {animating}");
    }
}
