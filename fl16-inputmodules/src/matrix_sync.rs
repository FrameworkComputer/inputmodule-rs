use crate::control::PwmFreqArg;
use crate::led_hal as bsp;
use crate::matrix::{LedmatrixState, HEIGHT, WIDTH};
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

/// 0xB4 LEDs on the first page, 0xAB on the second page
const PWM_BUF_LEN: usize = 0xB4 + 0xAB;

pub struct MatrixSync {
    matrix: Foo,
    /// PWM values last sent to the LED controller. None if unknown
    sent: Option<[u8; PWM_BUF_LEN]>,
}

impl MatrixSync {
    pub fn new(matrix: Foo) -> Self {
        Self { matrix, sent: None }
    }

    /// Fills the grid pixels but only sends the I2C commands if the PWM values changed
    pub fn flush_if_dirty(&mut self, state: &LedmatrixState) {
        let brightnesses = calc_brightnesses(state, &self.matrix);
        if self.sent.as_ref() != Some(&brightnesses) {
            self.matrix.device.fill_matrix(&brightnesses).unwrap();
            self.sent = Some(brightnesses);
        }
    }

    /// Sends the full grid, even if the controller should already have it.
    /// Recovers from the controller losing or corrupting its state.
    pub fn force_flush(&mut self, state: &LedmatrixState) {
        self.sent = None;
        self.flush_if_dirty(state);
    }

    pub fn set_pwm_freq(&mut self, freq: PwmFreqArg) {
        self.matrix.device.set_pwm_freq(freq.into()).unwrap();
    }
}

/// Calculate PWM values for the entire grid, so they can be sent with just two I2C commands
fn calc_brightnesses(state: &LedmatrixState, matrix: &Foo) -> [u8; PWM_BUF_LEN] {
    let mut brightnesses = [0x00; PWM_BUF_LEN];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let (register, page) = (matrix.device.calc_pixel)(x as u8, y as u8);
            brightnesses[(page as usize) * 0xB4 + (register as usize)] =
                ((state.grid.0[x][y] as u64) * (state.brightness as u64)
                    / (BRIGHTNESS_LEVELS as u64)) as u8;
        }
    }
    brightnesses
}
