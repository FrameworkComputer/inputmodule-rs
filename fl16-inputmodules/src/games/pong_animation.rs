use crate::control::GameControlArg;
use crate::games::pong::PongState;
use crate::matrix::Grid;

pub struct PongIterator {
    state: PongState,
    commands: [Option<GameControlArg>; 64],
    current_command: usize,
}

impl PongIterator {
    pub fn new() -> Self {
        PongIterator {
            state: PongState::new(),
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

// TODO: Plan out a nice looking game
const SAMPLE_GAME: [Option<GameControlArg>; 64] = [
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
