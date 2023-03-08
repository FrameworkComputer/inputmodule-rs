use rp2040_hal::rom_data::reset_to_usb_boot;

use crate::serialnum::{device_release, is_pre_release};

#[cfg(feature = "b1display")]
use crate::graphics::*;
#[cfg(feature = "b1display")]
use core::fmt::{Debug, Write};
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
use st7306_lcd::{instruction::Instruction, ST7306};

#[cfg(feature = "ledmatrix")]
use crate::games::pong;
#[cfg(feature = "ledmatrix")]
use crate::games::snake;
#[cfg(feature = "ledmatrix")]
use crate::matrix::*;
#[cfg(feature = "ledmatrix")]
use crate::patterns::*;

#[cfg(feature = "c1minimal")]
use smart_leds::{SmartLedsWrite, RGB8};

pub enum _CommandVals {
    _Brightness = 0x00,
    _Pattern = 0x01,
    _BootloaderReset = 0x02,
    _Sleep = 0x03,
    _Animate = 0x04,
    _Panic = 0x05,
    _Draw = 0x06,
    _StageGreyCol = 0x07,
    _DrawGreyColBuffer = 0x08,
    SetText = 0x09,
    StartGame = 0x10,
    GameControl = 0x11,
    GameStatus = 0x12,
    SetColor = 0x13,
    DisplayOn = 0x14,
    InvertScreen = 0x15,
    SetPixelColumn = 0x16,
    FlushFramebuffer = 0x17,
    Version = 0x20,
}

pub enum PatternVals {
    _Percentage = 0x00,
    Gradient,
    DoubleGradient,
    DisplayLotus,
    ZigZag,
    FullBrightness,
    DisplayPanic,
    DisplayLotus2,
}

pub enum Game {
    Snake,
    Pong,
    GameOfLife,
}

#[derive(Clone)]
pub enum GameControlArg {
    Up,
    Down,
    Left,
    Right,
    Exit,
    SecondLeft,
    SecondRight,
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
    InvertScreen(bool),
    SetPixelColumn(usize, [u8; 50]),
    FlushFramebuffer,
    _Unknown,
}

#[cfg(feature = "c1minimal")]
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

        match command {
            0x02 => Some(Command::BootloaderReset),
            0x03 => Some(if let Some(go_to_sleep) = arg {
                Command::Sleep(go_to_sleep == 1)
            } else {
                Command::IsSleeping
            }),
            0x05 => Some(Command::Panic),
            0x20 => Some(Command::Version),
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

        match command {
            0x00 => Some(if let Some(brightness) = arg {
                Command::SetBrightness(brightness)
            } else {
                Command::GetBrightness
            }),
            0x01 => match arg {
                Some(0x00) => {
                    if count >= 5 {
                        Some(Command::Percentage(buf[4]))
                    } else {
                        None
                    }
                }
                Some(0x01) => Some(Command::Pattern(PatternVals::Gradient)),
                Some(0x02) => Some(Command::Pattern(PatternVals::DoubleGradient)),
                Some(0x03) => Some(Command::Pattern(PatternVals::DisplayLotus)),
                Some(0x04) => Some(Command::Pattern(PatternVals::ZigZag)),
                Some(0x05) => Some(Command::Pattern(PatternVals::FullBrightness)),
                Some(0x06) => Some(Command::Pattern(PatternVals::DisplayPanic)),
                Some(0x07) => Some(Command::Pattern(PatternVals::DisplayLotus2)),
                Some(_) => None,
                None => None,
            },
            0x04 => Some(if let Some(run_animation) = arg {
                Command::SetAnimate(run_animation == 1)
            } else {
                Command::GetAnimate
            }),
            0x06 => {
                if count >= 3 + DRAW_BYTES {
                    let mut bytes = [0; DRAW_BYTES];
                    bytes.clone_from_slice(&buf[3..3 + DRAW_BYTES]);
                    Some(Command::Draw(bytes))
                } else {
                    None
                }
            }
            0x07 => {
                if count >= 3 + 1 + HEIGHT {
                    let mut bytes = [0; HEIGHT];
                    bytes.clone_from_slice(&buf[4..4 + HEIGHT]);
                    Some(Command::StageGreyCol(buf[3], bytes))
                } else {
                    None
                }
            }
            0x08 => Some(Command::DrawGreyColBuffer),
            0x10 => match arg {
                Some(0) => Some(Command::StartGame(Game::Snake)),
                Some(1) => Some(Command::StartGame(Game::Pong)),
                // Some(2) Reserved for Tetris
                Some(3) => Some(Command::StartGame(Game::GameOfLife)),
                _ => None,
            },
            0x11 => match arg {
                Some(0) => Some(Command::GameControl(GameControlArg::Up)),
                Some(1) => Some(Command::GameControl(GameControlArg::Down)),
                Some(2) => Some(Command::GameControl(GameControlArg::Left)),
                Some(3) => Some(Command::GameControl(GameControlArg::Right)),
                Some(4) => Some(Command::GameControl(GameControlArg::Exit)),
                Some(5) => Some(Command::GameControl(GameControlArg::SecondLeft)),
                Some(6) => Some(Command::GameControl(GameControlArg::SecondRight)),
                _ => None,
            },
            0x12 => Some(Command::GameStatus),
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

        match command {
            0x09 => {
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
            0x14 => Some(Command::DisplayOn(arg == Some(1))),
            0x15 => Some(Command::InvertScreen(arg == Some(1))),
            0x16 => {
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
            0x17 => Some(Command::FlushFramebuffer),
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
        Command::Sleep(_go_sleeping) => {
            // Handled elsewhere
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
    state: &mut State,
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
            state.brightness = *br;
            matrix
                .set_scaling(state.brightness)
                .expect("failed to set scaling");
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
                    state.brightness = 0xFF;
                    matrix
                        .set_scaling(state.brightness)
                        .expect("failed to set scaling");
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
                Game::GameOfLife => game_of_life::start_game(state, random),
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
        _ => handle_generic_command(command),
    }
}

#[cfg(feature = "b1display")]
pub fn handle_command<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize>(
    command: &Command,
    logo_rect: Rectangle,
    disp: &mut ST7306<SPI, DC, CS, RST, COLS, ROWS>,
) -> Option<[u8; 32]>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    <SPI as spi::Write<u8>>::Error: Debug,
{
    match command {
        Command::BootloaderReset => {
            //let _ = serial.write("Bootloader Reset".as_bytes());
            reset_to_usb_boot(0, 0);
            None
        }
        Command::Sleep(_go_sleeping) => {
            // Handled elsewhere
            None
        }
        Command::Panic => panic!("Ahhh"),
        Command::SetText(text) => {
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
            None
        }
        Command::DisplayOn(on) => {
            disp.on_off(*on).unwrap();
            None
        }
        Command::InvertScreen(invert) => {
            if *invert {
                disp.write_command(Instruction::INVON, &[]).unwrap();
            } else {
                disp.write_command(Instruction::INVOFF, &[]).unwrap();
            }
            None
        }
        Command::SetPixelColumn(column, pixel_bytes) => {
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

        match command {
            0x00 => Some(if let Some(brightness) = arg {
                Command::SetBrightness(brightness)
            } else {
                Command::GetBrightness
            }),
            0x13 => {
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
