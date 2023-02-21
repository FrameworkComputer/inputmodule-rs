mod inputmodule;

use clap::{Parser, Subcommand};

use crate::inputmodule::{serial_commands, B1DisplaySubcommand, LedMatrixSubcommand};

#[derive(Subcommand, Debug)]
enum Commands {
    LedMatrix(LedMatrixSubcommand),
    B1Display(B1DisplaySubcommand),
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

    /// VID (Vendor ID) in hex digits
    #[arg(long)]
    vid: Option<String>,

    /// PID (Product ID) in hex digits
    #[arg(long)]
    pid: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = ClapCli::parse_from(args);

    match args.command {
        Some(Commands::B1Display(_)) => serial_commands(&args),
        Some(Commands::LedMatrix(_)) => serial_commands(&args),
        None => panic!("Not allowed"),
    }
}
