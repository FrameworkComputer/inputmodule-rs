use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let gamma_path = Path::new(&out_dir).join("gamma.rs");
    let mut f = File::create(gamma_path).unwrap();

    // Determined empirically for a PVT panel. May need to become conditional
    // on features, TBD.
    const GAMMA: f32 = 3.2;

    let corrected: [f32; 256] =
        std::array::from_fn(|i| f32::powf((i as f32) / 255., GAMMA) * 255. + 0.5);

    writeln!(f, "const GAMMA: [u8; 256] = [").unwrap();

    const LINE_LEN: usize = 8;
    for line in corrected.chunks(LINE_LEN) {
        write!(f, "   ").unwrap();
        for element in line {
            write!(f, " {:>3},", *element as u8).unwrap();
        }
        writeln!(f).unwrap();
    }
    writeln!(f, "];").unwrap();
}
