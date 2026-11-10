use crate::control::PwmFreqArg;
use crate::gamma::{srgb_levels, LedLevel};
use crate::led_hal as bsp;
use crate::matrix::{LedmatrixState, HEIGHT, WIDTH};
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
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

// LED controller registers, which the is31fl3741 crate doesn't expose
const PAGE_SELECT_REGISTER: u8 = 0xFD;
const CONFIG_LOCK_REGISTER: u8 = 0xFE;
const CONFIG_WRITE_ENABLE: u8 = 0xC5;
/// Per-LED current scaling, same layout as the PWM values in pages 0 and 1
const SCALE_PAGE_1: u8 = 0x02;
const SCALE_PAGE_2: u8 = 0x03;
const CONFIG_PAGE: u8 = 0x04;
/// Scales the current of all LEDs, in the config page
const GLOBAL_CURRENT_REGISTER: u8 = 0x01;

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
    /// Times the LED controller was powered on
    pub led_power_ons: u32,
    /// Time the LED controller was powered off in microseconds
    pub led_off_us_total: u32,
    /// Times the per-LED scaling was sent to the LED controller over I2C
    pub scale_flushes: u32,
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
            2 => [
                self.led_power_ons,
                self.led_off_us_total,
                self.scale_flushes,
                0,
                0,
                0,
                0,
                0,
            ],
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
    /// Whether the LED controller may be powered on. False while sleeping
    enabled: bool,
    /// Whether the LED controller is currently powered on
    powered: bool,
    /// When the LED controller was powered off, or when its off time was last counted
    off_since: u64,
    /// PWM values last sent to the LED controller. None if unknown
    sent_pwm: Option<[u8; PWM_BUF_LEN]>,
    /// Per-LED scaling last sent to the LED controller. None if unknown
    sent_scale: Option<[u8; PWM_BUF_LEN]>,
    /// Global current last sent to the LED controller. None if unknown
    sent_current: Option<u8>,
    /// Per-LED scaling at full current. Limits how much current the LEDs draw
    max_scale: u8,
    /// How to drive an LED for each greyscale value, with gamma correction
    srgb_levels: [LedLevel; 256],
    /// Index into the PWM buffer of each pixel.
    /// Calculated once, because calc_pixel builds its whole lookup table on every call.
    pwm_index: [[u16; HEIGHT]; WIDTH],
    timer: Timer,
    pub stats: Stats,
}

