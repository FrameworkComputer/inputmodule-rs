//! Brick breaker game, played on the host and streamed to the LED matrix

use std::collections::VecDeque;

use rand::prelude::*;

use crate::hostgame::{add_rect, add_square, dark, play, to_grid, HostGame, Input, Light, FIELD_H};
use crate::inputmodule::{GreyGrid, HEIGHT};

/// Physics steps per second, independent of the frame rate.
/// Small enough steps that even the fastest ball can't skip through a brick.
const STEPS_PER_SEC: f32 = 480.0;

/// Top edge of the paddle, it's one LED tall
const PADDLE_Y: f32 = 32.0;
const PADDLE_WIDTH: f32 = 3.0;
const PADDLE_BRIGHTNESS: f32 = 0.6;
/// How fast the paddle moves while an arrow key is held, in LEDs per second
const PADDLE_KEY_SPEED: f32 = 14.0;
/// How fast the paddle catches up with the mouse or a key press, in LEDs per second
const PADDLE_MAX_SPEED: f32 = 40.0;

/// The ball is a square, one LED wide
const BALL_RADIUS: f32 = 0.5;
/// Ball speed in LEDs per second
const START_SPEED: f32 = 12.0;
const LEVEL_SPEEDUP: f32 = 2.0;
const PADDLE_SPEEDUP: f32 = 0.4;
const MAX_SPEED: f32 = 30.0;
/// Hitting the paddle at the very edge sends the ball off at this angle from vertical
const MAX_BOUNCE_ANGLE: f32 = std::f32::consts::FRAC_PI_3;
/// Fading copies of the ball where it was this long ago, and how bright they are
const TRAIL: [(f32, f32); 3] = [(0.012, 0.35), (0.024, 0.18), (0.036, 0.08)];

const BRICK_WIDTH: usize = 3;
/// Row of the topmost bricks, leaves room for the ball to bounce around above them
const BRICK_TOP: usize = 2;
/// How long a broken brick takes to fade out
const BRICK_FADE_SECS: f32 = 0.3;

const BALLS: u32 = 3;
/// The ball launches by itself if the player doesn't
const AUTO_LAUNCH_SECS: f32 = 3.0;
const LOST_SECS: f32 = 1.0;
const CLEARED_SECS: f32 = 1.2;

struct Brick {
    x: usize,
    y: usize,
    /// Hits left until it breaks
    hits: u8,
    /// When it was last hit, to flash it
    hit_at: Option<f32>,
}

impl Brick {
    fn left(&self) -> f32 {
        self.x as f32
    }
    fn right(&self) -> f32 {
        (self.x + BRICK_WIDTH) as f32
    }
    fn top(&self) -> f32 {
        self.y as f32
    }
    fn bottom(&self) -> f32 {
        (self.y + 1) as f32
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    /// Ball rests on the paddle until launched
    Serving {
        since: f32,
    },
    Playing,
    /// Ball fell off the bottom
    Lost {
        since: f32,
    },
    /// All bricks broken, about to start the next level
    Cleared {
        since: f32,
    },
    GameOver,
}

struct Game {
    rng: ThreadRng,
    /// Screen width in LEDs, one or more LED matrices side by side
    width: f32,
    state: State,
    level: u32,
    score: u32,
    balls: u32,
    bricks: Vec<Brick>,
    /// Center of the paddle
    paddle_x: f32,
    /// Center of the ball
    ball_x: f32,
    ball_y: f32,
    vel_x: f32,
    vel_y: f32,
    speed: f32,
    /// Recent ball positions for the trail, newest last
    trail: VecDeque<(f32, f32, f32)>,
}

impl Game {
    fn new(width: usize) -> Self {
        let mut game = Self {
            rng: thread_rng(),
            width: width as f32,
            state: State::Serving { since: 0.0 },
            level: 1,
            score: 0,
            balls: BALLS,
            bricks: vec![],
            paddle_x: width as f32 / 2.0,
            ball_x: 0.0,
            ball_y: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            speed: START_SPEED,
            trail: VecDeque::new(),
        };
        game.build_level();
        game
    }

    /// More rows every level, and from level 2 the top rows take two hits
    fn build_level(&mut self) {
        let rows = (3 + self.level as usize).min(10);
        let tough_rows = (self.level as usize - 1).min(rows - 2);
        let cols = self.width as usize / BRICK_WIDTH;
        self.bricks = (0..rows)
            .flat_map(|row| {
                (0..cols).map(move |col| Brick {
                    x: col * BRICK_WIDTH,
                    y: BRICK_TOP + row,
                    hits: if row < tough_rows { 2 } else { 1 },
                    hit_at: None,
                })
            })
            .collect();
    }

    fn serve(&mut self, now: f32) {
        self.state = State::Serving { since: now };
        self.speed = START_SPEED + LEVEL_SPEEDUP * (self.level - 1) as f32;
        self.trail.clear();
    }

