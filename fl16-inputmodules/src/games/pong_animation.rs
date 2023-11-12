use crate::control::GameControlArg;
use crate::games::pong::PongState;
use crate::matrix::Grid;

pub struct PongIterator {
    state: PongState,
    commands: [Option<GameControlArg>; 136],
    current_command: usize,
}

impl Default for PongIterator {
    fn default() -> Self {
        PongIterator {
            state: PongState::default(),
            commands: SAMPLE_GAME,
            current_command: 0,
        }
    }
}

impl Iterator for PongIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_command >= self.commands.len() {
            return None;
        }

        if let Some(command) = self.commands[self.current_command] {
            self.state.handle_control(&command);
        }
        self.current_command += 1;

        self.state.tick();
        Some(self.state.draw_matrix())
    }
}

const SAMPLE_GAME: [Option<GameControlArg>; 136] = [
    Some(GameControlArg::Left), // Middle
    None,
    Some(GameControlArg::Left),
    None,
    None,
    None,
    Some(GameControlArg::SecondRight),
    Some(GameControlArg::SecondRight),
    None,
    None,
    None,
    Some(GameControlArg::SecondLeft),
    None, // hit and bounce back
    None,
    None,
    None,
    Some(GameControlArg::Right),
    None,
    None,
    None,
    None,
    None,
    None,
    Some(GameControlArg::Right),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(GameControlArg::SecondLeft),
    None,
    Some(GameControlArg::SecondLeft),
    None,
    Some(GameControlArg::SecondRight),
    None,
    None,
    Some(GameControlArg::SecondLeft),
    None,
    None,
    Some(GameControlArg::Right),
    None,
    None,
    Some(GameControlArg::Left),
    None,
    Some(GameControlArg::Right),
    None,
    None,
    Some(GameControlArg::Left),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(GameControlArg::Right),
    Some(GameControlArg::Right),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];
