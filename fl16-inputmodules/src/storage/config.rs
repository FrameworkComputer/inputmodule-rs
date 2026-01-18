//! Configuration storage with simple wear leveling
//!
//! Uses a simple page rotation scheme for wear leveling.
//! Each config write goes to the next available slot in a page,
//! and pages are erased when full.

use crate::storage::flash::FlashStorage;
use crate::storage::{CONFIG_STORAGE_ADDR, CONFIG_STORAGE_SIZE, PAGE_SIZE};

/// Configuration format version (for future migrations)
pub const CONFIG_VERSION: u8 = 1;

/// Default brightness (0-255)
pub const DEFAULT_BRIGHTNESS: u8 = 51;

/// Default sleep timeout in seconds (60 seconds)
pub const DEFAULT_SLEEP_TIMEOUT_SECS: u16 = 60;

/// Default animation period in microseconds (31,250 = 32 FPS)
pub const DEFAULT_ANIMATION_PERIOD_US: u32 = 31_250;

/// Default PWM frequency (0 = 29kHz)
pub const DEFAULT_PWM_FREQ: u8 = 0;

/// No startup pattern selected (use built-in random animation)
pub const NO_STARTUP_PATTERN: u8 = 0xFF;

/// Config entry size including header (must be power of 2 for alignment)
const CONFIG_ENTRY_SIZE: usize = 32;

/// Magic bytes to identify valid config entry
const CONFIG_MAGIC: [u8; 2] = [0xCF, 0x01];

/// Stored configuration structure
///
/// This is stored in flash and persists across power cycles.
/// Total size: 16 bytes (padded to 32 for entry)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct StoredConfig {
    /// Format version for future migrations
    pub version: u8,
    /// Default brightness (0-255)
    pub brightness: u8,
    /// Sleep timeout in seconds (0 = disabled)
    pub sleep_timeout_secs: u16,
    /// Animation period in microseconds
    pub animation_period_us: u32,
    /// PWM frequency setting (0-3 maps to PwmFreqArg)
    pub pwm_freq: u8,
    /// Enable random startup animation
    pub startup_animation: bool,
    /// Pattern slot index for startup (0xFF = none, use random)
    pub startup_pattern_idx: u8,
    /// Reserved for future use
    pub _reserved: [u8; 5],
}

impl Default for StoredConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            brightness: DEFAULT_BRIGHTNESS,
            sleep_timeout_secs: DEFAULT_SLEEP_TIMEOUT_SECS,
            animation_period_us: DEFAULT_ANIMATION_PERIOD_US,
            pwm_freq: DEFAULT_PWM_FREQ,
            startup_animation: true,
            startup_pattern_idx: NO_STARTUP_PATTERN,
            _reserved: [0; 5],
        }
    }
}

impl StoredConfig {
    /// Serialize config to bytes for storage
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0] = self.version;
        bytes[1] = self.brightness;
        bytes[2..4].copy_from_slice(&self.sleep_timeout_secs.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.animation_period_us.to_le_bytes());
        bytes[8] = self.pwm_freq;
        bytes[9] = self.startup_animation as u8;
        bytes[10] = self.startup_pattern_idx;
        // bytes[11..16] = reserved
        bytes
    }

    /// Deserialize config from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 {
            return None;
        }

        let version = bytes[0];
        if version != CONFIG_VERSION {
            // Future: handle migration from older versions
            return None;
        }

        Some(Self {
            version,
            brightness: bytes[1],
            sleep_timeout_secs: u16::from_le_bytes([bytes[2], bytes[3]]),
            animation_period_us: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            pwm_freq: bytes[8],
            startup_animation: bytes[9] != 0,
            startup_pattern_idx: bytes[10],
            _reserved: [0; 5],
        })
    }
}

