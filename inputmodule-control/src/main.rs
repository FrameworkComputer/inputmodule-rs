#![allow(clippy::needless_range_loop)]
#![allow(clippy::single_match)]
mod b1display;
mod c1minimal;
mod font;
mod inputmodule;
mod ledmatrix;
mod commands;

use clap::{Parser};
use inputmodule::find_serialdevs;

use crate::inputmodule::serial_commands;

use crate::commands::ClapCli;

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
