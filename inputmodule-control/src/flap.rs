//! One-button flying game: flap to stay up, and fly through the gaps in the walls
//! that slide in from the right. Gravity pulls down along the tall side of the
//! matrix, so every flap is a long, smooth arc.

use std::collections::VecDeque;

use rand::prelude::*;

use crate::hostgame::{
    add_rect, add_square, play, to_grid, Burst, HostGame, Input, Light, FIELD_W,
};
use crate::inputmodule::{GreyGrid, HEIGHT, WIDTH};

/// Physics steps per second, independent of the frame rate
const STEPS_PER_SEC: f32 = 480.0;

/// Center of the column the player flies in, near the left to see the walls coming
const PLAYER_X: f32 = 2.0;
/// Half the size of the player's hitbox. A bit smaller than the LED,
/// so just grazing a wall doesn't count.
const HITBOX: f32 = 0.35;
/// Faded copies of the player where it was this long ago, and how bright they are
const TRAIL: [(f32, f32); 3] = [(0.03, 0.3), (0.06, 0.15), (0.09, 0.07)];

/// In LEDs per second squared, and LEDs per second
const GRAVITY: f32 = 54.0;
const FLAP_SPEED: f32 = 18.0;
const MAX_FALL_SPEED: f32 = 25.0;

/// Top of the bottom row, which is the ground
const GROUND: f32 = (HEIGHT - 1) as f32;
const GROUND_BRIGHTNESS: f32 = 0.25;

/// How fast the walls slide in, in LEDs per second. A bit faster with every wall passed.
const START_SPEED: f32 = 4.0;
const SPEEDUP: f32 = 0.06;
const MAX_SPEED: f32 = 6.5;
/// Distance from one wall to the next
const WALL_SPACING: f32 = 6.0;
const WALL_BRIGHTNESS: f32 = 0.4;
/// Extra brightness at the edges of the gap, to see it better
const EDGE_BRIGHTNESS: f32 = 0.45;
/// Height of the gap in a wall, a bit smaller with every wall passed
const START_GAP: f32 = 10.0;
const GAP_SHRINK: f32 = 0.15;
const MIN_GAP: f32 = 6.5;
/// How fast the player can climb by flapping, on average.
/// The next gap is never higher than that allows.
const CLIMB_SPEED: f32 = 5.0;
/// The next gap is never lower than this below the last one
const MAX_DROP: f32 = 6.0;

const CRASH_SECS: f32 = 1.0;

#[derive(Clone)]
struct Wall {
    /// Left edge, walls are one LED wide
    x: f32,
    gap_top: f32,
    gap_bottom: f32,
    passed: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    /// Hovering until the first flap
    Ready,
    Flying,
    Crashed {
        since: f32,
    },
}