/// Find the most recent valid config entry in flash
fn find_latest_config() -> Option<(usize, StoredConfig)> {
    let flash = FlashStorage::new(CONFIG_STORAGE_ADDR, CONFIG_STORAGE_SIZE as u32);
    let entries_per_page = PAGE_SIZE / CONFIG_ENTRY_SIZE;
    let total_pages = CONFIG_STORAGE_SIZE / PAGE_SIZE;
    let total_entries = entries_per_page * total_pages;

    // Scan backwards to find the most recent valid entry
    // (most recent writes go to higher addresses until page is erased)
    let mut latest_offset = None;
    let mut latest_config = None;

    for entry_idx in 0..total_entries {
        let offset = entry_idx * CONFIG_ENTRY_SIZE;
        let mut entry = [0u8; CONFIG_ENTRY_SIZE];
        flash.read_at(offset as u32, &mut entry);

        // Check magic bytes
        if entry[0] == CONFIG_MAGIC[0] && entry[1] == CONFIG_MAGIC[1] {
            // Check if this is a valid (non-invalidated) entry
            // An invalidated entry has byte[2] == 0x00
            if entry[2] != 0x00 {
                if let Some(config) = StoredConfig::from_bytes(&entry[4..20]) {
                    // Calculate simple checksum
                    let stored_checksum = u16::from_le_bytes([entry[20], entry[21]]);
                    let calc_checksum = calculate_checksum(&entry[4..20]);
                    if stored_checksum == calc_checksum {
                        latest_offset = Some(offset);
                        latest_config = Some(config);
                    }
                }
            }
        }
    }

    latest_offset.map(|off| (off, latest_config.unwrap()))
}

/// Find the next free slot for writing config
fn find_free_slot() -> Option<usize> {
    let flash = FlashStorage::new(CONFIG_STORAGE_ADDR, CONFIG_STORAGE_SIZE as u32);
    let entries_per_page = PAGE_SIZE / CONFIG_ENTRY_SIZE;
    let total_pages = CONFIG_STORAGE_SIZE / PAGE_SIZE;
    let total_entries = entries_per_page * total_pages;

    for entry_idx in 0..total_entries {
        let offset = entry_idx * CONFIG_ENTRY_SIZE;
        let mut magic = [0u8; 2];
        flash.read_at(offset as u32, &mut magic);

        // Empty slot (erased flash = 0xFF)
        if magic[0] == 0xFF && magic[1] == 0xFF {
            return Some(offset);
        }
    }

    None
}

/// Calculate simple checksum for config data
fn calculate_checksum(data: &[u8]) -> u16 {
    let mut sum: u16 = 0;
    for byte in data {
        sum = sum.wrapping_add(*byte as u16);
    }
    sum ^ 0xA5A5
}

/// Load configuration from flash
///
/// Returns the stored configuration or default if none exists or on error.
pub fn load_config() -> StoredConfig {
    match find_latest_config() {
        Some((_, config)) => config,
        None => StoredConfig::default(),
    }
}

/// Save configuration to flash
///
/// Returns true on success, false on error.
pub fn save_config(config: &StoredConfig) -> bool {
    let mut flash = FlashStorage::new(CONFIG_STORAGE_ADDR, CONFIG_STORAGE_SIZE as u32);

    // Find a free slot
    let offset = match find_free_slot() {
        Some(off) => off,
        None => {
            // No free slot, need to erase a page
            // For simplicity, erase the first page
            if flash.erase_sector(0).is_err() {
                return false;
            }
            0
        }
    };

    // Build the entry
    let mut entry = [0xFFu8; CONFIG_ENTRY_SIZE];
    entry[0] = CONFIG_MAGIC[0];
    entry[1] = CONFIG_MAGIC[1];
    entry[2] = 0x01; // Valid marker
    entry[3] = 0x00; // Reserved

    let config_bytes = config.to_bytes();
    entry[4..20].copy_from_slice(&config_bytes);

    let checksum = calculate_checksum(&config_bytes);
    entry[20..22].copy_from_slice(&checksum.to_le_bytes());

    // Write the entry
    flash.write_at(offset as u32, &entry).is_ok()
}

/// Reset configuration to defaults
///
/// Erases the config storage region and saves default config.
pub fn reset_config() -> bool {
    let mut flash = FlashStorage::new(CONFIG_STORAGE_ADDR, CONFIG_STORAGE_SIZE as u32);

    // Erase all config pages
    let total_pages = CONFIG_STORAGE_SIZE / PAGE_SIZE;
    for page in 0..total_pages {
        let page_offset = (page * PAGE_SIZE) as u32;
        if flash.erase_sector(page_offset).is_err() {
            return false;
        }
    }

    // Save default config
    save_config(&StoredConfig::default())
}
