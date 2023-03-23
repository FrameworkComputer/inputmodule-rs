#![allow(clippy::needless_range_loop)]
#![no_std]

#[cfg(feature = "ledmatrix")]
pub mod games;
#[cfg(feature = "ledmatrix")]
pub mod fl16;
#[cfg(feature = "ledmatrix")]
pub mod led_hal;
#[cfg(feature = "ledmatrix")]
pub mod mapping;
#[cfg(feature = "ledmatrix")]
pub mod matrix;
#[cfg(feature = "ledmatrix")]
pub mod patterns;

#[cfg(feature = "b1display")]
pub mod graphics;
#[cfg(feature = "b1display")]
pub mod lcd_hal;

#[cfg(feature = "c1minimal")]
pub mod minimal_hal;

pub mod control;
pub mod serialnum;
