use clap::Parser;

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
#[repr(u8)]
pub enum Pattern {
    Percentage = 0,
    Gradient = 1,
    DoubleGradient = 2,
    LotusSideways = 3,
    Zigzag = 4,
    AllOn = 5,
    Panic = 6,
    LotusTopDown = 7,
    //AllBrightnesses
}

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
#[repr(u8)]
pub enum Game {
    Snake = 0,
    Pong = 1,
    Tetris = 2,
    GameOfLife = 3,
}

/// LED Matrix
#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct LedMatrixSubcommand {
    /// Set LED max brightness percentage or get, if no value provided
    #[arg(long)]
    pub brightness: Option<Option<u8>>,

    /// Set sleep status or get, if no value provided
    #[arg(long)]
    pub sleeping: Option<Option<bool>>,

    /// Jump to the bootloader
    #[arg(long)]
    pub bootloader: bool,

    /// Display a percentage (0-100)
    #[arg(long)]
    pub percentage: Option<u8>,

    /// Start/stop animation
    #[arg(long)]
    pub animate: Option<Option<bool>>,

    /// Display a pattern
    #[arg(long)]
    #[clap(value_enum)]
    pub pattern: Option<Pattern>,

    /// Show every brightness, one per pixel
    #[arg(long)]
    pub all_brightnesses: bool,

    /// Blink the current pattern once a second
    #[arg(long)]
    pub blinking: bool,

    /// Breathing brightness of the current pattern
    #[arg(long)]
    pub breathing: bool,

    /// Display black&white image (9x34px)
    #[arg(long)]
    pub image_bw: Option<String>,

    /// Display grayscale image
    #[arg(long)]
    pub image_gray: Option<String>,

    /// Random EQ
    #[arg(long)]
    pub random_eq: bool,

    /// EQ with custom values
    #[arg(long, num_args(9))]
    pub eq: Option<Vec<u8>>,

    /// Clock
    #[arg(long)]
    pub clock: bool,

    /// Display a string (max 5 chars)
    #[arg(long)]
    pub string: Option<String>,

    /// Display a string (max 5 symbols)
    #[arg(long, num_args(0..6))]
    pub symbols: Option<Vec<String>>,

    /// Start a game
    #[arg(long)]
    #[clap(value_enum)]
    pub start_game: Option<Game>,

    /// Crash the firmware (TESTING ONLY!)
    #[arg(long)]
    pub panic: bool,

    /// Serial device, like /dev/ttyACM0 or COM0
    #[arg(long)]
    pub serial_dev: Option<String>,

    /// Get the device version
    #[arg(short, long)]
    pub version: bool,
}
