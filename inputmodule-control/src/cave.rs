//! Cave descent game: fall down an endless winding cave without touching the walls.
//! The cave scrolls by fractions of an LED, so it flows smoothly at high frame rates.

use std::collections::VecDeque;
use std::f32::consts::PI;

use rand::prelude::*;

use crate::hostgame::{add_rect, add_square, dark, play, to_grid, Burst, HostGame, Input, Light};
use crate::inputmodule::{GreyGrid, HEIGHT};

/// Physics steps per second, independent of the frame rate
const STEPS_PER_SEC: f32 = 480.0;

/// Center of the row the player stays in, the cave scrolls up past it.
/// Near the top, to see far ahead.
const PLAYER_Y: f32 = 5.5;
/// How fast the player moves while an arrow key is held, in LEDs per second
const PLAYER_KEY_SPEED: f32 = 12.0;
/// How fast the player catches up with the mouse or a key press, in LEDs per second
const PLAYER_MAX_SPEED: f32 = 30.0;
/// Half the size of the player's hitbox. A bit smaller than the LED,
/// so just grazing a wall doesn't count.
const PLAYER_HITBOX: f32 = 0.35;
/// Faded copies of the player where it was this long ago, and how bright they are
const TRAIL: [(f32, f32); 3] = [(0.02, 0.3), (0.04, 0.15), (0.06, 0.07)];

/// Falling speed in LEDs per second, grows over time
const START_SPEED: f32 = 8.0;
const ACCELERATION: f32 = 0.15;
const MAX_SPEED: f32 = 30.0;

/// The shape of the cave is set every this many LEDs of depth, and curves smoothly in between
const SEGMENT: f32 = 5.0;
/// No curves at first, so the start is easy
const STRAIGHT_START: f32 = 20.0;
/// Half the width of the cave, it gets narrower the deeper it goes
const START_HALF_WIDTH: f32 = 3.0;
const MIN_HALF_WIDTH: f32 = 1.25;
const NARROWING: f32 = 0.004;
/// How much steeper the cave may curve than the player can follow with the arrow keys
const MAX_STEEPNESS: f32 = 0.75;

/// Walls are dim, with a brighter rim along the edge of the cave
const WALL_BRIGHTNESS: f32 = 0.1;
const RIM_BRIGHTNESS: f32 = 0.35;
/// Samples per LED row, so the walls scroll smoothly by fractions of an LED
const ROW_SAMPLES: usize = 4;

/// Chance of a gem in each segment, and how many points it's worth
const GEM_CHANCE: f64 = 0.4;
const GEM_POINTS: u32 = 10;
const GEM_SIZE: f32 = 0.7;
const GEM_FADE_SECS: f32 = 0.25;

const CRASH_SECS: f32 = 1.0;

struct Segment {
    depth: f32,
    center: f32,
    half_width: f32,
}

struct Gem {
    depth: f32,
    x: f32,
    taken_at: Option<f32>,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    /// Waiting for the player to start
    Ready,
    Falling,
    Crashed {
        since: f32,
    },
}

struct Game {
    rng: ThreadRng,
    /// Screen width in LEDs, one or more LED matrices side by side
    width: f32,
    state: State,
    /// Depth of the top of the screen, how far the player has fallen
    depth: f32,
    speed: f32,
    player_x: f32,
    /// Shape of the cave, from just above the screen to beyond its bottom
    segments: VecDeque<Segment>,
    gems: VecDeque<Gem>,
    gems_taken: u32,
    best: u32,
    /// Where the player crashed
    burst: Option<Burst>,
    /// Recent player positions with the time, for the trail. Newest last.
    trail: VecDeque<(f32, f32)>,
}

impl Game {
    fn new(width: usize) -> Self {
        let mut game = Self {
            rng: thread_rng(),
            width: width as f32,
            state: State::Ready,
            depth: 0.0,
            speed: START_SPEED,
            player_x: width as f32 / 2.0,
            segments: VecDeque::new(),
            gems: VecDeque::new(),
            gems_taken: 0,
            best: 0,
            burst: None,
            trail: VecDeque::new(),
        };
        game.restart();
        game
    }

    fn restart(&mut self) {
        self.state = State::Ready;
        self.depth = 0.0;
        self.speed = START_SPEED;
        self.player_x = self.width / 2.0;
        self.segments.clear();
        self.gems.clear();
        self.gems_taken = 0;
        self.trail.clear();
        self.segments.push_back(Segment {
            depth: -SEGMENT,
            center: self.width / 2.0,
            half_width: START_HALF_WIDTH,
        });
        self.extend_cave();
    }

    fn score(&self) -> u32 {
        self.depth as u32 + self.gems_taken * GEM_POINTS
    }

