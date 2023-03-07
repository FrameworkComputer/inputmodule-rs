use crate::control::GameControlArg;
use crate::matrix::{GameState, Grid, State, HEIGHT, WIDTH};

const PADDLE_WIDTH: usize = 5;

#[derive(Clone)]
struct Score {
    _upper: u8,
    _lower: u8,
}

type Position = (usize, usize);
type Velocity = (i8, i8);

#[derive(Clone)]
struct Ball {
    pos: Position,
    // Not a position, more like a directional vector
    direction: Velocity,
}

#[derive(Clone)]
pub struct PongState {
    // TODO: Properly calculate score and display it
    _score: Score,
    ball: Ball,
    paddles: (usize, usize),
    pub speed: u64,
}

pub fn start_game(state: &mut State, _random: u8) {
    state.game = Some(GameState::Pong(PongState {
        _score: Score {
            _upper: 0,
            _lower: 0,
        },
        ball: Ball {
            pos: (4, 20),
            direction: (0, 1),
        },
        paddles: (PADDLE_WIDTH / 2, PADDLE_WIDTH / 2),
        speed: 0,
    }))
}
pub fn handle_control(state: &mut State, arg: &GameControlArg) {
    if let Some(GameState::Pong(ref mut pong_state)) = state.game {
        match arg {
            GameControlArg::Left => {
                if pong_state.paddles.0 + PADDLE_WIDTH < WIDTH {
                    pong_state.paddles.0 += 1;
                }
            }
            GameControlArg::Right => {
                if pong_state.paddles.0 >= 1 {
                    pong_state.paddles.0 -= 1;
                }
            }
            GameControlArg::SecondLeft => {
                if pong_state.paddles.1 + PADDLE_WIDTH < WIDTH {
                    pong_state.paddles.1 += 1;
                }
            }
            GameControlArg::SecondRight => {
                if pong_state.paddles.1 >= 1 {
                    pong_state.paddles.1 -= 1;
                }
            }
            //GameControlArg::Exit => {
            _ => {
                // TODO
            }
        }
    }
}

// TODO: Randomize the velocity vector upon respawning
fn _random_v(random: u8) -> Velocity {
    // TODO: while food == head:
    let x = ((random & 0xF0) >> 4) % WIDTH as u8;
    let y = (random & 0x0F) % HEIGHT as u8;
    (x as i8, y as i8)
}

fn add_velocity(pos: Position, v: Velocity) -> Position {
    let (vx, vy) = v;
    let (x, y) = pos;
    (((x as i8) + vx) as usize, ((y as i8) + vy) as usize)
}

fn hit_paddle(ball: Position, paddles: (usize, usize)) -> Option<usize> {
    let (x, y) = ball;
    if y == 1 && paddles.0 <= x && x <= paddles.0 + PADDLE_WIDTH {
        Some(((paddles.0 as i32) - (x as i32)).unsigned_abs() as usize)
    } else if y == HEIGHT - 2 && paddles.1 <= x && x <= paddles.1 + PADDLE_WIDTH {
        Some(((paddles.1 as i32) - (x as i32)).unsigned_abs() as usize)
    } else {
        None
    }
}

pub fn game_step(state: &mut State, _random: u8) {
    if let Some(GameState::Pong(ref mut pong_state)) = state.game {
        pong_state.ball.pos = {
            let (vx, vy) = pong_state.ball.direction;
            let (x, y) = add_velocity(pong_state.ball.pos, pong_state.ball.direction);
            let x = if x > WIDTH - 1 { WIDTH - 1 } else { x };
            if x == 0 || x == WIDTH - 1 {
                // Hit wall, bounce back
                pong_state.ball.direction = (-vx, vy);
            }

            let y = if y > HEIGHT - 1 { HEIGHT - 1 } else { y };
            let (x, y) = if let Some(paddle_hit) = hit_paddle((x, y), pong_state.paddles) {
                // Hit paddle, bounce back
                // TODO: Change vy direction slightly depending on where the paddle was hit
                let (vx, vy) = pong_state.ball.direction;
                pong_state.ball.direction = match paddle_hit {
                    0 => (vx - 2, -vy),
                    1 => (vx - 1, -vy),
                    2 => (vx, -vy),
                    3 => (vx + 1, -vy),
                    4 => (vx + 2, -vy),
                    // Shouldn't occur
                    _ => (vx, -vy),
                };
                // TODO: Not sure if I want the speed to change. Speed by angle change is already high enough
                //pong_state.speed += 1;
                (x, y)
            } else if y == 0 || y == HEIGHT - 1 {
                pong_state.speed = 0;
                pong_state.ball.direction = (1, 1); //random_v(random);
                (WIDTH / 2, HEIGHT / 2)
            } else {
                (x, y)
            };
            (x, y)
        };
        state.grid = draw_matrix(pong_state);
    }
}

fn draw_matrix(state: &PongState) -> Grid {
    let mut grid = Grid::default();

    for x in state.paddles.0..state.paddles.0 + PADDLE_WIDTH {
        grid.0[x][0] = 0xFF;
    }
    for x in state.paddles.1..state.paddles.1 + PADDLE_WIDTH {
        grid.0[x][HEIGHT - 1] = 0xFF;
    }
    grid.0[state.ball.pos.0][state.ball.pos.1] = 0xFF;

    grid
}
