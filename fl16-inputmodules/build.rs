//! Generates the lookup table for converting sRGB greyscale values to linear light.
//! Floating point math isn't available in const fns and is slow on the Cortex-M0+.

use std::env;
use std::fs;
use std::path::Path;

/// Linear light at 100%. Fine enough to resolve the darkest levels
const LINEAR_ONE: u32 = 1 << 24;

/// The sRGB transfer function, from 0-1 sRGB to 0-1 linear light
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut out = String::new();
    out += "/// Linear light at 100%\n";
    out += &format!("pub const LINEAR_ONE: u64 = {LINEAR_ONE};\n\n");
    out += "/// Linear light of each sRGB greyscale value, out of LINEAR_ONE\n";
    out += "pub const SRGB_TO_LINEAR: [u32; 256] = [\n";
    for value in 0..=255 {
        let linear = srgb_to_linear(value as f64 / 255.0);
        out += &format!("    {},\n", (linear * LINEAR_ONE as f64).round() as u32);
    }
    out += "];\n";

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("srgb_to_linear.rs");
    fs::write(path, out).unwrap();
}
