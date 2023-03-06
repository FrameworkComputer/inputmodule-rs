use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
pub enum Color {
    White,
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Purple,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct C1MinimalSubcommand {
    /// Set sleep status or get, if no value provided
    #[arg(long)]
    pub sleeping: Option<Option<bool>>,

    /// Jump to the bootloader
    #[arg(long)]
    pub bootloader: bool,

    /// Crash the firmware (TESTING ONLY!)
    #[arg(long)]
    pub panic: bool,

    /// Serial device, like /dev/ttyACM0 or COM0
    #[arg(long)]
    pub serial_dev: Option<String>,

    /// Get the device version
    #[arg(short, long)]
    pub version: bool,

    /// Set color
    // TODO: Allow getting current state
    #[arg(long)]
    #[clap(value_enum)]
    pub set_color: Option<Color>,
}
