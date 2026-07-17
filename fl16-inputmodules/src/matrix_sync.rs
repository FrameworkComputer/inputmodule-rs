use crate::control::PwmFreqArg;
use crate::led_hal as bsp;
use crate::matrix::{Grid, LedmatrixState, HEIGHT, WIDTH};
use crate::patterns::BRIGHTNESS_LEVELS;
use is31fl3741::devices::LedMatrix;
use rp2040_hal::{
    gpio::{
        bank0::{Gpio26, Gpio27},
        FunctionI2C, PullUp,
    },
    pac::I2C1,
};

pub type Foo = LedMatrix<
    bsp::hal::I2C<
        I2C1,
        (
            bsp::hal::gpio::Pin<Gpio26, FunctionI2C, PullUp>,
            bsp::hal::gpio::Pin<Gpio27, FunctionI2C, PullUp>,
        ),
    >,
>;

pub struct MatrixSync {
    matrix: Foo,
    sent_grid: Grid,
    sent_brightness: u8,
}

impl MatrixSync {
    pub fn new(matrix: Foo) -> Self {
        Self {
            matrix,
            sent_grid: Grid::default(),
            sent_brightness: 0,
        }
    }

    /// Fills the grid pixels but only sends the I2C commands if we expect it to update displayed state
    pub fn flush_if_dirty(&mut self, state: &LedmatrixState) {
        if state.grid != self.sent_grid || state.brightness != self.sent_brightness {
            fill_grid_pixels(state, &mut self.matrix);
            self.sent_grid = state.grid.clone();
            self.sent_brightness = state.brightness;
        }
    }

    pub fn set_pwm_freq(&mut self, freq: PwmFreqArg) {
        self.matrix.device.set_pwm_freq(freq.into()).unwrap();
    }
}

/// Just sends two I2C commands for the entire grid
fn fill_grid_pixels(state: &LedmatrixState, matrix: &mut Foo) {
    // 0xB4 LEDs on the first page, 0xAB on the second page
    let mut brightnesses = [0x00; 0xB4 + 0xAB];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let (register, page) = (matrix.device.calc_pixel)(x as u8, y as u8);
            brightnesses[(page as usize) * 0xB4 + (register as usize)] =
                ((state.grid.0[x][y] as u64) * (state.brightness as u64)
                    / (BRIGHTNESS_LEVELS as u64)) as u8;
        }
    }
    matrix.device.fill_matrix(&brightnesses).unwrap();
}

/// Same as fill_grid_pixels but does each pixel individually
/// So it's much slower because it has to send 306 I2C commands
pub fn _fill_grid(grid: &Grid, matrix: &mut Foo) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            matrix.device.pixel(x as u8, y as u8, grid.0[x][y]).unwrap();
        }
    }
}

pub fn full_brightness(matrix: &mut Foo) {
    // Fills every pixel individually
    //matrix.fill_brightness(0xFF).unwrap();

    // Fills full page at once
    matrix.device.fill(0xFF).unwrap();
}
