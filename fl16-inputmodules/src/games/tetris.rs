use crate::control::GameControlArg;
use crate::matrix::{GameState, Grid, LedmatrixState, HEIGHT, WIDTH};

use crate::animations::{Animation, StartupPercentageIterator};

type Position = (i8, i8);

/*
Square:
██ ██
██ ██

LLeft:
██ ██
   ██
   ██

LRight:
██ ██
██
██

T:
██ ██ ██
   ██

I:
██
██
██
██

ZLeft:
██ ██
   ██ ██

ZRight:
   ██ ██
██ ██

*/

#[derive(Clone, Copy, Eq, PartialEq)]
enum Tetromino {
    None = 0,
    Square,
    LLeft,
    LRight,
    T,
    I,
    ZLeft,
    ZRight,
}

fn get_tetromino_from_num(num: u8) -> Tetromino {
    match num {
        0 => {
            return Tetromino::None;
        }
        1 => {
            return Tetromino::Square;
        }
        2 => {
            return Tetromino::LLeft;
        }
        3 => {
            return Tetromino::LRight;
        }
        4 => {
            return Tetromino::T;
        }
        5 => {
            return Tetromino::I;
        }
        6 => {
            return Tetromino::ZLeft;
        }
        7 => {
            return Tetromino::ZRight;
        }
        _ => { Tetromino::None }
    }
}

#[derive(Clone)]
pub struct TetrisState {
    placed_tetrominos: [[u8; HEIGHT]; WIDTH],
    game_over: bool,
    current_tetromino: Tetromino,
    rotation: u8,
    position: Position,
    moved_this_tick: bool,
}

fn get_tetromino(tetromino: Tetromino, rotation: u8) -> [Position; 4] {
    let mut result: [Position; 4] = [(0,0);4];
    match tetromino {
        Tetromino::Square => {
            result[0] = (0, 0);
            result[1] = (0, 1);
            result[2] = (-1, 0);
            result[3] = (-1, 1);
        }
        Tetromino::LLeft => match rotation {
            0 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (0, -1);
                result[3] = (1, -1);
            }
            1 => {
                result[0] = (0, 0);
                result[1] = (1, 0);
                result[2] = (-1, 0);
                result[3] = (-1, -1);
            }
            2 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (0, -1);
                result[3] = (-1, 1);
            }
            3 => {
                result[0] = (0, 0);
                result[1] = (1, 0);
                result[2] = (-1, 0);
                result[3] = (1, 1);
            }
            _ => {}
        },
        Tetromino::LRight => match rotation {
            0 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (0, -1);
                result[3] = (-1, -1);
            }
            1 => {
                result[0] = (0, 0);
                result[1] = (1, 0);
                result[2] = (-1, 0);
                result[3] = (-1, 1);
            }
            2 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (0, -1);
                result[3] = (1, 1);
            }
            3 => {
                result[0] = (0, 0);
                result[1] = (1, 0);
                result[2] = (-1, 0);
                result[3] = (1, -1);
            }
            _ => {}
        },
        Tetromino::T => match rotation {
            0 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (1, 0);
                result[3] = (-1, 0);
            }
            1 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (1, 0);
                result[3] = (0, -1);
            }
            2 => {
                result[0] = (0, 0);
                result[1] = (0, -1);
                result[2] = (1, 0);
                result[3] = (-1, 0);
            }
            3 => {
                result[0] = (0, 0);
                result[1] = (0, 1);
                result[2] = (-1, 0);
                result[3] = (0, -1);
            }
            _ => {}
        },
        Tetromino::I => match rotation {
            0 | 2 => {
                result[0] = (0, 0);
                result[1] = (0, -1);
                result[2] = (0, 1);
                result[3] = (0, 2);
            }
            1 | 3 => {
                result[0] = (0, 0);
                result[1] = (1, 0);
                result[2] = (-1, 0);
                result[3] = (-2, 0);
            }
            _ => {}
        },
        Tetromino::ZLeft => match rotation {
            0 | 2 => {
                result[0] = (0, 0);
                result[1] = (1, 0);
                result[2] = (0, 1);
                result[3] = (-1, 1);
            }
            1 | 3 => {
                result[0] = (0, 0);
                result[1] = (0, -1);
                result[2] = (1, 0);
                result[3] = (1, 1);
            }
            _ => {}
        },
        Tetromino::ZRight => match rotation {
            0 | 2 => {
                result[0] = (0, 0);
                result[1] = (-1, 0);
                result[2] = (0, 1);
                result[3] = (1, 1);
            }
            1 | 3 => {
                result[0] = (0, 0);
                result[1] = (0, -1);
                result[2] = (-1, 0);
                result[3] = (-1, 1);
            }
            _ => {}
        },
        _ => {}
    }
    result
}

