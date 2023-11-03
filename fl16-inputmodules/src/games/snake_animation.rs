use crate::control::GameControlArg;
use crate::games::snake::SnakeState;
use crate::matrix::Grid;

pub struct SnakeIterator {
    state: SnakeState,
    commands: [(Option<GameControlArg>, u8); 64],
    current_command: usize,
}

impl SnakeIterator {
    pub fn new(random: u8) -> Self {
        Self {
            state: SnakeState::new(random),
            commands: SAMPLE_GAME,
            current_command: 0,
        }
    }
}

impl Iterator for SnakeIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_command >= self.commands.len() {
            return None;
        }
        if self.state.game_over {
            return None;
        }

        let (maybe_cmd, random) = self.commands[self.current_command];
        if let Some(command) = maybe_cmd {
            self.state.handle_control(&command);
        }
        self.current_command += 1;

        self.state.tick(random);
        Some(self.state.draw_matrix())
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
    (None, 0),
    (Some(GameControlArg::Right), 0),
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