    /// Add segments until the cave reaches past the bottom of the screen,
    /// and drop the ones that scrolled off the top
    fn extend_cave(&mut self) {
        let bottom = self.depth + HEIGHT as f32 + SEGMENT;
        while let Some(last) = self.segments.back().filter(|s| s.depth < bottom) {
            let depth = last.depth + SEGMENT;
            let narrowed = START_HALF_WIDTH - NARROWING * depth.max(0.0);
            let half_width = (narrowed + self.rng.gen_range(-0.3..0.3)).max(MIN_HALF_WIDTH);
            let center = if depth < STRAIGHT_START {
                self.width / 2.0
            } else {
                // Not steeper than the player can follow at the current speed.
                // Curving with a cosine is steepest in the middle, at pi/2 times the average.
                let steepness = (MAX_STEEPNESS * PLAYER_KEY_SPEED / self.speed).min(1.0);
                let max_shift = steepness * SEGMENT * 2.0 / PI;
                let shifted = last.center + self.rng.gen_range(-max_shift..max_shift);
                shifted.clamp(half_width, self.width - half_width)
            };
            if depth >= STRAIGHT_START && self.rng.gen_bool(GEM_CHANCE) {
                let room = half_width - GEM_SIZE;
                self.gems.push_back(Gem {
                    depth,
                    x: center + self.rng.gen_range(-room..room),
                    taken_at: None,
                });
            }
            self.segments.push_back(Segment {
                depth,
                center,
                half_width,
            });
        }
        while self.segments.len() > 2 && self.segments[1].depth < self.depth - SEGMENT {
            self.segments.pop_front();
        }
        while self
            .gems
            .front()
            .is_some_and(|g| g.depth < self.depth - 1.0)
        {
            self.gems.pop_front();
        }
    }

    /// Left and right wall of the cave at a depth, curving smoothly between segments
    fn walls(&self, depth: f32) -> (f32, f32) {
        let first = self.segments[0].depth;
        let i = (((depth - first) / SEGMENT).max(0.0) as usize).min(self.segments.len() - 2);
        let (a, b) = (&self.segments[i], &self.segments[i + 1]);
        let t = ((depth - a.depth) / SEGMENT).clamp(0.0, 1.0);
        let s = (1.0 - (t * PI).cos()) / 2.0;
        let center = a.center + (b.center - a.center) * s;
        let half_width = a.half_width + (b.half_width - a.half_width) * s;
        (center - half_width, center + half_width)
    }

    fn crashed(&self) -> bool {
        let player_depth = self.depth + PLAYER_Y;
        [-PLAYER_HITBOX, 0.0, PLAYER_HITBOX].iter().any(|dy| {
            let (left, right) = self.walls(player_depth + dy);
            self.player_x - PLAYER_HITBOX < left || self.player_x + PLAYER_HITBOX > right
        })
    }

    fn fall(&mut self, dt: f32, now: f32) {
        self.speed = (self.speed + ACCELERATION * dt).min(MAX_SPEED);
        self.depth += self.speed * dt;
        self.extend_cave();

        let player_depth = self.depth + PLAYER_Y;
        for gem in self.gems.iter_mut().filter(|g| g.taken_at.is_none()) {
            if (gem.depth - player_depth).abs() < 0.8 && (gem.x - self.player_x).abs() < 0.8 {
                gem.taken_at = Some(now);
                self.gems_taken += 1;
            }
        }

        if self.crashed() {
            self.state = State::Crashed { since: now };
            self.best = self.best.max(self.score());
            self.burst = Some(Burst::new(&mut self.rng, self.player_x, PLAYER_Y));
        }
    }
}

impl HostGame for Game {
    fn player_x(&self) -> f32 {
        self.player_x
    }

