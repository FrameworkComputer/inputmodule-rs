use crate::control::{GameControlArg, GameOfLifeStartParam};
use crate::matrix::{GameState, Grid, LedmatrixState, HEIGHT, WIDTH};

#[derive(Clone, Copy, num_derive::FromPrimitive, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[derive(Clone)]
pub struct GameOfLifeState {
    cells: [[Cell; WIDTH]; HEIGHT],
}

impl GameOfLifeState {
    pub fn combine(&self, other: &Self) -> Self {
        let mut state = self.clone();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                if other.cells[y][x] == Cell::Alive {
                    state.cells[y][x] = Cell::Alive;
                }
            }
        }
        state
    }
}

pub fn start_game(state: &mut LedmatrixState, _random: u8, param: GameOfLifeStartParam) {
    let gol = GameOfLifeState::new(param, &state.grid);
    state.grid = gol.draw_matrix();
    state.game = Some(GameState::GameOfLife(gol));
}
pub fn handle_control(state: &mut LedmatrixState, arg: &GameControlArg) {
    if let Some(GameState::GameOfLife(ref mut _gol_state)) = state.game {
        if let GameControlArg::Exit = arg {
            state.game = None
        }
    }
}
pub fn game_step(state: &mut LedmatrixState, _random: u8) {
    if let Some(GameState::GameOfLife(ref mut gol_state)) = state.game {
        gol_state.tick();
        state.grid = gol_state.draw_matrix();
    } else {
        panic!("Game of Life not started!")
    }
}

impl GameOfLifeState {
    // TODO: Integrate Grid into GameOfLifeStartParam because it's only used in one of the enum variants
    pub fn new(param: GameOfLifeStartParam, grid: &Grid) -> Self {
        match param {
            GameOfLifeStartParam::Beacon => Self::beacon(),
            GameOfLifeStartParam::CurrentMatrix => {
                let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
                for row in 0..HEIGHT {
                    for col in 0..WIDTH {
                        cells[row][col] = if grid.0[col][row] == 0 {
                            Cell::Dead
                        } else {
                            Cell::Alive
                        };
                    }
                }
                //cells: grid
                //    .0
                //    .map(|col| col.map(|val| if val == 0 { Cell::Dead } else { Cell::Alive })),
                GameOfLifeState { cells }
            }
            GameOfLifeStartParam::Pattern1 => Self::pattern1(),
            GameOfLifeStartParam::Blinker => Self::blinker(),
            GameOfLifeStartParam::Toad => Self::toad(),
            GameOfLifeStartParam::Glider => Self::glider(),
            GameOfLifeStartParam::BeaconToadBlinker => Self::beacon()
                .combine(&Self::toad())
                .combine(&Self::blinker()),
        }
    }
    fn pattern1() -> Self {
        // Starts off with lots of alive cells, quickly reduced.
        // Eventually reaches a stable pattern without changes.
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                let i = col * HEIGHT + row;
                if i.is_multiple_of(2) || i.is_multiple_of(7) {
                    cells[row][col] = Cell::Alive;
                }
            }
        }
        GameOfLifeState { cells }
    }
    fn blinker() -> Self {
        // Oscillates between:
        //     XXX
        // and
        //      X
        //      X
        //      X
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        cells[4][5] = Cell::Alive;
        cells[4][6] = Cell::Alive;
        cells[4][7] = Cell::Alive;
        cells[8][5] = Cell::Alive;
        cells[8][6] = Cell::Alive;
        cells[8][7] = Cell::Alive;
        GameOfLifeState { cells }
    }
    fn toad() -> Self {
        // Oscillates between
        //  XXX
        // XXX
        // and
        //   X
        // X  X
        // X  X
        //  X
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        cells[17][4] = Cell::Alive;
        cells[17][5] = Cell::Alive;
        cells[17][6] = Cell::Alive;
        cells[18][5] = Cell::Alive;
        cells[18][6] = Cell::Alive;
        cells[18][7] = Cell::Alive;
        GameOfLifeState { cells }
    }
    fn beacon() -> Self {
        // Oscillates between
        //   XX
        //   XX
        // XX
        // XX
        // and
        //   XX
        //    X
        // X
        // XX
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        cells[26][4] = Cell::Alive;
        cells[26][5] = Cell::Alive;
        cells[27][4] = Cell::Alive;
        cells[27][5] = Cell::Alive;

        cells[28][6] = Cell::Alive;
        cells[28][7] = Cell::Alive;
        cells[29][6] = Cell::Alive;
        cells[29][7] = Cell::Alive;
        GameOfLifeState { cells }
    }

    fn glider() -> Self {
        //  X
        //   X
        // XXX
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        cells[2][3] = Cell::Alive;
        cells[3][4] = Cell::Alive;
        cells[4][2] = Cell::Alive;
        cells[4][3] = Cell::Alive;
        cells[4][4] = Cell::Alive;

        cells[20][5] = Cell::Alive;
        cells[21][6] = Cell::Alive;
        cells[22][4] = Cell::Alive;
        cells[22][5] = Cell::Alive;
        cells[22][6] = Cell::Alive;
        GameOfLifeState { cells }
    }

    /// Count live neighbor cells
    pub fn live_neighbor_count(&self, row: usize, col: usize) -> u8 {
        let mut count = 0;
        // Use HEIGHT-1 instead of -1 because usize can't go below 0
        for delta_row in [HEIGHT - 1, 0, 1] {
            for delta_col in [WIDTH - 1, 0, 1] {
                if delta_row == 0 && delta_col == 0 {
                    // The cell itself
                    continue;
                }

                let neighbor_row = (row + delta_row) % HEIGHT;
                let neighbor_col = (col + delta_col) % WIDTH;

                count += self.cells[neighbor_row][neighbor_col] as u8;
            }
        }
        count
    }
    pub fn tick(&mut self) {
        let mut next_generation = self.cells;

        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                let cell = self.cells[row][col];
                let live_neighbors = self.live_neighbor_count(row, col);

                let child_cell = match (cell, live_neighbors) {
                    // Fewer than 2 neighbors causes it to die
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // 2 or three neighbors are good and it stays alive
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // More than 3 is too many and the cell dies
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // 3 neighbors when the cell is dead, revives it
                    (Cell::Dead, 3) => Cell::Alive,
                    // No change by default
                    (c, _) => c,
                };

                next_generation[row][col] = child_cell;
            }
        }

        self.cells = next_generation;
    }

    pub fn draw_matrix(&self) -> Grid {
        let mut grid = Grid::default();

        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                grid.0[col][row] = (self.cells[row][col] as u8) * 0xFF;
            }
        }

        grid
    }
}
