// Matrix thresholds and Fn layout based on QMK keyboards/framework.
// Copyright 2022-2023 Framework Computer
// SPDX-License-Identifier: GPL-2.0-or-later
//! Basic keyboard usages and momentary Fn. No hardware dependencies so the
//! input state machine can also be tested on the host with rustc --test.
pub const ROWS: usize = 8;
pub const COLS: usize = 16;
pub type Matrix = [[bool; COLS]; ROWS];
pub type Keymap = [[u8; COLS]; ROWS];
const FN: u8 = 0xff;
const DEBOUNCE_US: u64 = 5_000;

#[path = "layouts.rs"]
pub mod layouts;

pub struct Keys {
    raw: Matrix,
    stable: Matrix,
    changed_at: [[u64; COLS]; ROWS],
    // Bind on press: releasing Fn must not change a key that is still held.
    bindings: Keymap,
    fn_lock: bool,
}

impl Keys {
    pub const fn new() -> Self {
        Self {
            raw: [[false; COLS]; ROWS],
            stable: [[false; COLS]; ROWS],
            changed_at: [[0; COLS]; ROWS],
            bindings: [[0; COLS]; ROWS],
            fn_lock: false,
        }
    }

    pub fn update(&mut self, matrix: Matrix, now: u64, layout: &Keymap) {
        let previous = self.stable;
        for row in 0..ROWS {
            for col in 0..COLS {
                if matrix[row][col] != self.raw[row][col] {
                    self.raw[row][col] = matrix[row][col];
                    self.changed_at[row][col] = now;
                }
                if now.saturating_sub(self.changed_at[row][col]) >= DEBOUNCE_US {
                    self.stable[row][col] = self.raw[row][col];
                }
            }
        }
        let fn_down = (0..ROWS).any(|r| (0..COLS).any(|c| self.stable[r][c] && layout[r][c] == FN));
        if fn_down
            && (0..ROWS).any(|r| {
                (0..COLS).any(|c| self.stable[r][c] && !previous[r][c] && layout[r][c] == 0x29)
            })
        {
            self.fn_lock = !self.fn_lock;
        }
        for row in 0..ROWS {
            for col in 0..COLS {
                if !self.stable[row][col] {
                    self.bindings[row][col] = 0;
                } else if !previous[row][col] {
                    let base = layout[row][col];
                    self.bindings[row][col] = if base == FN || (fn_down && base == 0x29) {
                        0
                    } else if let Some(index) = function_index(row, col) {
                        // ANSI/ISO have an omitted media row at rest; JIS has F1-F12.
                        let jis = layout[3][5] == 0x3a;
                        if fn_down ^ self.fn_lock ^ jis {
                            0x3a + index
                        } else if jis {
                            // Only Print Screen uses the keyboard usage page;
                            // the other media/system actions are unsupported.
                            if index == 10 {
                                0x46
                            } else {
                                0
                            }
                        } else {
                            base
                        }
                    } else if fn_down {
                        match base {
                            0x4c => 0x49, // Delete -> Insert
                            0x50 => 0x4a, // Left -> Home
                            0x4f => 0x4d, // Right -> End
                            0x52 => 0x4b, // Up -> Page Up
                            0x51 => 0x4e, // Down -> Page Down
                            0x13 => 0x48, // P -> Pause
                            0x0e => 0x47, // K -> Scroll Lock
                            0x05 => 0x48, // B -> Break (same HID usage as Pause)
                            _ => base,
                        }
                    } else {
                        base
                    };
                }
            }
        }
    }

    pub fn pressed(&self) -> impl Iterator<Item = u8> + '_ {
        let mut seen = [false; 256];
        self.bindings.iter().flatten().copied().filter(move |&key| {
            let duplicate = seen[key as usize];
            seen[key as usize] = true;
            key != 0 && !duplicate
        })
    }

    pub fn any_pressed(&self) -> bool {
        self.stable.iter().flatten().any(|&pressed| pressed)
    }
}

