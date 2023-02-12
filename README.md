# Lotus LED Matrix Module

Project setup based off of: https://github.com/rp-rs/rp2040-project-template

## Control

Requirements: Python and [PySimpleGUI](https://www.pysimplegui.org).

Use `control.py`. Either the commandline, see `control.py --help` or the graphical version: `control.py --gui`

## Building

Dependencies: Rust

Prepare Rust toolchain:

```sh
rustup target install thumbv6m-none-eabi
cargo install flip-link
cargo install elf2uf2-rs --locked
```


Build:

```sh
cargo build
```

Generate UF2 file:

```sh
elf2uf2-rs target/thumbv6m-none-eabi/debug/led_matrix_fw led_matrix.uf2
```

## Flashing

First, put the module into bootloader mode, which will expose a filesystem

This can be done by pressing the bootsel button while plugging it in.

```sh
cargo run
```

## Panic

On panic the RP2040 resets itself into bootloader mode.
This means a new firmware can be written to overwrite the old one.

Additionally the panic message is written to flash, which can be read as follows:

```sh
sudo picotool save -r 0x15000000 0x15004000 message.bin
strings message.bin | head
```
