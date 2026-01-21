use crate::control::*;
use crate::games::game_of_life::*;
use crate::games::pong_animation::*;
use crate::games::snake_animation::*;
use crate::matrix::Grid;
use crate::matrix::*;
use crate::patterns::*;

// TODO
// - [ ] Is there a cancellable Iterator? I think Java/Kotlin has one
// - [ ] Each one has a number of frames
// - [ ] Each one might have a different frame-rate

#[allow(clippy::large_enum_variant)]
pub enum Animation {
    ZigZag(ZigZagIterator),
    Gof(GameOfLifeIterator),
    Percentage(StartupPercentageIterator),
    Breathing(BreathingIterator),
    Snake(SnakeIterator),
    Pong(PongIterator),
}
impl Iterator for Animation {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Animation::ZigZag(x) => x.next(),
            Animation::Gof(x) => x.next(),
            Animation::Percentage(x) => x.next(),
            Animation::Breathing(x) => x.next(),
            Animation::Snake(x) => x.next(),
            Animation::Pong(x) => x.next(),
        }
    }
}

pub struct ZigZagIterator {
    frames: usize,
    current_frame: usize,
}

impl ZigZagIterator {
    pub fn new(frames: usize) -> Self {
        Self {
            frames,
            current_frame: 0,
        }
    }
}

impl Default for ZigZagIterator {
    fn default() -> Self {
        Self::new(34)
    }
}

impl Iterator for ZigZagIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame < self.frames {
            let mut next = zigzag();
            next.rotate(self.current_frame);
            self.current_frame += 1;
            Some(next)
        } else {
            None
        }
    }
}

pub struct StartupPercentageIterator {
    frames: usize,
    current_frame: usize,
}

impl Default for StartupPercentageIterator {
    fn default() -> Self {
        Self {
            frames: 34,
            current_frame: 0,
        }
    }
}

impl Iterator for StartupPercentageIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame < self.frames {
            self.current_frame += 1;
            Some(rows(self.current_frame))
        } else {
            None
        }
    }
}

pub struct GameOfLifeIterator {
    state: GameOfLifeState,
    frames_remaining: usize,
}

impl GameOfLifeIterator {
    pub fn new(start_param: GameOfLifeStartParam, frames: usize) -> Self {
        Self {
            // Could start with a custom grid
            state: GameOfLifeState::new(start_param, &Grid::default()),
            frames_remaining: frames,
        }
    }
}

impl Iterator for GameOfLifeIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
            // Only update every 8th frame, otherwise the animation is too fast
            if self.frames_remaining.is_multiple_of(8) {
                self.state.tick();
            }
            Some(self.state.draw_matrix())
        } else {
            None
        }
    }
}

pub struct BreathingIterator {
    frames_remaining: usize,
    current_brightness: u8,
}

impl BreathingIterator {
    pub fn new(frames: usize) -> Self {
        Self {
            frames_remaining: frames,
            current_brightness: 0,
        }
    }
}
impl Default for BreathingIterator {
    fn default() -> Self {
        Self::new(64)
    }
}

impl Iterator for BreathingIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frames_remaining > 0 {
            let mut grid = Grid::default();
            let breath_step = 4;
            // TODO: Make it cycle up and down
            self.current_brightness = (self.current_brightness + breath_step) % 255;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    grid.0[x][y] = self.current_brightness;
                }
            }
            self.frames_remaining -= 1;
            Some(grid)
        } else {
            None
        }
    }
}
