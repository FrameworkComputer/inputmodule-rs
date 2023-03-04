use crate::control::GameControlArg;
use crate::matrix::{GameState, Grid, State, HEIGHT, WIDTH};

//use heapless::Vec;

const PADDLE_WIDTH: usize = 4;

type PieceShape = [u8; 4 * 4];
type Position = (usize, usize);

#[rustfmt::skip]
const PIECES: [PieceShape; 7] = [
    [
        1, 1, 1, 1,
        0, 0, 0, 0,
        0, 0, 0, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        0, 1, 1, 1,
        0, 1, 0, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        1, 1, 1, 0,
        0, 0, 1, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        1, 1, 1, 0,
        0, 1, 0, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        0, 1, 1, 0,
        0, 1, 1, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        1, 1, 0, 0,
        0, 1, 1, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        0, 1, 1, 0,
        1, 1, 0, 0,
        0, 0, 0, 0,
    ]
];

#[derive(Clone)]
struct Piece {
    position: Position,
    rotation: u8,
    shape: PieceShape,
}

#[derive(Clone)]
pub struct TetrisState {
    score: usize,
    grid: Grid,
    piece: Piece,
    command: Option<GameControlArg>,
}

pub fn start_game(state: &mut State, random: u8) {
    state.game = Some(GameState::Tetris(TetrisState {
        score: 0,
        grid: Grid::default(),
        piece: Piece {
            position: (4, 0),
            rotation: 0,
            shape: PIECES[1],
        },
        command: None,
    }))
}

pub fn handle_control(state: &mut State, arg: &GameControlArg) {
    if let Some(GameState::Tetris(ref mut tetris_state)) = state.game {
        // Let the piece sink each time there's a down press
        let (x, y) = tetris_state.piece.position;
        match tetris_state.command {
            Some(GameControlArg::Down) => {
                tetris_state.piece.position = (x, y + 1);
            }
            Some(GameControlArg::Up) => {
                tetris_state.piece.rotation += 1;
            }
            Some(GameControlArg::Left) => {
                if x + 1 < WIDTH {
                    tetris_state.piece.position = (x + 1, y);
                }
            }
            Some(GameControlArg::Right) => {
                if x > 0 {
                    tetris_state.piece.position = (x - 1, y);
                }
            }
            Some(_) => {}
            None => {}
        }

        tetris_state.command = Some(*arg);
    }
}
pub fn game_step(state: &mut State, random: u8) {
    if let Some(GameState::Tetris(ref mut tetris_state)) = state.game {
        let (x, y) = tetris_state.piece.position;
        match tetris_state.command {
            _ => {}
        }
        tetris_state.command = None;

        let (x, y) = tetris_state.piece.position;
        tetris_state.piece.position = (x, y + 1);

        state.grid = draw_matrix(&tetris_state);

        // TODO: Check if the piece has settled and we can
        if false {
            // Check if there's a full filled row and we can shift everything down
            for y in (0..HEIGHT).rev() {
                let mut row_filled = 0;
                for x in 0..9 {
                    if state.grid.0[x][y] == 0xFF {
                        row_filled += 1;
                    }
                }
                if row_filled == 9 {
                    // TODO: remove the row and shift everything down
                    tetris_state.score += 1;
                }
            }

            // Once the pieces have settled, we save this grid
            tetris_state.grid = state.grid.clone();
        }
    }
}

fn draw_matrix(tetris_state: &TetrisState) -> Grid {
    let mut grid = tetris_state.grid.clone();

    let pos = tetris_state.piece.position;
    for x in 0..4 {
        for y in 0..4 {
            grid.0[x + pos.0][y + pos.1] = tetris_state.piece.shape[x * 4 + y] * 0xFF;
        }
    }

    grid
}