impl TetrisState {
    pub fn new(random: u8) -> Self {
        TetrisState {
            placed_tetrominos: [[0; 34]; 9],
            game_over: false,
            current_tetromino: get_tetromino_from_num((random % 7) + 1),
            rotation: random % 4,
            position: (4, 1),
            moved_this_tick: false,
        }
    }

    pub fn tick(&mut self, random: u8) {
        if self.game_over {
            return;
        }
        self.moved_this_tick = false;
        let tetronimo = get_tetromino(self.current_tetromino, self.rotation);
        let mut can_fall: bool = true;
        for i in 0..4 {
            let x = self.position.0 + tetronimo[i].0;
            let y = self.position.1 + tetronimo[i].1 + 1;
            if y >= 34 {
                can_fall = false;
                break;
            }
            if self.placed_tetrominos[x as usize][y as usize] != 0 {
                can_fall = false;
                break;
            }
        }
        if can_fall {
            self.position.1 += 1;
        } else {
            for i in 0..4 {
                let x = self.position.0 + tetronimo[i].0;
                let y = self.position.1 + tetronimo[i].1;
                if self.placed_tetrominos[x as usize][y as usize] == 1{
                    self.game_over = true;
                }
                self.placed_tetrominos[x as usize][y as usize] = 1;
            }
            if self.game_over{
                return;
            }
            self.position = (4, 1);
            self.rotation = random % 4;
            self.current_tetromino = get_tetromino_from_num((random % 7) + 1);

            let mut cleared_rows: u8 = 0;
            for y in (0..34).rev() {
                let mut filled = true;
                for x in 0..9 {
                    if self.placed_tetrominos[x][y] == 0 {
                        filled = false;
                        break;
                    }
                }

                if filled {
                    cleared_rows += 1;
                } else {
                    if cleared_rows == 0 {
                        continue;
                    }
                    for x in 0..9 {
                        self.placed_tetrominos[x][y + cleared_rows as usize] = self.placed_tetrominos[x][y];
                        self.placed_tetrominos[x][y] = 0;
                    }
                }
            }
        }
    }

    pub fn draw_matrix(&self) -> Grid {
        let mut grid: Grid = Grid::default();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                grid.0[x][y] = (self.placed_tetrominos[x][y]) * 0xFF;
            }
        }
        let tetronimo = get_tetromino(self.current_tetromino, self.rotation);
        for i in 0..4 {
            let x = self.position.0 + tetronimo[i].0;
            let y = self.position.1 + tetronimo[i].1;
            grid.0[x as usize][y as usize] = 0xff;
        }
        grid
    }

    pub fn handle_control(&mut self, arg: &GameControlArg) {
        let tetromino = get_tetromino(self.current_tetromino, self.rotation);
        match arg {
            GameControlArg::Left => {
                if self.moved_this_tick {
                    return;
                }
                let mut can_move: bool = true;
                for i in 0..4 {
                    if self.position.0 + tetromino[i].0 == 8 {
                        can_move = false;
                        break;
                    }
                }
                if can_move {
                    self.position.0 += 1;
                    self.moved_this_tick = true;
                }
            }
            GameControlArg::Right => {
                if self.moved_this_tick {
                    return;
                }
                let mut can_move: bool = true;
                for i in 0..4 {
                    if self.position.0 + tetromino[i].0 == 0 {
                        can_move = false;
                        break;
                    }
                }
                if can_move {
                    self.position.0 -= 1;
                    self.moved_this_tick = true;
                }
            },
            GameControlArg::Up => {
                let tetromino = get_tetromino(self.current_tetromino, (self.rotation+1)%4);
                let mut can_rotate = true;
                for i in 0..4{
                    let x = self.position.0 + tetromino[i].0;
                    let y = self.position.1 + tetromino[i].1;
                    if x < 0 || x >= 9 || y >= 34{
                        can_rotate = false;
                        break;
                    }
                    if self.placed_tetrominos[x as usize][y as usize] == 1{
                        can_rotate = false;
                        break;
                    }
                }
                if can_rotate{
                    self.rotation = (self.rotation + 1) % 4;
                }
            }
            _ => {}
        }
    }
}

pub fn start_game(state: &mut LedmatrixState, random: u8) {
    state.game = Some(GameState::Tetris(TetrisState::new(random)));
}

pub fn handle_control(state: &mut LedmatrixState, arg: &GameControlArg) {
    if let Some(GameState::Tetris(ref mut tetris_state)) = state.game {
        match arg {
            GameControlArg::Exit => state.game = None,
            _ => tetris_state.handle_control(arg),
        }
    }
}

pub fn game_step(state: &mut LedmatrixState, random: u8) {
    if let Some(GameState::Tetris(ref mut tetris_state)) = state.game {
        if !tetris_state.game_over {
            tetris_state.tick(random);
            if tetris_state.game_over{
                state.upcoming_frames = Some(Animation::Percentage(StartupPercentageIterator::default()));
            }
        }
        state.grid = tetris_state.draw_matrix();
    }
}