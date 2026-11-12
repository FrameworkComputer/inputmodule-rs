//! Shared parts of the games that run on the host and stream frames to the LED matrix:
//! drawing by area, terminal input and the frame loop.
//!
//! Positions are in LEDs, but not rounded to whole LEDs. Each LED lights up by how
//! much of it a shape covers, so things glide smoothly instead of jumping from LED
//! to LED. The higher the frame rate, the smoother it looks.

use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    KeyboardEnhancementFlags, MouseButton, MouseEventKind, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::{cursor, execute, terminal};
use rand::Rng;

use crate::inputmodule::{GreyGrid, Matrices, HEIGHT};

pub const FIELD_H: f32 = HEIGHT as f32;

/// Brightness of each LED, 1.0 is fully on. Adds up where shapes overlap.
/// Indexed [x][y], as wide as the screen, which can be several LED matrices.
pub type Light = Vec<[f32; HEIGHT]>;

/// All LEDs off, for a screen this many LEDs wide
pub fn dark(width: usize) -> Light {
    vec![[0.0; HEIGHT]; width]
}

/// Light up the LEDs by how much of them the rectangle covers
pub fn add_rect(light: &mut Light, (x0, y0): (f32, f32), (x1, y1): (f32, f32), brightness: f32) {
    let cols = (x0.floor().max(0.0) as usize)..(x1.ceil().min(light.len() as f32) as usize);
    let rows = (y0.floor().max(0.0) as usize)..(y1.ceil().min(FIELD_H) as usize);
    for x in cols {
        let cover_x = x1.min(x as f32 + 1.0) - x0.max(x as f32);
        for y in rows.clone() {
            let cover_y = y1.min(y as f32 + 1.0) - y0.max(y as f32);
            light[x][y] += brightness * cover_x * cover_y;
        }
    }
}

/// A square one LED wide, centered on x, y
pub fn add_square(light: &mut Light, x: f32, y: f32, brightness: f32) {
    add_rect(light, (x - 0.5, y - 0.5), (x + 0.5, y + 0.5), brightness);
}

/// Sparks flying apart from a point, like when the player crashes
#[derive(Clone)]
pub struct Burst {
    x: f32,
    y: f32,
    /// Velocity of each spark in LEDs per second
    sparks: Vec<(f32, f32)>,
}

impl Burst {
    pub fn new(rng: &mut impl Rng, x: f32, y: f32) -> Self {
        const SPARKS: usize = 14;
        let sparks = (0..SPARKS)
            .map(|i| {
                // Evenly around the circle, a bit random so it's different every time
                let angle =
                    i as f32 / SPARKS as f32 * std::f32::consts::TAU + rng.gen_range(0.0..0.4);
                let speed = rng.gen_range(3.0..10.0);
                (speed * angle.cos(), speed * angle.sin())
            })
            .collect();
        Self { x, y, sparks }
    }

    /// Draw it age seconds after it started, faded out after duration
    pub fn draw(&self, light: &mut Light, age: f32, duration: f32) {
        let age = age.clamp(0.0, duration);
        let fade = 1.0 - age / duration;
        for &(vel_x, vel_y) in &self.sparks {
            add_square(
                light,
                self.x + vel_x * age,
                self.y + vel_y * age,
                0.8 * fade,
            );
        }
    }
}

/// Convert to a frame for the LED matrix, scaling everything by dim
pub fn to_grid(light: &Light, dim: f32) -> GreyGrid {
    light
        .iter()
        .map(|col| {
            // Linear instead of gamma, so the total light stays the same while moving
            col.map(|l| (l.clamp(0.0, 1.0) * dim * 255.0) as u8)
        })
        .collect()
}

/// What the player wants, collected from terminal events
#[derive(Default)]
pub struct Input {
    /// Where the player should go, from the mouse or arrow key presses
    pub target: Option<f32>,
    /// Arrow keys currently held, only known if the terminal reports key releases
    pub held_left: bool,
    pub held_right: bool,
    /// Space, Enter, up arrow or a click since the last update
    pub action: bool,
    /// Every key pressed since the last update, including key repeats, in order
    pub presses: Vec<KeyCode>,
    pub quit: bool,
}

impl Input {
    /// Move x the way the player steers: at key_speed while an arrow key is held,
    /// otherwise towards the mouse or the last key press, at most at max_speed
    pub fn steer(&mut self, x: f32, dt: f32, key_speed: f32, max_speed: f32) -> f32 {
        let held = self.held_right as i32 - self.held_left as i32;
        if held != 0 {
            self.target = None;
            x + held as f32 * key_speed * dt
        } else if let Some(target) = self.target {
            let max_step = max_speed * dt;
            x + (target - x).clamp(-max_step, max_step)
        } else {
            x
        }
    }
}

/// A game that runs on the host, see play()
pub trait HostGame {
    /// Where the player is, arrow key presses move on from there
    fn player_x(&self) -> f32;
    /// Advance to now, dt seconds after the last update
    fn update(&mut self, input: &mut Input, now: f32, dt: f32);
    fn render(&self, now: f32) -> GreyGrid;
    /// One line for the terminal, like the score
    fn status(&self, fps: f64) -> String;
}