fn function_index(row: usize, col: usize) -> Option<u8> {
    // Physical function row positions shared by ANSI, ISO and JIS.
    const POSITIONS: [(usize, usize); 12] = [
        (3, 5),
        (2, 5),
        (6, 4),
        (3, 4),
        (4, 10),
        (3, 10),
        (2, 10),
        (1, 15),
        (3, 11),
        (4, 8),
        (6, 8),
        (3, 13),
    ];
    POSITIONS
        .iter()
        .position(|&p| p == (row, col))
        .map(|i| i as u8)
}

/// QMK's row-dependent thresholds account for the resistor matrix when
/// several keys share a row. Input and thresholds are in millivolts.
pub fn interpret(voltages: &[[u16; COLS]; ROWS]) -> Matrix {
    let mut result = [[false; COLS]; ROWS];
    for row in 0..ROWS {
        let count = voltages[row].iter().filter(|&&v| v < 2900).count();
        let threshold = match count {
            0 | 1 => 1000,
            2 => 2000,
            3 => 2500,
            _ => 2900,
        };
        for col in 0..COLS {
            result[row][col] = voltages[row][col] < threshold;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(keys: &mut Keys, matrix: Matrix, now: u64, layout: &Keymap) {
        keys.update(matrix, now, layout);
        keys.update(matrix, now + DEBOUNCE_US, layout);
    }

    fn output(keys: &Keys) -> Vec<u8> {
        keys.pressed().collect()
    }

    #[test]
    fn modifiers_chords_holds_and_release() {
        let mut keys = Keys::new();
        let mut matrix = [[false; COLS]; ROWS];
        matrix[1][12] = true; // Left Ctrl
        matrix[1][9] = true; // Left Shift
        matrix[7][2] = true; // A
        settle(&mut keys, matrix, 0, &layouts::ANSI);
        assert_eq!(output(&keys), [0xe1, 0xe0, 0x04]);
        keys.update(matrix, 20_000, &layouts::ANSI);
        assert_eq!(output(&keys), [0xe1, 0xe0, 0x04]);
        settle(&mut keys, [[false; COLS]; ROWS], 30_000, &layouts::ANSI);
        assert!(output(&keys).is_empty());
    }

    #[test]
    fn bounce_does_not_press_or_release_a_key() {
        let mut keys = Keys::new();
        let mut matrix = [[false; COLS]; ROWS];
        matrix[7][2] = true;
        keys.update(matrix, 0, &layouts::ANSI);
        assert!(output(&keys).is_empty());
        keys.update([[false; COLS]; ROWS], 2_000, &layouts::ANSI);
        settle(&mut keys, matrix, 3_000, &layouts::ANSI);
        assert_eq!(output(&keys), [4]);
        keys.update([[false; COLS]; ROWS], 9_000, &layouts::ANSI);
        keys.update(matrix, 10_000, &layouts::ANSI);
        assert_eq!(output(&keys), [4]);
    }

    #[test]
    fn fn_navigation_and_binding_survive_fn_release() {
        for (position, usage) in [
            ((6, 11), 0x4a),
            ((1, 13), 0x4b),
            ((1, 8), 0x4e),
            ((2, 15), 0x4d),
            ((0, 1), 0x49),
            ((5, 13), 0x48),
            ((7, 10), 0x47),
        ] {
            let mut keys = Keys::new();
            let mut matrix = [[false; COLS]; ROWS];
            matrix[2][2] = true;
            matrix[position.0][position.1] = true;
            settle(&mut keys, matrix, 0, &layouts::ANSI);
            assert_eq!(output(&keys), [usage]);
            matrix[2][2] = false;
            settle(&mut keys, matrix, 10_000, &layouts::ANSI);
            assert_eq!(output(&keys), [usage]);
        }
    }

    #[test]
    fn fn_lock_toggles_once_and_only_changes_function_row() {
        let mut keys = Keys::new();
        let mut matrix = [[false; COLS]; ROWS];
        matrix[2][2] = true;
        matrix[7][5] = true; // Fn+Esc
        settle(&mut keys, matrix, 0, &layouts::ANSI);
        keys.update(matrix, 10_000, &layouts::ANSI);
        assert!(keys.fn_lock);
        assert!(output(&keys).is_empty());
        settle(&mut keys, [[false; COLS]; ROWS], 20_000, &layouts::ANSI);
        matrix = [[false; COLS]; ROWS];
        matrix[3][5] = true; // F1
        matrix[1][8] = true; // Down stays Down when locked
        settle(&mut keys, matrix, 30_000, &layouts::ANSI);
        assert_eq!(output(&keys), [0x51, 0x3a]);
        settle(&mut keys, [[false; COLS]; ROWS], 40_000, &layouts::ANSI);
        matrix = [[false; COLS]; ROWS];
        matrix[2][2] = true;
        matrix[7][5] = true;
        settle(&mut keys, matrix, 50_000, &layouts::ANSI);
        assert!(!keys.fn_lock);
    }

    #[test]
    fn pressing_fn_does_not_remap_an_already_held_key() {
        let mut keys = Keys::new();
        let mut matrix = [[false; COLS]; ROWS];
        matrix[1][8] = true; // Down
        settle(&mut keys, matrix, 0, &layouts::ANSI);
        matrix[2][2] = true;
        settle(&mut keys, matrix, 10_000, &layouts::ANSI);
        assert_eq!(output(&keys), [0x51]);
    }

    #[test]
    fn jis_fn_omits_media_but_preserves_print_screen() {
        let mut keys = Keys::new();
        let mut matrix = [[false; COLS]; ROWS];
        matrix[2][2] = true;
        matrix[3][5] = true; // Fn+F1 = unsupported Mute
        settle(&mut keys, matrix, 0, &layouts::JIS);
        assert!(output(&keys).is_empty());
        matrix[6][8] = true; // Fn+F11 = Print Screen
        settle(&mut keys, matrix, 10_000, &layouts::JIS);
        assert_eq!(output(&keys), [0x46]);
    }

    #[test]
    fn variant_specific_keys_and_all_function_keys() {
        assert_eq!(layouts::ISO[5][11], 0x64); // Non-US backslash
        assert_eq!(layouts::ISO[2][8], 0x32); // Non-US hash
        assert_eq!(layouts::ANSI[2][8], 0x31); // ANSI backslash
        assert_eq!(layouts::JIS[0][15], 0x89); // Yen
        assert_eq!(layouts::JIS[4][11], 0x87); // Ro
        assert_eq!(layouts::JIS[6][2], 0x8b); // Muhenkan
        for layout in [&layouts::ANSI, &layouts::ISO, &layouts::JIS] {
            for row in 0..ROWS {
                for col in 0..COLS {
                    if let Some(index) = function_index(row, col) {
                        let mut keys = Keys::new();
                        let mut matrix = [[false; COLS]; ROWS];
                        matrix[row][col] = true;
                        matrix[2][2] = layout != &layouts::JIS;
                        settle(&mut keys, matrix, 0, layout);
                        assert_eq!(output(&keys), [0x3a + index]);
                    }
                }
            }
        }
    }

    #[test]
    fn duplicate_macropad_keys_produce_one_usage() {
        let mut keys = Keys::new();
        let mut matrix = [[false; COLS]; ROWS];
        matrix[1][0] = true;
        matrix[2][7] = true;
        settle(&mut keys, matrix, 0, &layouts::MACROPAD);
        assert_eq!(output(&keys), [0x62]); // keypad 0
        assert_eq!(layouts::NUMPAD[1][0], 0x62);
        assert_eq!(layouts::NUMPAD[2][0], 0x58); // keypad Enter
    }

    #[test]
    fn row_thresholds_match_qmk_for_multiple_keys() {
        let mut voltages = [[3300; COLS]; ROWS];
        voltages[0][0] = 900;
        voltages[1][0] = 1900;
        voltages[1][1] = 1900;
        voltages[2][0..3].fill(2400);
        voltages[3][0..4].fill(2800);
        voltages[4][0] = 1100; // rejected under single-key threshold
        let matrix = interpret(&voltages);
        for (row, expected) in [1, 2, 3, 4, 0].into_iter().enumerate() {
            assert_eq!(matrix[row].iter().filter(|&&v| v).count(), expected);
        }
    }
}