    fn launch(&mut self) {
        // Slightly to a random side, so it's not the same every time
        let angle: f32 = self.rng.gen_range(-0.35..0.35);
        self.vel_x = self.speed * angle.sin();
        self.vel_y = -self.speed * angle.cos();
        self.state = State::Playing;
    }

    fn restart(&mut self, now: f32) {
        self.level = 1;
        self.score = 0;
        self.balls = BALLS;
        self.build_level();
        self.serve(now);
    }

    fn step_ball(&mut self, dt: f32, now: f32) {
        self.ball_x += self.vel_x * dt;
        self.ball_y += self.vel_y * dt;
        self.trail.push_back((now, self.ball_x, self.ball_y));
        let oldest = TRAIL[TRAIL.len() - 1].0 + 0.01;
        while self
            .trail
            .front()
            .is_some_and(|&(t, _, _)| now - t > oldest)
        {
            self.trail.pop_front();
        }

        // Walls
        if self.ball_x - BALL_RADIUS < 0.0 {
            self.ball_x = BALL_RADIUS;
            self.vel_x = self.vel_x.abs();
        }
        if self.ball_x + BALL_RADIUS > self.width {
            self.ball_x = self.width - BALL_RADIUS;
            self.vel_x = -self.vel_x.abs();
        }
        if self.ball_y - BALL_RADIUS < 0.0 {
            self.ball_y = BALL_RADIUS;
            self.vel_y = self.vel_y.abs();
        }

        // Paddle. Where it hits decides the angle it bounces off at.
        let reach = PADDLE_WIDTH / 2.0 + BALL_RADIUS;
        let bottom = self.ball_y + BALL_RADIUS;
        if self.vel_y > 0.0
            && (PADDLE_Y..PADDLE_Y + 0.5).contains(&bottom)
            && (self.ball_x - self.paddle_x).abs() <= reach
        {
            let offset = ((self.ball_x - self.paddle_x) / reach).clamp(-1.0, 1.0);
            let angle = offset * MAX_BOUNCE_ANGLE;
            self.speed = (self.speed + PADDLE_SPEEDUP).min(MAX_SPEED);
            self.vel_x = self.speed * angle.sin();
            self.vel_y = -self.speed * angle.cos();
            self.ball_y = PADDLE_Y - BALL_RADIUS;
        }

        // Bricks. Bounce off the side that the ball overlaps the least,
        // that's the side it came from.
        let (left, right) = (self.ball_x - BALL_RADIUS, self.ball_x + BALL_RADIUS);
        let (top, bottom) = (self.ball_y - BALL_RADIUS, self.ball_y + BALL_RADIUS);
        if let Some(brick) = self.bricks.iter_mut().find(|b| {
            b.hits > 0
                && left < b.right()
                && right > b.left()
                && top < b.bottom()
                && bottom > b.top()
        }) {
            let overlap_x = (right - brick.left()).min(brick.right() - left);
            let overlap_y = (bottom - brick.top()).min(brick.bottom() - top);
            if overlap_x < overlap_y {
                let from_left = self.ball_x < (brick.left() + brick.right()) / 2.0;
                self.vel_x = if from_left {
                    -self.vel_x.abs()
                } else {
                    self.vel_x.abs()
                };
                self.ball_x += if from_left { -overlap_x } else { overlap_x };
            } else {
                let from_above = self.ball_y < (brick.top() + brick.bottom()) / 2.0;
                self.vel_y = if from_above {
                    -self.vel_y.abs()
                } else {
                    self.vel_y.abs()
                };
                self.ball_y += if from_above { -overlap_y } else { overlap_y };
            }
            brick.hits -= 1;
            brick.hit_at = Some(now);
            self.score += if brick.hits == 0 { 10 } else { 5 };
            if self.bricks.iter().all(|b| b.hits == 0) {
                self.score += 100 * self.level;
                self.state = State::Cleared { since: now };
            }
        }

        if self.ball_y - BALL_RADIUS > FIELD_H {
            self.balls -= 1;
            self.state = State::Lost { since: now };
        }
    }
}

impl HostGame for Game {
    fn player_x(&self) -> f32 {
        self.paddle_x
    }

    fn update(&mut self, input: &mut Input, now: f32, dt: f32) {
        let half = PADDLE_WIDTH / 2.0;
        self.paddle_x = input
            .steer(self.paddle_x, dt, PADDLE_KEY_SPEED, PADDLE_MAX_SPEED)
            .clamp(half, self.width - half);
        match self.state {
            State::Serving { since } => {
                self.ball_x = self.paddle_x;
                self.ball_y = PADDLE_Y - BALL_RADIUS;
                if input.action || now - since >= AUTO_LAUNCH_SECS {
                    self.launch();
                }
            }
            State::Playing => {
                let steps = (dt * STEPS_PER_SEC).ceil().max(1.0);
                for i in 0..steps as u32 {
                    let step_time = now - dt + dt * (i + 1) as f32 / steps;
                    self.step_ball(dt / steps, step_time);
                    if self.state != State::Playing {
                        break;
                    }
                }
            }
            State::Lost { since } => {
                if now - since >= LOST_SECS {
                    if self.balls == 0 {
                        self.state = State::GameOver;
                    } else {
                        self.serve(now);
                    }
                }
            }
            State::Cleared { since } => {
                if now - since >= CLEARED_SECS {
                    self.level += 1;
                    self.build_level();
                    self.serve(now);
                }
            }
            State::GameOver => {
                if input.action {
                    self.restart(now);
                }
            }
        }
    }

