//! Firmware API - Commands
use num::FromPrimitive;
use rp2040_hal::rom_data::reset_to_usb_boot;

use crate::serialnum::{device_release, is_pre_release};

#[cfg(feature = "b1display")]
use crate::graphics::*;
#[cfg(feature = "b1display")]
use core::fmt::{Debug, Write};
#[cfg(feature = "b1display")]
use cortex_m::delay::Delay;
#[cfg(feature = "b1display")]
use embedded_graphics::Pixel;
#[cfg(feature = "b1display")]
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    primitives::Rectangle,
};
#[cfg(feature = "b1display")]
use embedded_hal::blocking::spi;
#[cfg(feature = "b1display")]
use embedded_hal::digital::v2::OutputPin;
#[cfg(feature = "b1display")]
use heapless::String;
#[cfg(feature = "b1display")]
use st7306::{FpsConfig, PowerMode, ST7306};

#[cfg(feature = "ledmatrix")]
use crate::games::pong;
#[cfg(feature = "ledmatrix")]
use crate::games::snake;
#[cfg(feature = "ledmatrix")]
use crate::matrix::*;
#[cfg(feature = "ledmatrix")]
use crate::patterns::*;
#[cfg(feature = "ledmatrix")]
use is31fl3741::PwmFreq;

#[cfg(feature = "c1minimal")]
use smart_leds::{SmartLedsWrite, RGB8};

#[repr(u8)]
#[derive(num_derive::FromPrimitive)]
/// All available commands
pub enum CommandVals {
    Brightness = 0x00,
    Pattern = 0x01,
    BootloaderReset = 0x02,
    Sleep = 0x03,
    Animate = 0x04,
    Panic = 0x05,
    Draw = 0x06,
    StageGreyCol = 0x07,
    DrawGreyColBuffer = 0x08,
    SetText = 0x09,
    StartGame = 0x10,
    GameControl = 0x11,
    GameStatus = 0x12,
    SetColor = 0x13,
    DisplayOn = 0x14,
    InvertScreen = 0x15,
    SetPixelColumn = 0x16,
    FlushFramebuffer = 0x17,
    ClearRam = 0x18,
    ScreenSaver = 0x19,
    SetFps = 0x1A,
    SetPowerMode = 0x1B,
    AnimationPeriod = 0x1C,
    PwmFreq = 0x1E,
    DebugMode = 0x1F,
    Version = 0x20,
}

#[derive(num_derive::FromPrimitive)]
pub enum PatternVals {
    Percentage = 0x00,
    Gradient = 0x01,
    DoubleGradient = 0x02,
    DisplayLotus = 0x03,
    ZigZag = 0x04,
    FullBrightness = 0x05,
    DisplayPanic = 0x06,
    DisplayLotus2 = 0x07,
}