    fn update(&mut self, input: &mut Input, now: f32, dt: f32) {
        if let State::Crashed { since } = self.state {
            if input.action && now - since >= CRASH_SECS {
                self.restart();
            }
            return;
        }

        self.player_x = input
            .steer(self.player_x, dt, PLAYER_KEY_SPEED, PLAYER_MAX_SPEED)
            .clamp(0.5, self.width - 0.5);
        self.trail.push_back((now, self.player_x));
        while self.trail.front().is_some_and(|&(t, _)| now - t > 0.1) {
            self.trail.pop_front();
        }

        match self.state {
            State::Ready if input.action => self.state = State::Falling,
            State::Falling => {
                let steps = (dt * STEPS_PER_SEC).ceil().max(1.0);
                for i in 0..steps as u32 {
                    let step_time = now - dt + dt * (i + 1) as f32 / steps;
                    self.fall(dt / steps, step_time);
                    if self.state != State::Falling {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn render(&self, now: f32) -> GreyGrid {
        let mut light: Light = dark(self.width as usize);

        // How much of the span x0..x1 lies within a..b
        let overlap = |x0: f32, x1: f32, a: f32, b: f32| (x1.min(b) - x0.max(a)).max(0.0);
        for y in 0..HEIGHT {
            for sample in 0..ROW_SAMPLES {
                let depth = self.depth + y as f32 + (sample as f32 + 0.5) / ROW_SAMPLES as f32;
                let (left, right) = self.walls(depth);
                for (x, col) in light.iter_mut().enumerate() {
                    let (x0, x1) = (x as f32, x as f32 + 1.0);
                    let wall = overlap(x0, x1, f32::MIN, left) + overlap(x0, x1, right, f32::MAX);
                    let rim =
                        overlap(x0, x1, left - 1.0, left) + overlap(x0, x1, right, right + 1.0);
                    col[y] += (WALL_BRIGHTNESS * wall + RIM_BRIGHTNESS * rim) / ROW_SAMPLES as f32;
                }
            }
        }

        for gem in &self.gems {
            let y = gem.depth - self.depth;
            let (size, brightness) = match gem.taken_at {
                // Pulse while waiting to be taken
                None => (GEM_SIZE, 0.6 + 0.3 * (now * 10.0).sin()),
                // Grow and fade out when taken
                Some(t) if now - t < GEM_FADE_SECS => {
                    let age = (now - t) / GEM_FADE_SECS;
                    (GEM_SIZE + 1.5 * age, 1.0 - age)
                }
                Some(_) => continue,
            };
            let half = size / 2.0;
            add_rect(
                &mut light,
                (gem.x - half, y - half),
                (gem.x + half, y + half),
                brightness,
            );
        }

        match self.state {
            State::Crashed { since } => {
                if let Some(burst) = &self.burst {
                    burst.draw(&mut light, now - since, CRASH_SECS);
                }
            }
            _ => {
                // The trail streams upwards, the way the cave scrolls
                for &(ago, brightness) in &TRAIL {
                    if let Some(&(_, x)) = self.trail.iter().rev().find(|&&(t, _)| now - t >= ago) {
                        add_square(&mut light, x, PLAYER_Y - self.speed * ago, brightness);
                    }
                }
                add_square(&mut light, self.player_x, PLAYER_Y, 1.0);
            }
        }

        let dim = match self.state {
            State::Crashed { since } if now - since >= CRASH_SECS => 0.4,
            _ => 1.0,
        };
        to_grid(&light, dim)
    }

    fn status(&self, fps: f64) -> String {
        let state = match self.state {
            State::Ready => "Space to start",
            State::Falling => "",
            State::Crashed { .. } => "Crashed! Space to try again, Q to quit",
        };
        format!(
            "Depth {}  Gems {}  Score {}  Best {}  Speed {:.0}  {:.0} FPS  {}",
            self.depth as u32,
            self.gems_taken,
            self.score(),
            self.best,
            self.speed,
            fps,
            state
        )
    }
}

/// Play until the player quits, as fast as possible or limited to fps
pub fn cave_cmd(serialdevs: &[String], extend: bool, fps: Option<u32>) {
    play(
        serialdevs,
        extend,
        fps,
        "Steer with the arrow keys, or move the mouse across this terminal.
Space or a click starts.",
        Game::new,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inputmodule::WIDTH;

    /// Following the middle of the cave gets through, even at full speed and width
    #[test]
    fn autoplay_survives() {
        // One LED matrix, and two side by side with --extend
        for width in [WIDTH, 2 * WIDTH] {
            let mut game = Game::new(width);
            let mut input = Input {
                action: true,
                ..Default::default()
            };
            let dt = 1.0 / 120.0;
            let mut now = 0.0;
            while now < 200.0 {
                let (left, right) = game.walls(game.depth + PLAYER_Y + 1.0);
                input.target = Some((left + right) / 2.0);
                now += dt;
                game.update(&mut input, now, dt);
                game.render(now);
                assert!(
                    !matches!(game.state, State::Crashed { .. }),
                    "Crashed at depth {}",
                    game.depth
                );

                for y in 0..HEIGHT {
                    let (left, right) = game.walls(game.depth + y as f32);
                    assert!(
                        left >= -0.01 && right <= game.width + 0.01,
                        "Cave leaves the screen"
                    );
                    assert!(
                        right - left >= 2.0 * MIN_HALF_WIDTH - 0.01,
                        "Cave too narrow"
                    );
                }
            }
            assert_eq!(game.speed, MAX_SPEED);
            println!("Depth {:.0}, {} gems", game.depth, game.gems_taken);
        }
    }
}
