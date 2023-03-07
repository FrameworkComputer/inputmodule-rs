use crate::games::pong::PongState;
use crate::games::snake::SnakeState;

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 34;
pub const LEDS: usize = WIDTH * HEIGHT;

#[derive(Clone)]
pub struct Grid(pub [[u8; HEIGHT]; WIDTH]);
impl Default for Grid {
    fn default() -> Self {
        Grid([[0; HEIGHT]; WIDTH])
    }
}

pub struct State {
    pub grid: Grid,
    pub col_buffer: Grid,
    pub animate: bool,
    pub brightness: u8,
    pub sleeping: SleepState,
    pub game: Option<GameState>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum SleepState {
    Awake,
    Sleeping(Grid),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum GameState {
    Snake(SnakeState),
    Pong(PongState),
}
