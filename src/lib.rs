#![no_std]

#[cfg(feature = "ledmatrix")]
pub mod games;
#[cfg(feature = "ledmatrix")]
pub mod lotus;
#[cfg(feature = "ledmatrix")]
pub mod lotus_led_hal;
#[cfg(feature = "ledmatrix")]
pub mod mapping;
#[cfg(feature = "ledmatrix")]
pub mod matrix;
#[cfg(feature = "ledmatrix")]
pub mod patterns;

#[cfg(feature = "b1display")]
pub mod graphics;
#[cfg(feature = "b1display")]
pub mod lotus_lcd_hal;

pub mod control;
pub mod serialnum;