#[derive(Clone)]
struct Game {
    rng: ThreadRng,
    state: State,
    /// Center of the player
    y: f32,
    vel_y: f32,
    speed: f32,
    gap: f32,
    walls: VecDeque<Wall>,
    /// How far the ground has scrolled, for its dashes
    scrolled: f32,
    score: u32,
    best: u32,
    burst: Option<Burst>,
    /// Recent heights of the player with the time, for the trail. Newest last.
    trail: VecDeque<(f32, f32)>,
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            rng: thread_rng(),
            state: State::Ready,
            y: 0.0,
            vel_y: 0.0,
            speed: START_SPEED,
            gap: START_GAP,
            walls: VecDeque::new(),
            scrolled: 0.0,
            score: 0,
            best: 0,
            burst: None,
            trail: VecDeque::new(),
        };
        game.restart();
        game
    }

    fn restart(&mut self) {
        self.state = State::Ready;
        self.y = GROUND / 2.0;
        self.vel_y = 0.0;
        self.speed = START_SPEED;
        self.gap = START_GAP;
        self.walls.clear();
        self.score = 0;
        self.trail.clear();
        self.add_walls();
    }

    /// Add walls until the next one is off the screen on the right,
    /// and drop the ones that slid off on the left
    fn add_walls(&mut self) {
        while self
            .walls
            .back()
            .is_none_or(|w| w.x < FIELD_W + WALL_SPACING)
        {
            let (x, center) = match self.walls.back() {
                // The first wall comes in after a moment
                None => (FIELD_W + 2.0, GROUND / 2.0),
                Some(last) => {
                    let last_center = (last.gap_top + last.gap_bottom) / 2.0;
                    // Anywhere, but not higher than the player can climb in time.
                    // Falling is quick, but not too far either
                    let time = WALL_SPACING / self.speed;
                    let up = (CLIMB_SPEED * time).min(10.0);
                    let center = self.rng.gen_range(0.0..GROUND);
                    let center = center.clamp(last_center - up, last_center + MAX_DROP);
                    (last.x + WALL_SPACING, center)
                }
            };
            let half = self.gap / 2.0;
            let center = center.clamp(half + 1.0, GROUND - half - 1.0);
            self.walls.push_back(Wall {
                x,
                gap_top: center - half,
                gap_bottom: center + half,
                passed: false,
            });
        }
        while self.walls.front().is_some_and(|w| w.x + 1.0 < 0.0) {
            self.walls.pop_front();
        }
    }

    fn crashed(&self) -> bool {
        if self.y + HITBOX > GROUND {
            return true;
        }
        let (left, right) = (PLAYER_X - HITBOX, PLAYER_X + HITBOX);
        let (top, bottom) = (self.y - HITBOX, self.y + HITBOX);
        self.walls
            .iter()
            .any(|w| left < w.x + 1.0 && right > w.x && (top < w.gap_top || bottom > w.gap_bottom))
    }

    fn fly(&mut self, dt: f32, now: f32) {
        self.vel_y = (self.vel_y + GRAVITY * dt).min(MAX_FALL_SPEED);
        self.y += self.vel_y * dt;
        // Bump against the top
        if self.y - HITBOX < 0.0 {
            self.y = HITBOX;
            self.vel_y = 0.0;
        }

        for wall in &mut self.walls {
            wall.x -= self.speed * dt;
            if !wall.passed && wall.x + 1.0 < PLAYER_X - HITBOX {
                wall.passed = true;
                self.score += 1;
                self.speed = (self.speed + SPEEDUP).min(MAX_SPEED);
                self.gap = (self.gap - GAP_SHRINK).max(MIN_GAP);
            }
        }
        self.scrolled += self.speed * dt;
        self.add_walls();

        if self.crashed() {
            self.state = State::Crashed { since: now };
            self.best = self.best.max(self.score);
            self.burst = Some(Burst::new(&mut self.rng, PLAYER_X, self.y));
        }
    }
}

impl HostGame for Game {
    fn player_x(&self) -> f32 {
        PLAYER_X
    }

    fn update(&mut self, input: &mut Input, now: f32, dt: f32) {
        match self.state {
            State::Crashed { since } => {
                if input.action && now - since >= CRASH_SECS {
                    self.restart();
                }
                return;
            }
            State::Ready => {
                // Bob up and down while waiting
                self.y = GROUND / 2.0 + (now * 4.0).sin() * 0.8;
                if input.action {
                    self.state = State::Flying;
                }
            }
            State::Flying => {}
        }

        if self.state == State::Flying {
            if input.action {
                self.vel_y = -FLAP_SPEED;
            }
            let steps = (dt * STEPS_PER_SEC).ceil().max(1.0);
            for i in 0..steps as u32 {
                let step_time = now - dt + dt * (i + 1) as f32 / steps;
                self.fly(dt / steps, step_time);
                if self.state != State::Flying {
                    break;
                }
            }
        }

        self.trail.push_back((now, self.y));
        while self.trail.front().is_some_and(|&(t, _)| now - t > 0.15) {
            self.trail.pop_front();
        }
    }