pub enum Game {
    Snake,
    Pong,
    Tetris,
    GameOfLife(GameOfLifeStartParam),
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum GameVal {
    Snake = 0,
    Pong = 1,
    Tetris = 2,
    GameOfLife = 3,
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum GameControlArg {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Exit = 4,
    SecondLeft = 5,
    SecondRight = 6,
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum GameOfLifeStartParam {
    CurrentMatrix = 0x00,
    Pattern1 = 0x01,
    Blinker = 0x02,
    Toad = 0x03,
    Beacon = 0x04,
    Glider = 0x05,
    BeaconToadBlinker = 0x06,
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum DisplayMode {
    /// Low Power Mode
    Lpm = 0x00,
    /// High Power Mode
    Hpm = 0x01,
}

#[cfg(feature = "ledmatrix")]
#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum PwmFreqArg {
    /// 29kHz
    P29k = 0x00,
    /// 3.6kHz
    P3k6 = 0x01,
    /// 1.8kHz
    P1k8 = 0x02,
    /// 900Hz
    P900 = 0x03,
}
#[cfg(feature = "ledmatrix")]
impl From<PwmFreqArg> for PwmFreq {
    fn from(val: PwmFreqArg) -> Self {
        match val {
            PwmFreqArg::P29k => PwmFreq::P29k,
            PwmFreqArg::P3k6 => PwmFreq::P3k6,
            PwmFreqArg::P1k8 => PwmFreq::P1k8,
            PwmFreqArg::P900 => PwmFreq::P900,
        }
    }
}

// TODO: Reduce size for modules that don't require other commands
pub enum Command {
    /// Get current brightness scaling
    GetBrightness,
    /// Set brightness scaling
    SetBrightness(u8),
    /// Display pre-programmed pattern
    Pattern(PatternVals),
    /// Reset into bootloader
    BootloaderReset,
    /// Light up a percentage of the screen
    Percentage(u8),
    /// Go to sleepe or wake up
    Sleep(bool),
    IsSleeping,
    /// Start/stop animation (vertical scrolling)
    SetAnimate(bool),
    GetAnimate,
    /// Panic. Just to test what happens
    Panic,
    /// Draw black/white on the grid
    #[cfg(feature = "ledmatrix")]
    Draw([u8; DRAW_BYTES]),
    #[cfg(feature = "ledmatrix")]
    StageGreyCol(u8, [u8; HEIGHT]),
    DrawGreyColBuffer,
    #[cfg(feature = "b1display")]
    SetText(String<64>),
    StartGame(Game),
    GameControl(GameControlArg),
    GameStatus,
    Version,
    GetColor,
    #[cfg(feature = "c1minimal")]
    SetColor(RGB8),
    DisplayOn(bool),
    GetDisplayOn,
    InvertScreen(bool),
    GetInvertScreen,
    SetPixelColumn(usize, [u8; 50]),
    FlushFramebuffer,
    ClearRam,
    ScreenSaver(bool),
    GetScreenSaver,
    SetFps(u8),
    GetFps,
    SetPowerMode(u8),
    GetPowerMode,
    /// Set the animation period in milliseconds
    SetAnimationPeriod(u16),
    /// Get the animation period in milliseconds
    GetAnimationPeriod,
    #[cfg(feature = "ledmatrix")]
    SetPwmFreq(PwmFreqArg),
    GetPwmFreq,
    SetDebugMode(bool),
    GetDebugMode,
    _Unknown,
}

impl Command {
    pub fn should_wake(&self) -> bool {
        match self {
            Self::SetBrightness(_)
            | Self::Pattern(_)
            | Self::Percentage(_)
            | Self::SetAnimate(_)
            | Self::Draw(_)
            | Self::DrawGreyColBuffer
            | Self::StartGame(_)
            | Self::GameControl(_)
            | Self::DisplayOn(_)
            | Self::InvertScreen(_)
            | Self::SetPixelColumn(_, _)
            | Self::FlushFramebuffer
            | Self::ScreenSaver(_)
            | Self::SetFps(_)
            | Self::SetPowerMode(_)
            | Self::SetDebugMode(_) => true,

            #[cfg(feature = "ledmatrix")]
            Self::Draw(_) | Self::SetPwmFreq(_) => true,

            #[cfg(feature = "c1minimal")]
            Self::SetColor(_) => true,

            #[cfg(feature = "b1display")]
            Self::SetText(_) => true,

            _ => false,
        }
    }
}

#[cfg(any(feature = "c1minimal", feature = "b1display"))]
#[derive(Clone)]
pub enum SimpleSleepState {
    Awake,
    Sleeping,
}

#[cfg(feature = "c1minimal")]
pub struct C1MinimalState {
    pub sleeping: SimpleSleepState,
    pub color: RGB8,
    pub brightness: u8,
}

#[derive(Copy, Clone)]
pub struct ScreenSaverState {
    pub rightwards: i32,
    pub downwards: i32,
}

impl Default for ScreenSaverState {
    fn default() -> Self {
        Self {
            rightwards: 1,
            downwards: 1,
        }
    }
}

#[cfg(feature = "b1display")]
pub struct B1DIsplayState {
    pub sleeping: SimpleSleepState,
    pub screen_inverted: bool,
    pub screen_on: bool,
    pub screensaver: Option<ScreenSaverState>,
    pub power_mode: PowerMode,
    pub fps_config: FpsConfig,
    /// Animation period in microseconds
    pub animation_period: u64,
}

pub fn parse_command(count: usize, buf: &[u8]) -> Option<Command> {
    if let Some(command) = parse_module_command(count, buf) {
        return Some(command);
    }

    // Parse the generic commands common to all modules
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        //let mut text: String<64> = String::new();
        //writeln!(&mut text, "Command: {command}, arg: {arg}").unwrap();
        //let _ = serial.write(text.as_bytes());
        match FromPrimitive::from_u8(command) {
            Some(CommandVals::Sleep) => Some(if let Some(go_to_sleep) = arg {
                Command::Sleep(go_to_sleep == 1)
            } else {
                Command::IsSleeping
            }),
            Some(CommandVals::BootloaderReset) => Some(Command::BootloaderReset),
            Some(CommandVals::Panic) => Some(Command::Panic),
            Some(CommandVals::Version) => Some(Command::Version),
            _ => None, //Some(Command::Unknown),
        }
    } else {
        None
    }
}

#[cfg(feature = "ledmatrix")]
pub fn parse_module_command(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        match FromPrimitive::from_u8(command) {
            Some(CommandVals::Brightness) => Some(if let Some(brightness) = arg {
                Command::SetBrightness(brightness)
            } else {
                Command::GetBrightness
            }),
            Some(CommandVals::Pattern) => match arg.and_then(FromPrimitive::from_u8) {
                // TODO: Convert arg to PatternVals
                Some(PatternVals::Percentage) => {
                    if count >= 5 {
                        Some(Command::Percentage(buf[4]))
                    } else {
                        None
                    }
                }
                Some(PatternVals::Gradient) => Some(Command::Pattern(PatternVals::Gradient)),
                Some(PatternVals::DoubleGradient) => {
                    Some(Command::Pattern(PatternVals::DoubleGradient))
                }
                Some(PatternVals::DisplayLotus) => {
                    Some(Command::Pattern(PatternVals::DisplayLotus))
                }
                Some(PatternVals::ZigZag) => Some(Command::Pattern(PatternVals::ZigZag)),
                Some(PatternVals::FullBrightness) => {
                    Some(Command::Pattern(PatternVals::FullBrightness))
                }
                Some(PatternVals::DisplayPanic) => {
                    Some(Command::Pattern(PatternVals::DisplayPanic))
                }
                Some(PatternVals::DisplayLotus2) => {
                    Some(Command::Pattern(PatternVals::DisplayLotus2))
                }
                None => None,
            },
            Some(CommandVals::Animate) => Some(if let Some(run_animation) = arg {
                Command::SetAnimate(run_animation == 1)
            } else {
                Command::GetAnimate
            }),
            Some(CommandVals::Draw) => {
                if count >= 3 + DRAW_BYTES {
                    let mut bytes = [0; DRAW_BYTES];
                    bytes.clone_from_slice(&buf[3..3 + DRAW_BYTES]);
                    Some(Command::Draw(bytes))
                } else {
                    None
                }
            }
            Some(CommandVals::StageGreyCol) => {
                if count >= 3 + 1 + HEIGHT {
                    let mut bytes = [0; HEIGHT];
                    bytes.clone_from_slice(&buf[4..4 + HEIGHT]);
                    Some(Command::StageGreyCol(buf[3], bytes))
                } else {
                    None
                }
            }
            Some(CommandVals::DrawGreyColBuffer) => Some(Command::DrawGreyColBuffer),
            Some(CommandVals::StartGame) => match arg.and_then(FromPrimitive::from_u8) {
                Some(GameVal::Snake) => Some(Command::StartGame(Game::Snake)),
                Some(GameVal::Pong) => Some(Command::StartGame(Game::Pong)),
                Some(GameVal::Tetris) => None,
                Some(GameVal::GameOfLife) => {
                    if count >= 5 {
                        FromPrimitive::from_u8(buf[4])
                            .map(|x| Command::StartGame(Game::GameOfLife(x)))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Some(CommandVals::GameControl) => match arg.and_then(FromPrimitive::from_u8) {
                Some(GameControlArg::Up) => Some(Command::GameControl(GameControlArg::Up)),
                Some(GameControlArg::Down) => Some(Command::GameControl(GameControlArg::Down)),
                Some(GameControlArg::Left) => Some(Command::GameControl(GameControlArg::Left)),
                Some(GameControlArg::Right) => Some(Command::GameControl(GameControlArg::Right)),
                Some(GameControlArg::Exit) => Some(Command::GameControl(GameControlArg::Exit)),
                Some(GameControlArg::SecondLeft) => {
                    Some(Command::GameControl(GameControlArg::SecondLeft))
                }
                Some(GameControlArg::SecondRight) => {
                    Some(Command::GameControl(GameControlArg::SecondRight))
                }
                _ => None,
            },
            Some(CommandVals::GameStatus) => Some(Command::GameStatus),
            Some(CommandVals::AnimationPeriod) => {
                if count == 3 + 2 {
                    let period = u16::from_le_bytes([buf[3], buf[4]]);
                    Some(Command::SetAnimationPeriod(period))
                } else {
                    Some(Command::GetAnimationPeriod)
                }
            }
            Some(CommandVals::PwmFreq) => {
                if let Some(freq) = arg {
                    FromPrimitive::from_u8(freq).map(Command::SetPwmFreq)
                } else {
                    Some(Command::GetPwmFreq)
                }
            }
            Some(CommandVals::DebugMode) => Some(if let Some(debug_mode) = arg {
                Command::SetDebugMode(debug_mode == 1)
            } else {
                Command::GetDebugMode
            }),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(feature = "b1display")]
pub fn parse_module_command(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        match FromPrimitive::from_u8(command) {
            Some(CommandVals::SetText) => {
                if let Some(arg) = arg {
                    let available_len = count - 4;
                    let str_len = arg as usize;
                    assert!(str_len <= available_len);

                    assert!(str_len < 32);
                    let mut bytes = [0; 32];
                    bytes[..str_len].copy_from_slice(&buf[4..4 + str_len]);

                    let text_str = core::str::from_utf8(&bytes[..str_len]).unwrap();
                    let mut text: String<64> = String::new();
                    writeln!(&mut text, "{}", text_str).unwrap();

                    Some(Command::SetText(text))
                } else {
                    None
                }
            }
            Some(CommandVals::DisplayOn) => Some(if let Some(on) = arg {
                Command::DisplayOn(on == 1)
            } else {
                Command::GetDisplayOn
            }),
            Some(CommandVals::InvertScreen) => Some(if let Some(invert) = arg {
                Command::InvertScreen(invert == 1)
            } else {
                Command::GetInvertScreen
            }),
            Some(CommandVals::SetPixelColumn) => {
                //  3B for magic and command
                //  2B for column (u16)
                // 50B for 400 pixels (400/8=50)
                if count == 3 + 2 + 50 {
                    let column = u16::from_le_bytes([buf[3], buf[4]]);
                    //panic!("SetPixelColumn. Col: {}", column);
                    let mut pixels: [u8; 50] = [0; 50];
                    pixels.clone_from_slice(&buf[5..55]);
                    Some(Command::SetPixelColumn(column as usize, pixels))
                } else {
                    None
                }
            }
            Some(CommandVals::FlushFramebuffer) => Some(Command::FlushFramebuffer),
            Some(CommandVals::ClearRam) => Some(Command::ClearRam),
            Some(CommandVals::ScreenSaver) => Some(if let Some(on) = arg {
                Command::ScreenSaver(on == 1)
            } else {
                Command::GetScreenSaver
            }),
            Some(CommandVals::SetFps) => Some(if let Some(fps) = arg {
                Command::SetFps(fps)
            } else {
                Command::GetFps
            }),
            Some(CommandVals::SetPowerMode) => Some(if let Some(mode) = arg {
                Command::SetPowerMode(mode)
            } else {
                Command::GetPowerMode
            }),
            Some(CommandVals::AnimationPeriod) => {
                if count == 3 + 2 {
                    let period = u16::from_le_bytes([buf[3], buf[4]]);
                    Some(Command::SetAnimationPeriod(period))
                } else {
                    Some(Command::GetAnimationPeriod)
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(not(any(feature = "ledmatrix", feature = "b1display", feature = "c1minimal")))]
pub fn parse_module_command(_count: usize, _buf: &[u8]) -> Option<Command> {
    None
}

pub fn handle_generic_command(command: &Command) -> Option<[u8; 32]> {
    match command {
        Command::BootloaderReset => {
            //let _ = serial.write("Bootloader Reset".as_bytes());
            reset_to_usb_boot(0, 0);
            None
        }
        Command::Panic => panic!("Ahhh"),
        Command::Version => {
            let mut response: [u8; 32] = [0; 32];
            let bcd_device = device_release().to_be_bytes();
            response[0] = bcd_device[0];
            response[1] = bcd_device[1];
            response[2] = is_pre_release() as u8;
            Some(response)
        }
        _ => None,
    }
}

#[cfg(feature = "ledmatrix")]
pub fn handle_command(
    command: &Command,
    state: &mut LedmatrixState,
    matrix: &mut Foo,
    random: u8,
) -> Option<[u8; 32]> {
    use crate::games::game_of_life;

    match command {
        Command::GetBrightness => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.brightness;
            Some(response)
        }
        Command::SetBrightness(br) => {
            //let _ = serial.write("Brightness".as_bytes());
            set_brightness(state, *br, matrix);
            None
        }
        Command::Percentage(p) => {
            //let p = if count >= 5 { buf[4] } else { 100 };
            state.grid = percentage(*p as u16);
            None
        }
        Command::Pattern(pattern) => {
            //let _ = serial.write("Pattern".as_bytes());
            match pattern {
                PatternVals::Gradient => state.grid = gradient(),
                PatternVals::DoubleGradient => state.grid = double_gradient(),
                PatternVals::DisplayLotus => state.grid = display_lotus(),
                PatternVals::ZigZag => state.grid = zigzag(),
                PatternVals::FullBrightness => {
                    state.grid = percentage(100);
                    set_brightness(state, BRIGHTNESS_LEVELS, matrix);
                }
                PatternVals::DisplayPanic => state.grid = display_panic(),
                PatternVals::DisplayLotus2 => state.grid = display_lotus2(),
                _ => {}
            }
            None
        }
        Command::SetAnimate(a) => {
            state.animate = *a;
            None
        }
        Command::GetAnimate => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.animate as u8;
            Some(response)
        }
        Command::Draw(vals) => {
            state.grid = draw(vals);
            None
        }
        Command::StageGreyCol(col, vals) => {
            draw_grey_col(&mut state.col_buffer, *col, vals);
            None
        }
        Command::DrawGreyColBuffer => {
            // Copy the staging buffer to the real grid and display it
            state.grid = state.col_buffer.clone();
            // Zero the old staging buffer, just for good measure.
            state.col_buffer = percentage(0);
            None
        }
        // TODO: Move to handle_generic_command
        Command::IsSleeping => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.sleeping {
                SleepState::Sleeping(_) => 1,
                SleepState::Awake => 0,
            };
            Some(response)
        }
        Command::StartGame(game) => {
            match game {
                Game::Snake => snake::start_game(state, random),
                Game::Pong => pong::start_game(state, random),
                Game::Tetris => {}
                Game::GameOfLife(param) => game_of_life::start_game(state, random, *param),
            }
            None
        }
        Command::GameControl(arg) => {
            match state.game {
                Some(GameState::Snake(_)) => snake::handle_control(state, arg),
                Some(GameState::Pong(_)) => pong::handle_control(state, arg),
                Some(GameState::GameOfLife(_)) => game_of_life::handle_control(state, arg),
                _ => {}
            }
            None
        }
        Command::GameStatus => None,
        Command::SetAnimationPeriod(period) => {
            state.animation_period = (*period as u64) * 1_000;
            None
        }
        Command::GetAnimationPeriod => {
            // TODO: Doesn't seem to work when the FPS is 16 or higher
            let mut response: [u8; 32] = [0; 32];
            let period_ms = state.animation_period / 1_000;
            response[0..2].copy_from_slice(&(period_ms as u16).to_le_bytes());
            Some(response)
        }
        Command::SetPwmFreq(arg) => {
            state.pwm_freq = *arg;
            matrix.device.set_pwm_freq(state.pwm_freq.into()).unwrap();
            None
        }
        Command::GetPwmFreq => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.pwm_freq as u8;
            Some(response)
        }
        Command::SetDebugMode(arg) => {
            state.debug_mode = *arg;
            None
        }
        Command::GetDebugMode => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.debug_mode as u8;
            Some(response)
        }
        _ => handle_generic_command(command),
    }
}

#[cfg(feature = "b1display")]
pub fn handle_command<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize>(
    command: &Command,
    state: &mut B1DIsplayState,
    logo_rect: Rectangle,
    disp: &mut ST7306<SPI, DC, CS, RST, COLS, ROWS>,
    delay: &mut Delay,
) -> Option<[u8; 32]>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    <SPI as spi::Write<u8>>::Error: Debug,
{
    match command {
        // TODO: Move to handle_generic_command
        Command::IsSleeping => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.sleeping {
                SimpleSleepState::Sleeping => 1,
                SimpleSleepState::Awake => 0,
            };
            Some(response)
        }
        Command::Panic => panic!("Ahhh"),
        Command::SetText(text) => {
            // Turn screensaver off, when drawing something
            state.screensaver = None;

            clear_text(
                disp,
                Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y + logo_rect.size.height as i32),
                Rgb565::WHITE,
            )
            .unwrap();

            draw_text(
                disp,
                text,
                Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y + logo_rect.size.height as i32),
            )
            .unwrap();
            disp.flush().unwrap();
            None
        }
        Command::DisplayOn(on) => {
            state.screen_on = *on;
            disp.on_off(*on).unwrap();
            None
        }
        Command::GetDisplayOn => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.screen_on as u8;
            Some(response)
        }
        Command::InvertScreen(invert) => {
            state.screen_inverted = *invert;
            disp.invert_screen(state.screen_inverted).unwrap();
            None
        }
        Command::GetInvertScreen => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.screen_inverted as u8;
            Some(response)
        }
        Command::SetPixelColumn(column, pixel_bytes) => {
            // Turn screensaver off, when drawing something
            state.screensaver = None;

            let mut pixels: [bool; 400] = [false; 400];
            for (i, byte) in pixel_bytes.iter().enumerate() {
                pixels[8 * i] = byte & 0b00000001 != 0;
                pixels[8 * i + 1] = byte & 0b00000010 != 0;
                pixels[8 * i + 2] = byte & 0b00000100 != 0;
                pixels[8 * i + 3] = byte & 0b00001000 != 0;
                pixels[8 * i + 4] = byte & 0b00010000 != 0;
                pixels[8 * i + 5] = byte & 0b00100000 != 0;
                pixels[8 * i + 6] = byte & 0b01000000 != 0;
                pixels[8 * i + 7] = byte & 0b10000000 != 0;
            }
            disp.draw_pixels(
                pixels.iter().enumerate().map(|(y, black)| {
                    Pixel(
                        Point::new(*column as i32, y as i32),
                        if *black { Rgb565::BLACK } else { Rgb565::WHITE },
                    )
                }),
                false,
            )
            .unwrap();
            None
        }
        Command::FlushFramebuffer => {
            disp.flush().unwrap();
            None
        }
        Command::ClearRam => {
            // Turn screensaver off, when drawing something
            state.screensaver = None;

            disp.clear_ram().unwrap();
            None
        }
        Command::ScreenSaver(on) => {
            state.screensaver = match (*on, state.screensaver) {
                (true, Some(x)) => Some(x),
                (true, None) => Some(ScreenSaverState::default()),
                (false, Some(_)) => None,
                (false, None) => None,
            };
            None
        }
        Command::GetScreenSaver => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.screensaver.is_some() as u8;
            Some(response)
        }
        Command::SetFps(fps) => {
            if let Some(fps_config) = FpsConfig::from_u8(*fps) {
                state.fps_config = fps_config;
                disp.set_fps(state.fps_config).unwrap();
                // TODO: Need to reinit the display
            }
            None
        }
        Command::GetFps => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.fps_config.as_u8();
            Some(response)
        }
        Command::SetPowerMode(mode) => {
            match mode {
                0 => {
                    state.power_mode = PowerMode::Lpm;
                    disp.switch_mode(delay, state.power_mode).unwrap();
                }
                1 => {
                    state.power_mode = PowerMode::Hpm;
                    disp.switch_mode(delay, state.power_mode).unwrap();
                }
                _ => {}
            }
            None
        }
        Command::GetPowerMode => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.power_mode {
                PowerMode::Lpm => 0,
                PowerMode::Hpm => 1,
            };
            Some(response)
        }
        Command::SetAnimationPeriod(period) => {
            state.animation_period = (*period as u64) * 1_000;
            None
        }
        Command::GetAnimationPeriod => {
            // TODO: Doesn't seem to work when the FPS is 16 or higher
            let mut response: [u8; 32] = [0; 32];
            let period_ms = state.animation_period / 1_000;
            response[0..2].copy_from_slice(&(period_ms as u16).to_le_bytes());
            Some(response)
        }
        _ => handle_generic_command(command),
    }
}

