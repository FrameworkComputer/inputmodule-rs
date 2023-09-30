#![allow(clippy::needless_range_loop)]
#![no_std]

#[cfg(any(
    all(feature = "ledmatrix", feature = "b1display"),
    all(feature = "ledmatrix", feature = "c1minimal"),
    all(feature = "b1display", feature = "c1minimal"),
    all(feature = "ledmatrix", feature = "keyboard"),
    all(feature = "b1display", feature = "keyboard"),
    all(feature = "c1minimal", feature = "keyboard"),
))]
compile_error!(
    "Features \"ledmatrix\", \"b1display\", \"c1minimal\", and \"keyboard\" are mutually exclusive"
);

#[cfg(feature = "ledmatrix")]
pub mod fl16;
#[cfg(feature = "ledmatrix")]
pub mod games;
#[cfg(feature = "ledmatrix")]
pub mod led_hal;
#[cfg(feature = "ledmatrix")]
#[rustfmt::skip]
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

#[cfg(feature = "keyboard")]
pub mod keyboard_hal;

pub mod control;
pub mod serialnum;