/// Puts the terminal into raw mode with mouse reporting, and back when dropped,
/// even if the game panics
struct RawTerminal {
    /// Whether the terminal reports key releases, so keys can be held
    key_releases: bool,
}

impl RawTerminal {
    fn new() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), EnableMouseCapture, cursor::Hide)?;
        // Windows always reports releases, other terminals only with the kitty keyboard protocol
        let enhanced = matches!(terminal::supports_keyboard_enhancement(), Ok(true));
        if enhanced {
            execute!(
                stdout(),
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
            )?;
        }
        Ok(Self {
            key_releases: enhanced || cfg!(windows),
        })
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        if self.key_releases && !cfg!(windows) {
            let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
        }
        let _ = execute!(stdout(), DisableMouseCapture, cursor::Show);
        let _ = terminal::disable_raw_mode();
        println!();
    }
}

/// Apply all pending terminal events to the input, for a screen width LEDs wide
fn read_input(
    input: &mut Input,
    key_releases: bool,
    player_x: f32,
    width: f32,
) -> std::io::Result<()> {
    while event::poll(Duration::ZERO)? {
        match event::read()? {
            Event::Key(key) => {
                let pressed = key.kind != KeyEventKind::Release;
                if pressed {
                    input.presses.push(key.code);
                }
                let ctrl_c =
                    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
                let dir = match key.code {
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => -1.0,
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => 1.0,
                    _ => 0.0,
                };
                if dir != 0.0 {
                    if key_releases {
                        if dir < 0.0 {
                            input.held_left = pressed;
                        } else {
                            input.held_right = pressed;
                        }
                    } else if pressed {
                        // Without releases, every press and key repeat moves by one LED
                        let from = input.target.unwrap_or(player_x);
                        input.target = Some(from + dir);
                    }
                } else if pressed {
                    match key.code {
                        KeyCode::Char(' ') | KeyCode::Up | KeyCode::Enter => input.action = true,
                        KeyCode::Char('q') | KeyCode::Esc => input.quit = true,
                        _ if ctrl_c => input.quit = true,
                        _ => {}
                    }
                }
            }
            Event::Mouse(mouse) => {
                // The width of the terminal maps to the width of the screen
                let (cols, _) = terminal::size()?;
                let x = (mouse.column as f32 + 0.5) / cols.max(1) as f32 * width;
                match mouse.kind {
                    MouseEventKind::Moved | MouseEventKind::Drag(_) => input.target = Some(x),
                    MouseEventKind::Down(MouseButton::Left) => input.action = true,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Play until the player quits, as fast as possible or limited to fps.
/// help explains the controls. new_game makes the game for a screen that many LEDs wide.
/// With extend, that's all LED matrices side by side.
pub fn play<G: HostGame>(
    serialdevs: &[String],
    extend: bool,
    fps: Option<u32>,
    help: &str,
    new_game: impl FnOnce(usize) -> G,
) {
    let mut matrices = Matrices::open(serialdevs, extend);
    let width = matrices.width();
    let mut game = new_game(width);
    println!("{} Q or Esc quits.", help);

    let term = RawTerminal::new().expect("Failed to set up the terminal");
    let mut input = Input::default();

    let start = Instant::now();
    let mut last = 0.0;
    let mut frames: u64 = 0;
    let mut fps_frames = 0;
    let mut fps_since = start;
    let mut measured_fps = 0.0;
    loop {
        read_input(&mut input, term.key_releases, game.player_x(), width as f32)
            .expect("Failed to read input");
        if input.quit {
            break;
        }
        let now = start.elapsed().as_secs_f32();
        // Don't jump ahead after a hiccup, like the matrix not responding for a while
        let dt = (now - last).min(0.05);
        last = now;
        game.update(&mut input, now, dt);
        input.action = false;
        input.presses.clear();
        matrices.send(&game.render(now));
        frames += 1;

        fps_frames += 1;
        if fps_since.elapsed() >= Duration::from_millis(500) {
            measured_fps = fps_frames as f64 / fps_since.elapsed().as_secs_f64();
            fps_frames = 0;
            fps_since = Instant::now();
            print!("\r{}\x1b[K", game.status(measured_fps));
            let _ = stdout().flush();
        }

        if let Some(fps) = fps {
            let next_frame = start + Duration::from_secs_f64(frames as f64 / fps as f64);
            if let Some(wait) = next_frame.checked_duration_since(Instant::now()) {
                std::thread::sleep(wait);
            }
        }
    }

    // Leave the matrix dark instead of on the last frame
    matrices.clear();
    drop(term);
    println!("{}", game.status(measured_fps));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_covers_partial_leds() {
        let mut light = dark(crate::inputmodule::WIDTH);
        add_square(&mut light, 1.25, 1.5, 1.0);
        let total: f32 = light.iter().flatten().sum();
        assert!((total - 1.0).abs() < 1e-5);
        assert!((light[0][1] - 0.25).abs() < 1e-5);
        assert!((light[1][1] - 0.75).abs() < 1e-5);
    }
}