#[cfg(feature = "c1minimal")]
pub fn handle_command(
    command: &Command,
    state: &mut C1MinimalState,
    ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ()>,
) -> Option<[u8; 32]> {
    match command {
        // TODO: Move to handle_generic_command
        Command::IsSleeping => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.sleeping {
                SimpleSleepState::Sleeping => 1,
                SimpleSleepState::Awake => 0,
            };
            Some(response)
        }
        Command::GetBrightness => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.brightness;
            Some(response)
        }
        Command::SetBrightness(br) => {
            //let _ = serial.write("Brightness".as_bytes());
            state.brightness = *br;
            ws2812
                .write(smart_leds::brightness(
                    [state.color].iter().cloned(),
                    state.brightness,
                ))
                .unwrap();
            None
        }
        Command::GetColor => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.color.r;
            response[1] = state.color.g;
            response[2] = state.color.b;
            Some(response)
        }
        Command::SetColor(color) => {
            state.color = *color;
            ws2812
                .write(smart_leds::brightness(
                    [*color].iter().cloned(),
                    state.brightness,
                ))
                .unwrap();
            None
        }
        // TODO: Make it return something
        _ => handle_generic_command(command),
    }
}

#[cfg(feature = "c1minimal")]
pub fn parse_module_command(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        match FromPrimitive::from_u8(command) {
            Some(CommandVals::Brightness) => Some(if let Some(brightness) = arg {
                Command::SetBrightness(brightness)
            } else {
                Command::GetBrightness
            }),
            Some(CommandVals::SetColor) => {
                if count >= 6 {
                    let (red, green, blue) = (buf[3], buf[4], buf[5]);
                    Some(Command::SetColor(RGB8::new(red, green, blue)))
                } else if arg.is_none() {
                    Some(Command::GetColor)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
