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

#[derive(Copy, Clone, Debug, PartialEq, clap::ValueEnum)]
pub enum GameOfLifeStartParam {
    CurrentMatrix = 0x00,
    Pattern1 = 0x01,
    Blinker = 0x02,
    Toad = 0x03,
    Beacon = 0x04,
    Glider = 0x05,
    BeaconToadBlinker = 0x06,
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

    /// Display EQ of microphone input
    #[cfg(feature = "audio-visualizations")]
    #[arg(long)]
    pub input_eq: bool,

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

    /// Paramater for starting the game. Required for some games
    #[arg(long)]
    #[clap(value_enum)]
    pub game_param: Option<GameOfLifeStartParam>,

    /// Stop the currently running game
    #[arg(long)]
    #[clap(value_enum)]
    pub stop_game: bool,

    /// Set/get animation FPS
    #[arg(long)]
    pub animation_fps: Option<Option<u16>>,

    /// Set/get PWM Frequency in Hz
    #[arg(long)]
    #[clap(value_enum)]
    pub pwm_freq: Option<Option<u16>>,

    /// Set debug mode or get current mode, if no value provided
    #[arg(long)]
    pub debug_mode: Option<Option<bool>>,

    /// Crash the firmware (TESTING ONLY!)
    #[arg(long)]
    pub panic: bool,

    /// Get the device version
    #[arg(short, long)]
    pub version: bool,

    // Flash storage commands
    /// Save current display to flash pattern slot (0-7)
    #[arg(long)]
    pub save_pattern: Option<u8>,

    /// Load pattern from flash slot (0-7)
    #[arg(long)]
    pub load_pattern: Option<u8>,

    /// Delete pattern from flash slot (0-7)
    #[arg(long)]
    pub delete_pattern: Option<u8>,

    /// List all saved patterns
    #[arg(long)]
    pub list_patterns: bool,

    /// Save current settings to flash
    #[arg(long)]
    pub save_config: bool,

    /// Get stored config from flash
    #[arg(long)]
    pub get_config: bool,

    /// Reset config to defaults
    #[arg(long)]
    pub reset_config: bool,

    /// Set default brightness (0-255) and save to flash
    #[arg(long)]
    pub set_default_brightness: Option<u8>,

    /// Set sleep timeout in seconds (0 = disabled) and save to flash
    #[arg(long)]
    pub set_sleep_timeout: Option<u16>,

    /// Set startup pattern slot (0-7, or 255 for none)
    #[arg(long)]
    pub set_startup_pattern: Option<u8>,

    /// Enable/disable startup animation and save to flash
    #[arg(long)]
    pub set_startup_animation: Option<bool>,
}
