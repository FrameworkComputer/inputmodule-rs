// Taken from https://github.com/phip1611/max-7219-led-matrix-util/blob/main/src/mappings.rs

/// We have 8 rows and 8 bits per row.
pub type SingleDisplayData = [u8; 8];

/// Capital letter A
pub const CAP_A: SingleDisplayData = [
    0b00010000,
    0b00101000,
    0b00101000,
    0b01000100,
    0b01111100,
    0b01000100,
    0b01000100,
    0b01000100,
];
/// Capital letter B
pub const CAP_B: SingleDisplayData = [
    0b01111000,
    0b01000100,
    0b01000100,
    0b01111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01111000,
];
/// Capital letter C
pub const CAP_C: SingleDisplayData = [
    0b01111100,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01111100,
];
/// Capital letter D
pub const CAP_D: SingleDisplayData = [
    0b01111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01111000,
];
/// Capital letter E
pub const CAP_E: SingleDisplayData = [
    0b01111100,
    0b01000000,
    0b01000000,
    0b01111100,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01111100,
];
/// Capital letter F
pub const CAP_F: SingleDisplayData = [
    0b01111100,
    0b01000000,
    0b01000000,
    0b01111100,
    0b01000000,
    0b01000000,
    0b01000000,
    0b01000000,
];
/// Capital letter G
pub const CAP_G: SingleDisplayData = [
    0b01111000,
    0b11000100,
    0b10000100,
    0b10000000,
    0b10011100,
    0b10000100,
    0b11000100,
    0b01111100,
];
/// Capital letter H
pub const CAP_H: SingleDisplayData = [
    0b01000100,
    0b01000100,
    0b01000100,
    0b01111100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
];
/// Capital letter I
pub const CAP_I: SingleDisplayData = [
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
];
/// Capital letter J
pub const CAP_J: SingleDisplayData = [
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b10010000,
    0b01100000,
];
/// Capital letter K
pub const CAP_K: SingleDisplayData = [
    0b01000100,
    0b01001000,
    0b01010000,
    0b01100000,
    0b01010000,
    0b01001000,
    0b01000100,
    0b01000010,
];
/// Capital letter L
/// I shifted it one left
pub const CAP_L: SingleDisplayData = [
    0b10000000,
    0b10000000,
    0b10000000,
    0b10000000,
    0b10000000,
    0b10000000,
    0b10000000,
    0b11111000,
];
/// Capital letter M
pub const CAP_M: SingleDisplayData = [
    0b10000010,
    0b11000110,
    0b10101010,
    0b10111010,
    0b10010010,
    0b10000010,
    0b10000010,
    0b10000010,
];
/// Capital letter N
pub const CAP_N: SingleDisplayData = [
    0b01000100,
    0b01100100,
    0b01110100,
    0b01010100,
    0b01011100,
    0b01001100,
    0b01001100,
    0b01000100,
];
/// Capital letter O
pub const CAP_O: SingleDisplayData = [
    0b00011000,
    0b00100100,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b00100100,
    0b00011000,
];
/// Capital letter P
pub const CAP_P: SingleDisplayData = [
    0b01111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01111000,
    0b01000000,
    0b01000000,
    0b01000000,
];
/// Capital letter Q
pub const CAP_Q: SingleDisplayData = [
    0b00011000,
    0b00100100,
    0b01000010,
    0b01000010,
    0b01001010,
    0b01000110,
    0b00100110,
    0b00011001,
];
/// Capital letter R
pub const CAP_R: SingleDisplayData = [
    0b01111000,
    0b01000100,
    0b01000100,
    0b01111000,
    0b01100000,
    0b01010000,
    0b01001000,
    0b01000100,
];
/// Capital letter S
/// I shifted it one to the right
pub const CAP_S: SingleDisplayData = [
    0b00000111,
    0b00001000,
    0b00010000,
    0b00001100,
    0b00000010,
    0b00000001,
    0b00000001,
    0b00011110,
];
/// Capital letter T
pub const CAP_T: SingleDisplayData = [
    0b11111110,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
];
/// Capital letter U
pub const CAP_U: SingleDisplayData = [
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b01000010,
    0b00111100,
];
/// Capital letter V
pub const CAP_V: SingleDisplayData = [
    0b10000001,
    0b10000001,
    0b10000001,
    0b10000001,
    0b10000010,
    0b01000100,
    0b00101000,
    0b00010000,
];
/// Capital letter W
pub const CAP_W: SingleDisplayData = [
    0b10000010,
    0b10010010,
    0b11010110,
    0b01010100,
    0b01111100,
    0b00110000,
    0b00010000,
    0b00000000,
];
/// Capital letter X
pub const CAP_X: SingleDisplayData = [
    0b00000000,
    0b10000010,
    0b01000100,
    0b00101000,
    0b00010000,
    0b00101000,
    0b01000100,
    0b10000010,
];
/// Capital letter Y
pub const CAP_Y: SingleDisplayData = [
    0b01000100,
    0b01000100,
    0b00101000,
    0b00101000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
];
/// Capital letter Z
pub const CAP_Z: SingleDisplayData = [
    0b01111110,
    0b00000010,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b01000000,
    0b01111110,
];
/// Number 0
pub const ZERO: SingleDisplayData = [
    0b00111000,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b01000100,
    0b00111000,
];
/// Number 1
pub const ONE: SingleDisplayData = [
    0b00000100,
    0b00011100,
    0b00000100,
    0b00000100,
    0b00000100,
    0b00000100,
    0b00000100,
    0b00000100,
];
/// " " character
pub const SPACE: SingleDisplayData = [0; 8];
/// "." character
pub const DOT: SingleDisplayData = [0, 0, 0, 0, 0, 0, 0, 0b00010000];
/// "!" character
pub const EXCLAMATION_MARK: SingleDisplayData = [
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00010000,
    0b00000000,
    0b00010000,
];
pub const HASH: SingleDisplayData = [
    0b00100100,
    0b00100111,
    0b00111100,
    0b11100100,
    0b00100111,
    0b00111100,
    0b11100100,
    0b00100100,
];
