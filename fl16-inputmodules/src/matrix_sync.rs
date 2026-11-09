use crate::control::PwmFreqArg;
use crate::led_hal as bsp;
use crate::matrix::{LedmatrixState, HEIGHT, WIDTH};
use crate::patterns::BRIGHTNESS_LEVELS;
use embedded_hal::digital::OutputPin;
use is31fl3741::devices::LedMatrix;
use rp2040_hal::{
    gpio::{
        bank0::{Gpio26, Gpio27, Gpio29},
        FunctionI2C, FunctionSioOutput, Pin, PullDown, PullUp,
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

/// SDB pin of the LED controller. Low shuts it down
pub type LedEnable = Pin<Gpio29, FunctionSioOutput, PullDown>;

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
    /// Main loop iterations
    pub loops: u32,
    /// Main loop iterations that didn't receive any USB data
    pub idle_loops: u32,
    /// Time spent in main loop iterations that didn't receive any USB data in microseconds
    pub idle_us_total: u32,
    /// Time spent in main loop iterations that received USB data in microseconds
    pub busy_us_total: u32,
    /// Times the PWM values were calculated
    pub calcs: u32,
    /// Time spent calculating PWM values in microseconds
    pub calc_us_total: u32,
    /// Longest time spent calculating PWM values once in microseconds
    pub calc_us_max: u32,
    /// Time the CPU spent asleep waiting for an event in microseconds
    pub wait_us_total: u32,
}

impl Stats {
    /// Serialize a page as little-endian u32s
    ///
    /// Each page resets the max values it contains.
    pub fn take_response(&mut self, page: u8) -> [u8; 32] {
        let vals = match page {
            0 => {
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
                self.flush_us_max = 0;
                self.loop_us_max = 0;
                vals
            }
            1 => {
                let vals = [
                    self.loops,
                    self.idle_loops,
                    self.idle_us_total,
                    self.busy_us_total,
                    self.calcs,
                    self.calc_us_total,
                    self.calc_us_max,
                    self.wait_us_total,
                ];
                self.calc_us_max = 0;
                vals
            }
            _ => [0; 8],
        };
        let mut response = [0; 32];
        for (i, val) in vals.iter().enumerate() {
            response[i * 4..i * 4 + 4].copy_from_slice(&val.to_le_bytes());
        }
        response
    }
}

pub struct MatrixSync {
    matrix: Foo,
    led_enable: LedEnable,
    /// PWM values last sent to the LED controller. None if unknown
    sent: Option<[u8; PWM_BUF_LEN]>,
    /// Index into the PWM buffer of each pixel.
    /// Calculated once, because calc_pixel builds its whole lookup table on every call.
    pwm_index: [[u16; HEIGHT]; WIDTH],
    timer: Timer,
    pub stats: Stats,
}

impl MatrixSync {
    pub fn new(matrix: Foo, led_enable: LedEnable, timer: Timer) -> Self {
        let mut pwm_index = [[0; HEIGHT]; WIDTH];
        for (x, col) in pwm_index.iter_mut().enumerate() {
            for (y, index) in col.iter_mut().enumerate() {
                let (register, page) = (matrix.device.calc_pixel)(x as u8, y as u8);
                *index = (page as u16) * 0xB4 + (register as u16);
            }
        }
        Self {
            matrix,
            led_enable,
            sent: None,
            pwm_index,
            timer,
            stats: Stats::default(),
        }
    }

    /// Fills the grid pixels but only sends the I2C commands if the PWM values changed
    pub fn flush_if_dirty(&mut self, state: &LedmatrixState) {
        let calc_start = self.timer.get_counter().ticks();
        let brightnesses = calc_brightnesses(state, &self.pwm_index);
        let calc_elapsed = (self.timer.get_counter().ticks() - calc_start) as u32;
        self.stats.calcs = self.stats.calcs.wrapping_add(1);
        self.stats.calc_us_total = self.stats.calc_us_total.wrapping_add(calc_elapsed);
        self.stats.calc_us_max = self.stats.calc_us_max.max(calc_elapsed);

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

    /// Turn the LED controller on or off
    pub fn set_powered(&mut self, powered: bool) {
        if powered {
            self.led_enable.set_high().unwrap();
        } else {
            self.led_enable.set_low().unwrap();
        }
    }

    pub fn set_pwm_freq(&mut self, freq: PwmFreqArg) {
        self.matrix.device.set_pwm_freq(freq.into()).unwrap();
    }
}

/// Calculate PWM values for the entire grid, so they can be sent with just two I2C commands
fn calc_brightnesses(
    state: &LedmatrixState,
    pwm_index: &[[u16; HEIGHT]; WIDTH],
) -> [u8; PWM_BUF_LEN] {
    let mut brightnesses = [0x00; PWM_BUF_LEN];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // u32 is plenty for 255 * 255 and avoids slow 64-bit division on the Cortex-M0+
            brightnesses[pwm_index[x][y] as usize] =
                ((state.grid.0[x][y] as u32) * (state.brightness as u32)
                    / (BRIGHTNESS_LEVELS as u32)) as u8;
        }
    }
    brightnesses
}
