use crate::control::PwmFreqArg;
use crate::games::game_of_life::GameOfLifeState;
use crate::games::pong::PongState;
use crate::games::snake::SnakeState;

use core::ops::Deref;
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use serde::{Deserialize, Serialize};
use rp2040_flash::flash;

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 34;
pub const LEDS: usize = WIDTH * HEIGHT;

//#[derive(Clone, Copy, Debug, PartialEq, Eq)]
//pub struct Col( pub [u8; HEIGHT]);
//#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
//pub struct Grid(
//    pub [Col; WIDTH],
//);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid(pub Vec<Vec<u8, HEIGHT>, WIDTH>);
impl Default for Grid {
    fn default() -> Self {
        let mut vec: Vec<Vec<u8, HEIGHT>, WIDTH> = Vec::new();
        for _ in 0..WIDTH {
            let mut col = Vec::new();
            for _ in 0..HEIGHT {
                col.push(0).unwrap();
            }
            vec.push(col).unwrap();
        }
        Grid(vec)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct LedmatrixState {
    pub rev: u16,
    pub grid: Grid,
    #[serde(skip)]
    pub col_buffer: Grid,
    pub animate: bool,
    pub brightness: u8,
    pub sleeping: SleepState,
    #[serde(skip)]
    pub game: Option<GameState>,
    pub animation_period: u64,
    pub pwm_freq: PwmFreqArg,
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

impl LedmatrixState {
    pub fn save(&self) {
        //let foo: Vec<u8, {core::mem::size_of::<Grid>()}> = to_vec(&self.grid).unwrap();
        let foo: &[u8] = unsafe {any_as_u8_slice(&self.grid)};
        let mut data: Vec<u8, 4096> = Vec::new();
        for _ in 0..4096 {
            data.push(0);
        }
        data[0..{core::mem::size_of::<Grid>()}].copy_from_slice(foo);
        let addr = 0xfe000; // 2nd to last 4K sector
        cortex_m::interrupt::free(|_cs| {
            unsafe {flash::flash_range_erase_and_program(addr, &data, true)};
        });
    }
    pub fn restore() -> Grid {
        let addr = 0xfe000 + 0x10000000; // 2nd to last 4K sector
        unsafe {
            //let bytes = *(&*(addr as *const &[u8; 4096]));
            //let g: *const Grid = (bytes as *const u8) as *const Grid;
            //let g = &*(addr as *const &Grid);
            //(*g).clone()
            //from_bytes(bytes).unwrap()
        }
        Grid::default()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SleepState {
    Awake,
    Sleeping((Grid, u8)),
}
//impl Default for SleepState {
//    fn default() -> Self {
//        SleepState::Awake
//    }
//}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Snake(SnakeState),
    Pong(PongState),
    GameOfLife(GameOfLifeState),
}
