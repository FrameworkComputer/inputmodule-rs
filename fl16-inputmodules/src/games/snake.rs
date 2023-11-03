use crate::control::GameControlArg;
use crate::matrix::{GameState, Grid, LedmatrixState, HEIGHT, LEDS, WIDTH};

use heapless::Vec;

// Wrap around the edges
const WRAP_ENABLE: bool = false;

#[derive(Clone, Debug, Copy)]
pub enum HeadDirection {
    Up,
    Down,
    Left,
    Right,
}

type Position = (i8, i8);

#[derive(Clone)]
pub struct SnakeState {
    head: Position,
    pub direction: HeadDirection,
    // Unrealistic that the body will ever get this long
    pub body: Vec<Position, LEDS>,
    pub game_over: bool,
    food: Position,
}

impl SnakeState {
    pub fn new(random: u8) -> Self {
        SnakeState {
            head: (4, 0),
            direction: HeadDirection::Down,
            body: Vec::new(),
            game_over: false,
            food: place_food(random),
        }
    }
    pub fn tick(&mut self, random: u8) {
        if self.game_over {
            return;
        }

        let (x, y) = self.head;
        let oldhead = self.head;
        self.head = match self.direction {
            // (0, 0) is at the top right corner
            HeadDirection::Right => (x - 1, y),
            HeadDirection::Left => (x + 1, y),
            HeadDirection::Down => (x, y + 1),
            HeadDirection::Up => (x, y - 1),
        };
        let (x, y) = self.head;
        let width = WIDTH as i8;
        let height = HEIGHT as i8;

        if self.body.contains(&self.head) {
            // Ran into itself
            self.game_over = true
        } else if x >= width || x < 0 || y >= height || y < 0 {
            // Hit an edge
            if WRAP_ENABLE {
                self.head = if x >= width {
                    (0, y)
                } else if x < 0 {
                    (width - 1, y)
                } else if y >= height {
                    (x, 0)
                } else if y < 0 {
                    (x, height - 1)
                } else {
                    (x, y)
                };
            } else {
                self.game_over = true
            }
        } else if self.head == self.food {
            // Eating food and growing
            self.body.insert(0, oldhead).unwrap();
            self.food = place_food(random);
        } else if !self.body.is_empty() {
            // Move body along
            self.body.pop();
            self.body.insert(0, oldhead).unwrap();
        }
    }

    pub fn handle_control(&mut self, arg: &GameControlArg) {
        match arg {
            GameControlArg::Up => self.direction = HeadDirection::Up,
            GameControlArg::Down => self.direction = HeadDirection::Down,
            GameControlArg::Left => self.direction = HeadDirection::Left,
            GameControlArg::Right => self.direction = HeadDirection::Right,
            _ => {}
        }
    }
    pub fn draw_matrix(&self) -> Grid {
        let (x, y) = self.head;
        let mut grid = Grid::default();

        // TODO: Why does it crash here? x out of range (9)
        grid.0[x as usize][y as usize] = 0xFF;
        grid.0[self.food.0 as usize][self.food.1 as usize] = 0xFF;
        for bodypart in &self.body {
            let (x, y) = bodypart;
            grid.0[*x as usize][*y as usize] = 0xFF;
        }

        grid
    }
}

fn place_food(random: u8) -> Position {
    // TODO: while food == head:
    let x = ((random & 0xF0) >> 4) % WIDTH as u8;
    let y = (random & 0x0F) % HEIGHT as u8;
    (x as i8, y as i8)
}

pub fn start_game(state: &mut LedmatrixState, random: u8) {
    state.game = Some(GameState::Snake(SnakeState::new(random)));
}

pub fn handle_control(state: &mut LedmatrixState, arg: &GameControlArg) {
    if let Some(GameState::Snake(ref mut snake_state)) = state.game {
        match arg {
            GameControlArg::Exit => state.game = None,
            _ => snake_state.handle_control(arg),
        }
    }
}

pub fn game_step(state: &mut LedmatrixState, random: u8) -> (HeadDirection, bool, usize, Position) {
    if let Some(GameState::Snake(ref mut snake_state)) = state.game {
        snake_state.tick(random);

        if !snake_state.game_over {
            state.grid = snake_state.draw_matrix();
        }

        (
            snake_state.direction,
            snake_state.game_over,
            snake_state.body.len(),
            snake_state.head,
        )
    } else {
        (HeadDirection::Down, true, 0, (0, 0))
    }
}
