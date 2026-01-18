//! Flash storage for persistent configuration and patterns
//!
//! This module provides wear-leveled flash storage for:
//! - Configuration settings (brightness, sleep timeout, etc.)
//! - Custom startup patterns (static frames or animations)
//!
//! Flash layout (from memory.x):
//! - PATTERN_STORAGE: 0x100E0000, 60KB (15 x 4KB pages)
//! - CONFIG_STORAGE:  0x100EF000, 64KB (16 x 4KB pages)
//! - SERIALNUM:       0x100FF000, 4KB  (read-only, untouched)

pub mod config;
pub mod flash;
pub mod patterns;

pub use config::{load_config, save_config, StoredConfig};
pub use flash::FlashStorage;
pub use patterns::{
    delete_pattern, list_patterns, load_pattern, save_pattern, PatternHeader, PatternSlotInfo,
    StoredPattern, MAX_ANIMATION_FRAMES, MAX_PATTERN_SLOTS,
};

/// Flash page size for RP2040 (4KB)
pub const PAGE_SIZE: usize = 4096;

/// Flash sector size for RP2040 (4KB, same as page)
pub const SECTOR_SIZE: usize = 4096;

/// Pattern storage region start address
pub const PATTERN_STORAGE_ADDR: u32 = 0x100E_0000;

/// Pattern storage region size (60KB = 15 pages)
pub const PATTERN_STORAGE_SIZE: usize = 60 * 1024;

/// Config storage region start address
pub const CONFIG_STORAGE_ADDR: u32 = 0x100E_F000;

/// Config storage region size (64KB = 16 pages)
pub const CONFIG_STORAGE_SIZE: usize = 64 * 1024;
