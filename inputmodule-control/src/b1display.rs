use clap::Parser;

#[derive(Copy, Clone, Debug, PartialEq, clap::ValueEnum)]
pub enum B1Pattern {
    White,
    Black,
    //Checkerboard,
}

/// B1 Display
#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct B1DisplaySubcommand {
    /// Set sleep status or get, if no value provided
    #[arg(long)]
    pub sleeping: Option<Option<bool>>,

    /// Jump to the bootloader
    #[arg(long)]
    pub bootloader: bool,

    /// Crash the firmware (TESTING ONLY!)
    #[arg(long)]
    pub panic: bool,

    /// Get the device version
    #[arg(short, long)]
    pub version: bool,

    /// Turn display on/off
    // TODO: Allow getting current state
    #[arg(long)]
    pub display_on: Option<Option<bool>>,

    /// Display a simple pattern
    #[arg(long)]
    #[clap(value_enum)]
    pub pattern: Option<B1Pattern>,

    /// Invert screen on/off
    #[arg(long)]
    pub invert_screen: Option<Option<bool>>,

    /// Display black&white image (300x400px)
    #[arg(long)]
    pub image_bw: Option<String>,

    /// Clear display RAM
    #[arg(long)]
    pub clear_ram: bool,
}
