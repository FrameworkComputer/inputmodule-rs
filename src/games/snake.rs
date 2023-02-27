use crate::control::GameControlArg;
use crate::matrix::{GameState, Grid, State, HEIGHT, LEDS, WIDTH};

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

fn place_food(random: u8) -> Position {
    // TODO: while food == head:
    let x = ((random & 0xF0) >> 4) % WIDTH as u8;
    let y = (random & 0x0F) % HEIGHT as u8;
    (x as i8, y as i8)
}

pub fn start_game(state: &mut State, random: u8) {
    state.game = Some(GameState::Snake(SnakeState {
        head: (4, 0),
        direction: HeadDirection::Down,
        body: Vec::new(),
        game_over: false,
        food: place_food(random),
    }));
}
pub fn handle_control(state: &mut State, arg: &GameControlArg) {
    if let Some(GameState::Snake(ref mut snake_state)) = state.game {
        match arg {
            GameControlArg::Up => snake_state.direction = HeadDirection::Up,
            GameControlArg::Down => snake_state.direction = HeadDirection::Down,
            GameControlArg::Left => snake_state.direction = HeadDirection::Left,
            GameControlArg::Right => snake_state.direction = HeadDirection::Right,
            //GameControlArg::Exit => {
            _ => {
                // TODO
            }
        }
    }
}

pub fn game_step(state: &mut State, random: u8) -> (HeadDirection, bool, usize, Position) {
    if let Some(GameState::Snake(ref mut snake_state)) = state.game {
        if snake_state.game_over {
            return (
                snake_state.direction,
                snake_state.game_over,
                snake_state.body.len(),
                snake_state.head,
            );
        }

        let (x, y) = snake_state.head;
        let oldhead = snake_state.head.clone();
        snake_state.head = match snake_state.direction {
            // (0, 0) is at the top right corner
            HeadDirection::Right => (x - 1, y),
            HeadDirection::Left => (x + 1, y),
            HeadDirection::Down => (x, y + 1),
            HeadDirection::Up => (x, y - 1),
        };
        let (x, y) = snake_state.head;
        let width = WIDTH as i8;
        let height = HEIGHT as i8;

        if snake_state.body.contains(&snake_state.head) {
            // Ran into itself
            snake_state.game_over = true
        } else if x >= width || x < 0 || y >= height || y < 0 {
            // Hit an edge
            if WRAP_ENABLE {
                snake_state.head = if x >= width {
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
                snake_state.game_over = true
            }
        } else if snake_state.head == snake_state.food {
            // Eating food and growing
            snake_state.body.insert(0, oldhead).unwrap();
            snake_state.food = place_food(random);
        } else if !snake_state.body.is_empty() {
            // Move body along
            snake_state.body.pop();
            snake_state.body.insert(0, oldhead).unwrap();
        }

        if !snake_state.game_over {
            state.grid = draw_matrix(&snake_state);
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

fn draw_matrix(state: &SnakeState) -> Grid {
    let (x, y) = state.head;
    let mut grid = Grid::default();

    grid.0[x as usize][y as usize] = 0xFF;
    grid.0[state.food.0 as usize][state.food.1 as usize] = 0xFF;
    for bodypart in &state.body {
        let (x, y) = bodypart;
        grid.0[*x as usize][*y as usize] = 0xFF;
    }

    grid
}
