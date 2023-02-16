use rp2040_hal::rom_data::reset_to_usb_boot;

use crate::{patterns::*, State};

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

pub enum Command {
    /// Set brightness scaling
    Brightness(u8),
    /// Display pre-programmed pattern
    Pattern(PatternVals),
    /// Reset into bootloader
    BootloaderReset,
    /// Light up a percentage of the screen
    Percentage(u8),
    /// Go to sleepe or wake up
    Sleep(bool),
    /// Start/stop animation (vertical scrolling)
    Animate(bool),
    /// Panic. Just to test what happens
    Panic,
    /// Draw black/white on the grid
    Draw([u8; DRAW_BYTES]),
    StageGreyCol(u8, [u8; HEIGHT]),
    DrawGreyColBuffer,
    _Unknown,
}

pub fn parse_command(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 4 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = buf[3];

        //let mut text: String<64> = String::new();
        //writeln!(&mut text, "Command: {command}, arg: {arg}").unwrap();
        //let _ = serial.write(text.as_bytes());

        match command {
            0x00 => Some(Command::Brightness(arg)),
            0x01 => match arg {
                0x00 => {
                    if count >= 5 {
                        Some(Command::Percentage(buf[4]))
                    } else {
                        None
                    }
                }
                0x01 => Some(Command::Pattern(PatternVals::Gradient)),
                0x02 => Some(Command::Pattern(PatternVals::DoubleGradient)),
                0x03 => Some(Command::Pattern(PatternVals::DisplayLotus)),
                0x04 => Some(Command::Pattern(PatternVals::ZigZag)),
                0x05 => Some(Command::Pattern(PatternVals::FullBrightness)),
                0x06 => Some(Command::Pattern(PatternVals::DisplayPanic)),
                0x07 => Some(Command::Pattern(PatternVals::DisplayLotus2)),
                _ => None,
            },
            0x02 => Some(Command::BootloaderReset),
            0x03 => Some(Command::Sleep(arg == 1)),
            0x04 => Some(Command::Animate(arg == 1)),
            0x05 => Some(Command::Panic),
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
            _ => None, //Some(Command::Unknown),
        }
    } else {
        None
    }
}

pub fn handle_command(command: &Command, state: &mut State, matrix: &mut Foo) {
    match command {
        Command::Brightness(br) => {
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
        Command::BootloaderReset => {
            //let _ = serial.write("Bootloader Reset".as_bytes());
            reset_to_usb_boot(0, 0);
        }
        Command::Sleep(_go_sleeping) => {
            // Handled elsewhere
        }
        Command::Animate(a) => state.animate = *a,
        Command::Panic => panic!("Ahhh"),
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
        _ => {}
    }
}