impl MatrixSync {
    /// max_scale is the per-LED scaling at full current, which limits the total current draw
    pub fn new(matrix: Foo, led_enable: LedEnable, timer: Timer, max_scale: u8) -> Self {
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
            // The LED controller must already be on, to set it up
            enabled: true,
            powered: true,
            off_since: 0,
            sent_pwm: None,
            sent_scale: None,
            sent_current: None,
            max_scale,
            srgb_levels: srgb_levels(max_scale),
            pwm_index,
            timer,
            stats: Stats::default(),
        }
    }

    /// Fills the grid pixels but only sends the I2C commands if the PWM values changed.
    /// Powers the LED controller off while no LED is lit and back on when one is.
    pub fn flush_if_dirty(&mut self, state: &LedmatrixState) {
        let calc_start = self.timer.get_counter().ticks();
        let (pwm, scale) = self.calc_frame(state);
        let calc_elapsed = (self.timer.get_counter().ticks() - calc_start) as u32;
        self.stats.calcs = self.stats.calcs.wrapping_add(1);
        self.stats.calc_us_total = self.stats.calc_us_total.wrapping_add(calc_elapsed);
        self.stats.calc_us_max = self.stats.calc_us_max.max(calc_elapsed);

        let lit = state.brightness != 0 && pwm.iter().any(|&b| b != 0);
        self.set_powered(self.enabled && lit);
        if !self.powered {
            return;
        }

        let start = self.timer.get_counter().ticks();
        let mut flushed = false;
        // Brightness scales the current instead of the PWM values,
        // so that all 256 PWM levels are available at every brightness
        if self.sent_current != Some(state.brightness) {
            self.write_config_register(GLOBAL_CURRENT_REGISTER, state.brightness);
            self.sent_current = Some(state.brightness);
            flushed = true;
        }
        if self.sent_scale.as_ref() != Some(&scale) {
            self.write_scale(&scale);
            self.sent_scale = Some(scale);
            self.stats.scale_flushes = self.stats.scale_flushes.wrapping_add(1);
            flushed = true;
        }
        if self.sent_pwm.as_ref() != Some(&pwm) {
            self.matrix.device.fill_matrix(&pwm).unwrap();
            self.sent_pwm = Some(pwm);
            flushed = true;
        }
        if flushed {
            let elapsed = (self.timer.get_counter().ticks() - start) as u32;
            self.stats.flushes = self.stats.flushes.wrapping_add(1);
            self.stats.flush_us_total = self.stats.flush_us_total.wrapping_add(elapsed);
            self.stats.flush_us_max = self.stats.flush_us_max.max(elapsed);
        }
    }

    /// Calculate the PWM and scaling values for the entire grid,
    /// so each can be sent with just two I2C commands
    fn calc_frame(&self, state: &LedmatrixState) -> ([u8; PWM_BUF_LEN], [u8; PWM_BUF_LEN]) {
        let mut pwm = [0x00; PWM_BUF_LEN];
        let mut scale = [self.max_scale; PWM_BUF_LEN];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let index = self.pwm_index[x][y] as usize;
                let value = state.grid.0[x][y];
                if state.gamma {
                    let level = self.srgb_levels[value as usize];
                    pwm[index] = level.pwm;
                    scale[index] = level.scale;
                } else {
                    pwm[index] = value;
                }
            }
        }
        (pwm, scale)
    }

    /// Write the per-LED scaling of all LEDs
    fn write_scale(&mut self, scale: &[u8; PWM_BUF_LEN]) {
        // The first byte is the register to start writing at
        let mut buf = [0x00; 0xB4 + 1];
        buf[1..].copy_from_slice(&scale[..0xB4]);
        self.write_page(SCALE_PAGE_1, &buf);
        buf[1..=0xAB].copy_from_slice(&scale[0xB4..]);
        self.write_page(SCALE_PAGE_2, &buf[..=0xAB]);
    }

    /// Write a register in the config page of the LED controller
    fn write_config_register(&mut self, register: u8, value: u8) {
        self.write_page(CONFIG_PAGE, &[register, value]);
    }

    /// Write to a page of the LED controller.
    /// The first byte is the register to start at, the rest are the values.
    fn write_page(&mut self, page: u8, data: &[u8]) {
        let device = &mut self.matrix.device;
        let address = device.address;
        device
            .i2c
            .write(address, &[CONFIG_LOCK_REGISTER, CONFIG_WRITE_ENABLE])
            .unwrap();
        device
            .i2c
            .write(address, &[PAGE_SELECT_REGISTER, page])
            .unwrap();
        device.i2c.write(address, data).unwrap();
    }

    /// Sends the full grid, even if the controller should already have it.
    /// Recovers from the controller losing or corrupting its state.
    pub fn force_flush(&mut self, state: &LedmatrixState) {
        self.forget_sent();
        self.flush_if_dirty(state);
    }

    /// Allow or prevent powering the LED controller on, e.g. while sleeping.
    /// Even when enabled, it's only powered on while an LED is lit, after the next flush.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.set_powered(false);
        }
    }

    fn set_powered(&mut self, powered: bool) {
        if powered == self.powered {
            return;
        }
        self.count_off_time();
        if powered {
            self.led_enable.set_high().unwrap();
            // It may not have kept or accepted register values while off
            self.forget_sent();
            self.stats.led_power_ons = self.stats.led_power_ons.wrapping_add(1);
        } else {
            self.led_enable.set_low().unwrap();
        }
        self.powered = powered;
    }

    /// Send everything again on the next flush
    fn forget_sent(&mut self) {
        self.sent_pwm = None;
        self.sent_scale = None;
        self.sent_current = None;
    }

    /// Add the time the LED controller has been off since the last call to the stats
    fn count_off_time(&mut self) {
        let now = self.timer.get_counter().ticks();
        if !self.powered {
            let off_us = (now - self.off_since) as u32;
            self.stats.led_off_us_total = self.stats.led_off_us_total.wrapping_add(off_us);
        }
        self.off_since = now;
    }

    /// Get a page of the stats, see [Stats::take_response]
    pub fn take_stats(&mut self, page: u8) -> [u8; 32] {
        self.count_off_time();
        self.stats.take_response(page)
    }

    pub fn set_pwm_freq(&mut self, freq: PwmFreqArg) {
        // Make sure the controller accepts the write. Powered off again on the next flush, if dark
        self.set_powered(true);
        self.matrix.device.set_pwm_freq(freq.into()).unwrap();
    }
}