    fn render(&self, now: f32) -> GreyGrid {
        let mut light: Light = [[0.0; HEIGHT]; WIDTH];

        // Dashed ground, scrolling along with the walls
        let offset = self.scrolled % 2.0;
        for i in 0..=WIDTH / 2 + 1 {
            let x = i as f32 * 2.0 - offset;
            add_rect(
                &mut light,
                (x, GROUND),
                (x + 1.0, GROUND + 1.0),
                GROUND_BRIGHTNESS,
            );
        }

        for wall in &self.walls {
            let (left, right) = (wall.x, wall.x + 1.0);
            add_rect(
                &mut light,
                (left, 0.0),
                (right, wall.gap_top),
                WALL_BRIGHTNESS,
            );
            add_rect(
                &mut light,
                (left, wall.gap_bottom),
                (right, GROUND),
                WALL_BRIGHTNESS,
            );
            let (top, bottom) = (wall.gap_top, wall.gap_bottom);
            add_rect(&mut light, (left, top - 1.0), (right, top), EDGE_BRIGHTNESS);
            add_rect(
                &mut light,
                (left, bottom),
                (right, bottom + 1.0),
                EDGE_BRIGHTNESS,
            );
        }

        match self.state {
            State::Crashed { since } => {
                if let Some(burst) = &self.burst {
                    burst.draw(&mut light, now - since, CRASH_SECS);
                }
            }
            _ => {
                // The trail streams to the left, the way the walls slide
                let speed = if self.state == State::Flying {
                    self.speed
                } else {
                    0.0
                };
                for &(ago, brightness) in &TRAIL {
                    if let Some(&(_, y)) = self.trail.iter().rev().find(|&&(t, _)| now - t >= ago) {
                        add_square(&mut light, PLAYER_X - speed * ago, y, brightness);
                    }
                }
                add_square(&mut light, PLAYER_X, self.y, 1.0);
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
            State::Ready => "Space to flap",
            State::Flying => "",
            State::Crashed { .. } => "Crashed! Space to try again, Q to quit",
        };
        format!(
            "Score {}  Best {}  Speed {:.1}  {:.0} FPS  {}",
            self.score, self.best, self.speed, fps, state
        )
    }
}

/// Play until the player quits, as fast as possible or limited to fps
pub fn flap_cmd(serialdevs: &[String], fps: Option<u32>) {
    play(
        serialdevs,
        fps,
        "Space, up or a click flaps.",
        &mut Game::new(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple rule: flap whenever below the middle of the next gap,
    /// but while in a wall or about to enter one, stay in its gap
    fn greedy_flap(game: &Game) -> bool {
        // A flap goes up about 3 LEDs, so aim a bit below the middle
        let next = game.walls.iter().find(|w| w.x > PLAYER_X + HITBOX).unwrap();
        let mut target = (next.gap_top + next.gap_bottom) / 2.0 + 1.0;
        let inside = |w: &&Wall| w.x < PLAYER_X + HITBOX + 1.0 && w.x + 1.0 > PLAYER_X - HITBOX;
        if let Some(wall) = game.walls.iter().find(inside) {
            target = target.clamp(wall.gap_top + 3.5, wall.gap_bottom - 1.0);
        }
        game.y > target && game.vel_y > 0.0
    }

    /// How long the game lasts after flapping or not, then following the simple rule
    fn survives(mut game: Game, flap: bool, now: f32, dt: f32, secs: f32) -> f32 {
        let mut input = Input {
            action: flap,
            ..Default::default()
        };
        let mut t = 0.0;
        while t < secs {
            t += dt;
            game.update(&mut input, now + t, dt);
            if game.state != State::Flying {
                return t;
            }
            input.action = greedy_flap(&game);
        }
        secs
    }

    /// A player who looks ahead gets through every wall, even at full speed with
    /// the smallest gaps, so the hardest walls are still fair
    #[test]
    fn autoplay_survives() {
        let mut game = Game::new();
        let mut input = Input::default();
        let dt = 1.0 / 60.0;
        let lookahead = 1.0;
        let mut now = 0.0;
        while now < 150.0 {
            // Flap only if not flapping would crash
            input.action = game.state == State::Ready
                || (survives(game.clone(), false, now, dt, lookahead) < lookahead
                    && survives(game.clone(), true, now, dt, lookahead)
                        > survives(game.clone(), false, now, dt, lookahead));
            now += dt;
            game.update(&mut input, now, dt);
            game.render(now);
            assert!(
                !matches!(game.state, State::Crashed { .. }),
                "Crashed after {} walls at y {:.2}, speed {:.1}, walls {:?}",
                game.score,
                game.y,
                game.vel_y,
                game.walls
                    .iter()
                    .map(|w| (w.x, w.gap_top, w.gap_bottom))
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(game.speed, MAX_SPEED);
        assert_eq!(game.gap, MIN_GAP);
        println!("Passed {} walls", game.score);
    }
}
