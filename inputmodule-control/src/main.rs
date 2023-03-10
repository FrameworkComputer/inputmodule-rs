#![allow(clippy::needless_range_loop)]
#![allow(clippy::single_match)]
mod b1display;
mod c1minimal;
mod font;
mod inputmodule;
mod ledmatrix;

use clap::{Parser, Subcommand};
use inputmodule::find_serialdevs;

use crate::b1display::B1DisplaySubcommand;
use crate::c1minimal::C1MinimalSubcommand;
use crate::inputmodule::{serial_commands, B1_LCD_PID, LED_MATRIX_PID};
use crate::ledmatrix::LedMatrixSubcommand;

#[derive(Subcommand, Debug)]
enum Commands {
    LedMatrix(LedMatrixSubcommand),
    B1Display(B1DisplaySubcommand),
    C1Minimal(C1MinimalSubcommand),
}

impl Commands {
    pub fn to_pid(&self) -> u16 {
        match self {
            Self::LedMatrix(_) => LED_MATRIX_PID,
            Self::B1Display(_) => B1_LCD_PID,
            Self::C1Minimal(_) => 0x22,
        }
    }
}

/// RAW HID and VIA commandline for QMK devices
#[derive(Parser, Debug)]
#[command(version, arg_required_else_help = true)]
pub struct ClapCli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// List connected HID devices
    #[arg(short, long)]
    list: bool,

    /// Verbose outputs to the console
    #[arg(short, long)]
    verbose: bool,

    /// Serial device, like /dev/ttyACM0 or COM0
    #[arg(long)]
    pub serial_dev: Option<String>,

    /// Retry connecting to the device until it works
    #[arg(long)]
    wait_for_device: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    match args.command {
        Some(_) => serial_commands(&args),
        None => {
            if args.list {
                find_serialdevs(&args, false);
            }
        }
    }
}
