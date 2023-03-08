use crate::control::{Game, GameControlArg};
use crate::matrix::{GameState, Grid, State, HEIGHT, WIDTH};

#[derive(Clone, Copy)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[derive(Clone)]
pub struct GameOfLifeState {
    cells: [[Cell; WIDTH]; HEIGHT],
}

pub fn start_game(state: &mut State, _random: u8) {
    let gol = GameOfLifeState::new();
    state.grid = gol.draw_matrix();
    state.game = Some(GameState::GameOfLife(gol));
}
pub fn handle_control(state: &mut State, arg: &GameControlArg) {
    if let Some(GameState::GameOfLife(ref mut gol_state)) = state.game {
        match arg {
            //GameControlArg::Exit => {}
            _ => {
                // TODO
            }
        }
    }
}
pub fn game_step(state: &mut State, _random: u8) {
    if let Some(GameState::GameOfLife(ref mut gol_state)) = state.game {
        gol_state.tick();
        state.grid = gol_state.draw_matrix();
    } else {
        panic!("Game of Life not started!")
    }
}

impl GameOfLifeState {
    fn pattern1() -> Self {
        // Starts off with lots of alive cells, quickly reduced.
        // Eventually reaches a stable pattern without changes.
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                let i = col * HEIGHT + row;
                if i % 2 == 0 || i % 7 == 0 {
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
        cells[10][5] = Cell::Alive;
        cells[10][6] = Cell::Alive;
        cells[10][7] = Cell::Alive;
        cells[14][5] = Cell::Alive;
        cells[14][6] = Cell::Alive;
        cells[14][7] = Cell::Alive;
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
        cells[10][4] = Cell::Alive;
        cells[10][5] = Cell::Alive;
        cells[10][6] = Cell::Alive;
        cells[11][5] = Cell::Alive;
        cells[11][6] = Cell::Alive;
        cells[11][7] = Cell::Alive;
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
        cells[10][4] = Cell::Alive;
        cells[10][5] = Cell::Alive;
        cells[11][4] = Cell::Alive;
        cells[11][5] = Cell::Alive;

        cells[12][6] = Cell::Alive;
        cells[12][7] = Cell::Alive;
        cells[13][6] = Cell::Alive;
        cells[13][7] = Cell::Alive;
        GameOfLifeState { cells }
    }

    pub fn new() -> Self {
        // TODO: Allow selection between patterns
        Self::pattern1()
        //Self::blinker()
        //Self::toad()
        //Self::beacon()
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
        let mut next_generation = self.cells.clone();

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

    fn draw_matrix(&self) -> Grid {
        let mut grid = Grid::default();

        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                grid.0[col][row] = (self.cells[row][col] as u8) * 0xFF;
            }
        }

        grid
    }
}
