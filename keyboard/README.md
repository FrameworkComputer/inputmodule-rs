# Keyboard

NOT THE OFFICIAL Keyboard firmware
Just experimental reference code for building keyboard firmare in Rust

Basic typing is implemented for ANSI, ISO, JIS, numpad and macropad using the
matrix positions and default keymaps in QMK's `keyboards/framework`.
Modifiers and simultaneous held keys use USB boot-keyboard reports (six ordinary
keys plus modifiers). Keys are debounced for 5 ms; the host handles key repeat.
The numpad and macropad send keypad usages, with Num Lock handled by the host.

On ANSI/ISO, hold Fn for F1–F12. JIS sends F1–F12 without Fn, matching QMK.
Fn+arrows sends Home/Page Up/Page Down/End, Fn+Delete sends Insert,
Fn+P/B sends Pause/Break, and Fn+K sends Scroll Lock.
Fn+Esc toggles Fn lock for the top row until reset. Keys keep the meaning they
had when pressed until released, even if Fn is released first.
Down is now a normal arrow key rather than a bootloader shortcut.

Media, brightness, airplane mode, calculator, and lighting shortcuts are not
implemented. Print Screen is supported. Existing basic RGB lighting remains.

Build one variant at a time (`macropad` is the default):

```sh
cargo build -p keyboard --release --no-default-features --features ansi
```

The pure input state machine can be tested without RP2040 hardware:

```sh
rustc --edition=2021 --test keyboard/src/keymap.rs -o target/keyboard-keymap-tests
./target/keyboard-keymap-tests
```

To regenerate the layout tables from a QMK checkout:

```sh
python keyboard/generate_keymaps.py ../qmk_firmware
cargo fmt -p keyboard
```

Hardware validation is still needed for scan timing, resistor thresholds,
simultaneous keys, and suspend/resume on each variant.
