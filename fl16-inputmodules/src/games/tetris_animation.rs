use crate::control::GameControlArg;
use crate::games::tetris::TetrisState;
use crate::matrix::Grid;

pub struct TetrisIterator {
    state: TetrisState,
    commands: [(Option<GameControlArg>, u8); 64],
    current_tick: usize,
}

impl Default for TetrisIterator {
    fn default() -> Self{ Self::new(27)}
}

impl TetrisIterator {
    pub fn new(random: u8) -> Self {
        Self {
            state: TetrisState::new(random),
            commands: SAMPLE_GAME,
            current_tick: 0,
        }
    }
}

impl Iterator for TetrisIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_tick / 4 >= self.commands.len() {
            return None;
        }

        let (maybe_cmd, random) = self.commands[self.current_tick/4];
        if let Some(command) = maybe_cmd {
            self.state.handle_control(&command);
        }
        self.state.tick(random);
        self.current_tick += 1;
        Some(self.state.draw_matrix())
    }
}

const SAMPLE_GAME: [(Option<GameControlArg>, u8); 64] = [
    (Some(GameControlArg::Up), 31),
    (None, 2),
    (None, 255),
    (None, 10),
    (None, 6),
    (Some(GameControlArg::Left), 50),
    (Some(GameControlArg::Left), 200),
    (None, 27),
    (None, 0),
    (Some(GameControlArg::Up), 83),
    (None, 240),
    (None, 50),
    (None, 7),
    (None, 56),
    (None, 4),
    (None, 3),
    (Some(GameControlArg::Right), 2),
    (None, 1),
    (None, 7),
    (None, 6),
    (None, 78),
    (None, 123),
    (None, 45),
    (Some(GameControlArg::Up), 0),
    (None, 67),
    (None, 89),
    (None, 10),
    (None, 11),
    (Some(GameControlArg::Right), 20),
    (Some(GameControlArg::Right), 09),
    (Some(GameControlArg::Right), 12),
    (None, 83), //
    (None, 101),
    (None, 99),
    (None, 114),
    (None, 101),
    (None, 116),
    (None, 32),
    (None, 77),
    (None, 101),
    (None, 115),
    (None, 115),
    (None, 97),
    (None, 104),
    (None, 101),
    (Some(GameControlArg::Left), 33),
    (Some(GameControlArg::Right), 75),
    (None, 43),
    (None, 9),
    (None, 87),
    (None, 36),
    (None, 99),
    (None, 100),
    (None, 200),
    (None, 45),
    (None, 54),
    (None, 130),
    (None, 145),
    (None, 150),
    (None, 89),
    (None, 56),
    (None, 0),
    (None, 1),
    (None, 2),
];
