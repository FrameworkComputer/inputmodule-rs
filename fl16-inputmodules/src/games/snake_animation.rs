use crate::control::GameControlArg;
use crate::games::snake::SnakeState;
use crate::matrix::Grid;

pub struct SnakeIterator {
    state: SnakeState,
    commands: [(Option<GameControlArg>, u8); 64],
    current_tick: usize,
}

impl SnakeIterator {
    pub fn new(random: u8) -> Self {
        Self {
            state: SnakeState::new(random),
            commands: SAMPLE_GAME,
            current_tick: 0,
        }
    }
}
impl Default for SnakeIterator {
    fn default() -> Self {
        Self::new(31)
    }
}

impl Iterator for SnakeIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_tick / 4 >= self.commands.len() {
            return None;
        }

        // Slow down animation by a factor of 4
        if self.current_tick.is_multiple_of(4) {
            let (maybe_cmd, random) = self.commands[self.current_tick / 4];
            if let Some(command) = maybe_cmd {
                self.state.handle_control(&command);
            }
            self.state.tick(random);
        }
        self.current_tick += 1;

        if self.state.game_over {
            None
        } else {
            Some(self.state.draw_matrix())
        }
    }
}

// TODO: Plan out a nice looking game
const SAMPLE_GAME: [(Option<GameControlArg>, u8); 64] = [
    (Some(GameControlArg::Down), 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (Some(GameControlArg::Left), 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (Some(GameControlArg::Down), 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (Some(GameControlArg::Right), 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (Some(GameControlArg::Down), 0),
    (None, 0),
    (None, 10),
    (None, 0),
    (None, 0),
    (Some(GameControlArg::Right), 0),
    (Some(GameControlArg::Up), 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (Some(GameControlArg::Left), 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
    (None, 0),
];
