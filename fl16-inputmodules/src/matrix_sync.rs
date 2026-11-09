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
    Timer,
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

/// Performance counters, read by the host with the Stats command.
///
/// Counters wrap around, the host should only look at the difference between two reads.
/// The max values are reset after every read.
#[derive(Default)]
pub struct Stats {
    /// Bytes received over USB
    pub bytes_received: u32,
    /// Commands that were successfully parsed
    pub cmds_parsed: u32,
    /// USB reads that contained data but no valid command
    pub cmds_rejected: u32,
    /// Frames committed by Draw or DrawGreyColBuffer
    pub frames_committed: u32,
    /// Frames actually sent to the LED controller over I2C
    pub flushes: u32,
    /// Time spent sending frames over I2C in microseconds
    pub flush_us_total: u32,
    /// Longest time spent sending a single frame over I2C in microseconds
    pub flush_us_max: u32,
    /// Longest main loop iteration in microseconds
    pub loop_us_max: u32,
}

impl Stats {
    /// Serialize as little-endian u32s and reset the max values
    pub fn take_response(&mut self) -> [u8; 32] {
        let vals = [
            self.bytes_received,
            self.cmds_parsed,
            self.cmds_rejected,
            self.frames_committed,
            self.flushes,
            self.flush_us_total,
            self.flush_us_max,
            self.loop_us_max,
        ];
        let mut response = [0; 32];
        for (i, val) in vals.iter().enumerate() {
            response[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
        }
        self.flush_us_max = 0;
        self.loop_us_max = 0;
        response
    }
}

pub struct MatrixSync {
    matrix: Foo,
    /// PWM values last sent to the LED controller. None if unknown
    sent: Option<[u8; PWM_BUF_LEN]>,
    timer: Timer,
    pub stats: Stats,
}

impl MatrixSync {
    pub fn new(matrix: Foo, timer: Timer) -> Self {
        Self {
            matrix,
            sent: None,
            timer,
            stats: Stats::default(),
        }
    }

    /// Fills the grid pixels but only sends the I2C commands if the PWM values changed
    pub fn flush_if_dirty(&mut self, state: &LedmatrixState) {
        let brightnesses = calc_brightnesses(state, &self.matrix);
        if self.sent.as_ref() != Some(&brightnesses) {
            let start = self.timer.get_counter().ticks();
            self.matrix.device.fill_matrix(&brightnesses).unwrap();
            let elapsed = (self.timer.get_counter().ticks() - start) as u32;
            self.sent = Some(brightnesses);

            self.stats.flushes = self.stats.flushes.wrapping_add(1);
            self.stats.flush_us_total = self.stats.flush_us_total.wrapping_add(elapsed);
            self.stats.flush_us_max = self.stats.flush_us_max.max(elapsed);
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