    fn render(&self, now: f32) -> GreyGrid {
        let mut light: Light = dark(self.width as usize);

        for brick in &self.bricks {
            let since_hit = brick.hit_at.map_or(f32::MAX, |t| now - t);
            let brightness = if brick.hits == 0 {
                // Flash and fade out
                if since_hit >= BRICK_FADE_SECS {
                    continue;
                }
                1.0 - since_hit / BRICK_FADE_SECS
            } else {
                // Alternate, so neighboring bricks can be told apart
                let base = match brick.hits {
                    1 if (brick.x / BRICK_WIDTH + brick.y).is_multiple_of(2) => 0.3,
                    1 => 0.5,
                    _ => 0.85,
                };
                // Flash when hit but not broken yet
                let flash = (1.0 - since_hit / 0.15).max(0.0);
                base + (1.0 - base) * flash
            };
            add_rect(
                &mut light,
                (brick.left(), brick.top()),
                (brick.right(), brick.bottom()),
                brightness,
            );
        }

        // Paddle blinks after losing the ball
        let paddle = match self.state {
            State::Lost { since } if ((now - since) * 8.0) as u32 % 2 == 1 => 0.1,
            _ => PADDLE_BRIGHTNESS,
        };
        let half = PADDLE_WIDTH / 2.0;
        add_rect(
            &mut light,
            (self.paddle_x - half, PADDLE_Y),
            (self.paddle_x + half, PADDLE_Y + 1.0),
            paddle,
        );

        if matches!(self.state, State::Playing | State::Serving { .. }) {
            for &(ago, brightness) in &TRAIL {
                if let Some(&(_, x, y)) = self.trail.iter().rev().find(|&&(t, _, _)| now - t >= ago)
                {
                    add_square(&mut light, x, y, brightness);
                }
            }
            add_square(&mut light, self.ball_x, self.ball_y, 1.0);
        }

        // Spare balls as dim dots in the bottom right corner
        let spare = match self.state {
            State::Serving { .. } | State::Playing => self.balls.saturating_sub(1),
            _ => self.balls,
        };
        for i in 0..spare as usize {
            light[self.width as usize - 1 - 2 * i][HEIGHT - 1] += 0.15;
        }

        // A bright band sweeps up after clearing a level
        if let State::Cleared { since } = self.state {
            let y = FIELD_H - (now - since) / CLEARED_SECS * (FIELD_H + 2.0);
            add_rect(&mut light, (0.0, y), (self.width, y + 2.0), 0.8);
        }

        let dim = if self.state == State::GameOver {
            0.25
        } else {
            1.0
        };
        to_grid(&light, dim)
    }

    fn status(&self, fps: f64) -> String {
        let state = match self.state {
            State::Serving { .. } => "Space to launch",
            State::Playing => "",
            State::Lost { .. } => "Lost the ball!",
            State::Cleared { .. } => "Level cleared!",
            State::GameOver => "Game over! Space to play again, Q to quit",
        };
        format!(
            "Level {}  Score {}  Balls {}  {:.0} FPS  {}",
            self.level, self.score, self.balls, fps, state
        )
    }
}

/// Play until the player quits, as fast as possible or limited to fps
pub fn bricks_cmd(serialdevs: &[String], extend: bool, fps: Option<u32>) {
    play(
        serialdevs,
        extend,
        fps,
        "Space or a click launches the ball.",
        Game::new,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inputmodule::WIDTH;

    /// Ball that the paddle always catches clears the level and never gets stuck
    #[test]
    fn autoplay_clears_level() {
        // One LED matrix, and two side by side with --extend
        for width in [WIDTH, 2 * WIDTH] {
            let mut game = Game::new(width);
            let mut input = Input::default();
            let dt = 1.0 / 120.0;
            let mut now: f32 = 0.0;
            while game.level == 1 {
                // Hit it at different spots on the paddle, so it doesn't bounce in a loop
                input.target = Some(game.ball_x + 0.4 + 0.6 * (now * 0.7).sin());
                input.action = true;
                now += dt;
                game.update(&mut input, now, dt);
                game.render(now);
                assert_eq!(game.balls, BALLS, "Lost a ball");
                assert!(now < 300.0, "Didn't clear the level");
            }
            println!("Cleared level 1 in {:.1}s, score {}", now, game.score);
        }
    }
}
