use rp2040_hal::rom_data::reset_to_usb_boot;

#[cfg(feature = "b1display")]
use crate::graphics::*;
#[cfg(feature = "b1display")]
use core::fmt::{Debug, Write};
#[cfg(feature = "b1display")]
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, Point},
    primitives::Rectangle,
};
#[cfg(feature = "b1display")]
use heapless::String;

#[cfg(feature = "ledmatrix")]
use crate::matrix::*;
#[cfg(feature = "ledmatrix")]
use crate::patterns::*;
use crate::serialnum::{device_release, is_pre_release};

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
    Version,
    _Unknown,
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
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(feature = "b1display")]
pub fn parse_module_command(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 4 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = buf[3];

        match command {
            0x09 => {
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
            }
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(not(any(feature = "ledmatrix", feature = "b1display")))]
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
            return Some(response);
        }
        _ => None,
    }
}

#[cfg(feature = "ledmatrix")]
pub fn handle_command(command: &Command, state: &mut State, matrix: &mut Foo) -> Option<[u8; 32]> {
    match command {
        Command::GetBrightness => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.brightness;
            return Some(response);
        }
        Command::SetBrightness(br) => {
            //let _ = serial.write("Brightness".as_bytes());
            state.brightness = *br;
            matrix
                .set_scaling(state.brightness)
                .expect("failed to set scaling");
        }
        Command::Percentage(p) => {
            //let p = if count >= 5 { buf[4] } else { 100 };
            state.grid = percentage(*p as u16);
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
        }
        Command::SetAnimate(a) => state.animate = *a,
        Command::GetAnimate => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.animate as u8;
            return Some(response);
        }
        Command::Draw(vals) => state.grid = draw(vals),
        Command::StageGreyCol(col, vals) => {
            draw_grey_col(&mut state.col_buffer, *col, vals);
        }
        Command::DrawGreyColBuffer => {
            // Copy the staging buffer to the real grid and display it
            state.grid = state.col_buffer.clone();
            // Zero the old staging buffer, just for good measure.
            state.col_buffer = percentage(0);
        }
        // TODO: Move to handle_generic_command
        Command::IsSleeping => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.sleeping {
                SleepState::Sleeping(_) => 1,
                SleepState::Awake => 0,
            };
            return Some(response);
        }
        // TODO: Make it return something
        _ => return handle_generic_command(command),
    }
    None
}

#[cfg(feature = "b1display")]
pub fn handle_command<D>(command: &Command, disp: &mut D, logo_rect: Rectangle) -> Option<[u8; 32]>
where
    D: DrawTarget<Color = Rgb565>,
    <D as DrawTarget>::Error: Debug,
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
                Point::new(0, LOGO_OFFSET + logo_rect.size.height as i32),
            )
            .unwrap();

            draw_text(
                disp,
                text,
                Point::new(0, LOGO_OFFSET + logo_rect.size.height as i32),
            )
            .unwrap();
            None
        }
        _ => return handle_generic_command(command),
    }
}
