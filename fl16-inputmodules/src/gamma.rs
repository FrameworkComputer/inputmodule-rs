//! Gamma correction for the LED matrix
//!
//! With gamma correction, greyscale values are treated as sRGB, like in images,
//! and converted to linear light, which is what the LEDs emit.
//! Linear light needs much finer steps near black than the 8-bit PWM values have,
//! so for the darkest levels the LED's current is scaled down as well.
//! Light is proportional to PWM * scaling, which together have about 14 bits.

// Defines SRGB_TO_LINEAR and LINEAR_ONE
include!(concat!(env!("OUT_DIR"), "/srgb_to_linear.rs"));

/// Lowest PWM value to use at full current.
/// Darker levels scale the current down instead, so the PWM value stays around here.
///
/// Higher is more accurate, rounding the PWM value to at most 1/(2 * PWM_KNEE) off.
/// But it scales the current of more levels, so more changes have to send the scaling
/// over I2C as well, which takes as long as the PWM values.
/// Very low PWM values and very low currents may both make some LEDs look uneven.
const PWM_KNEE: u64 = 32;

/// How to drive an LED
#[derive(Clone, Copy)]
pub struct LedLevel {
    pub pwm: u8,
    pub scale: u8,
}

/// Calculate how to drive an LED for each sRGB greyscale value
///
/// max_scale is the scaling at full brightness.
pub fn srgb_levels(max_scale: u8) -> [LedLevel; 256] {
    let mut levels = [LedLevel {
        pwm: 0,
        scale: max_scale,
    }; 256];
    let max_scale = max_scale as u64;
    for value in 1..levels.len() {
        // Light in units of PWM * scaling / LINEAR_ONE, so it can be compared without dividing
        let target = SRGB_TO_LINEAR[value] as u64 * 255 * max_scale;
        let light = |level: &LedLevel| level.pwm as u64 * level.scale as u64 * LINEAR_ONE;
        let error = |level: &LedLevel| light(level).abs_diff(target);
        // The closest PWM value at this scaling
        let with_scale = |scale: u64| {
            let unit = scale * LINEAR_ONE;
            LedLevel {
                pwm: ((target + unit / 2) / unit).clamp(1, 255) as u8,
                scale: scale as u8,
            }
        };

        // Full current, unless that makes the PWM value lower than the knee.
        // Integer division rounds the scaling down, the next one up might be closer.
        let scale = (target / (PWM_KNEE * LINEAR_ONE)).clamp(1, max_scale);
        let lower = with_scale(scale);
        let upper = with_scale((scale + 1).min(max_scale));
        let mut level = if error(&upper) < error(&lower) {
            upper
        } else {
            lower
        };

        // Rounding shouldn't make it darker than the level below
        if light(&level) < light(&levels[value - 1]) {
            level = levels[value - 1];
        }
        levels[value] = level;
    }
    levels
}
