//! Pattern storage for custom startup patterns
//!
//! Stores static frames or short animations in flash for use as startup patterns.
//! Each pattern slot can hold a single frame or an animation up to MAX_ANIMATION_FRAMES.
//!
//! ## Wear Leveling
//!
//! Pattern storage uses a simple wear-leveling scheme to extend flash life:
//! - Each slot has space for multiple pattern entries
//! - New patterns are written to the next empty position (sequential write)
//! - A sequence number tracks which entry is the most recent
//! - When the slot is full, it's erased and writing starts fresh
//!
//! This reduces erase cycles since we only erase when the slot fills up,
//! not on every write.

use crate::matrix::{Grid, HEIGHT, WIDTH};
use crate::storage::flash::FlashStorage;
use crate::storage::{PAGE_SIZE, PATTERN_STORAGE_ADDR, PATTERN_STORAGE_SIZE};

/// Maximum number of pattern slots
pub const MAX_PATTERN_SLOTS: usize = 8;

/// Maximum frames per animation
pub const MAX_ANIMATION_FRAMES: usize = 16;

/// Size of a single frame (9 * 34 = 306 bytes)
pub const FRAME_SIZE: usize = WIDTH * HEIGHT;

/// Pages per pattern slot (enough for header + 16 frames)
/// 16 frames * 306 bytes = 4896 bytes, so we need 2 pages (8KB) per slot
const PAGES_PER_SLOT: usize = 2;

/// Size of each pattern slot
const SLOT_SIZE: usize = PAGES_PER_SLOT * PAGE_SIZE;

/// Size of a single pattern entry (header + 1 frame, aligned to 512 bytes)
/// This allows ~16 entries per slot before needing to erase
const ENTRY_SIZE: usize = 512;

/// Maximum entries per slot for wear leveling (8KB / 512 = 16)
const MAX_ENTRIES_PER_SLOT: usize = SLOT_SIZE / ENTRY_SIZE;

/// Pattern type discriminator
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PatternType {
    /// Single static frame
    Static = 0,
    /// Multi-frame animation
    Animation = 1,
}

/// Magic bytes to identify valid pattern header
const PATTERN_MAGIC: [u8; 2] = [0xAA, 0x01];

/// Pattern header stored in flash (32 bytes)
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct PatternHeader {
    /// Pattern slot index (0-7)
    pub slot: u8,
    /// Pattern type (static or animation)
    pub pattern_type: PatternType,
    /// Number of frames (1 for static, 2-16 for animation)
    pub frame_count: u8,
    /// Frame delay in milliseconds (for animations)
    pub frame_delay_ms: u8,
    /// CRC16 of the frame data for validation
    pub data_crc: u16,
    /// Sequence number for wear leveling (higher = newer)
    pub sequence: u16,
}

impl PatternHeader {
    /// Serialize header to bytes (10 bytes of data, padded to 32)
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0xFFu8; 32];
        bytes[0] = PATTERN_MAGIC[0];
        bytes[1] = PATTERN_MAGIC[1];
        bytes[2] = self.slot;
        bytes[3] = self.pattern_type as u8;
        bytes[4] = self.frame_count;
        bytes[5] = self.frame_delay_ms;
        bytes[6..8].copy_from_slice(&self.data_crc.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.sequence.to_le_bytes());
        bytes
    }

    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 32 {
            return None;
        }

        // Check magic bytes
        if bytes[0] != PATTERN_MAGIC[0] || bytes[1] != PATTERN_MAGIC[1] {
            return None;
        }

        let pattern_type = match bytes[3] {
            0 => PatternType::Static,
            1 => PatternType::Animation,
            _ => return None,
        };

        Some(Self {
            slot: bytes[2],
            pattern_type,
            frame_count: bytes[4],
            frame_delay_ms: bytes[5],
            data_crc: u16::from_le_bytes([bytes[6], bytes[7]]),
            sequence: u16::from_le_bytes([bytes[8], bytes[9]]),
        })
    }
}

/// Stored pattern with header and frame data
pub struct StoredPattern {
    pub header: PatternHeader,
    pub frames: [[u8; FRAME_SIZE]; MAX_ANIMATION_FRAMES],
}

impl StoredPattern {
    /// Create a new static pattern from a grid
    pub fn from_grid(slot: u8, grid: &Grid, sequence: u16) -> Self {
        let mut frames = [[0u8; FRAME_SIZE]; MAX_ANIMATION_FRAMES];

        // Flatten the grid into the first frame
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                frames[0][x * HEIGHT + y] = grid.0[x][y];
            }
        }

        let crc = calculate_crc(&frames[0]);

        Self {
            header: PatternHeader {
                slot,
                pattern_type: PatternType::Static,
                frame_count: 1,
                frame_delay_ms: 0,
                data_crc: crc,
                sequence,
            },
            frames,
        }
    }

    /// Get the first frame as a Grid
    pub fn first_frame(&self) -> Grid {
        self.frame_to_grid(0)
    }

    /// Convert a frame index to a Grid
    pub fn frame_to_grid(&self, index: usize) -> Grid {
        let mut grid = Grid::default();
        if index < self.header.frame_count as usize {
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    grid.0[x][y] = self.frames[index][x * HEIGHT + y];
                }
            }
        }
        grid
    }

    /// Check if this is an animation
    pub fn is_animation(&self) -> bool {
        self.header.pattern_type == PatternType::Animation
    }
}

/// Information about a pattern slot (for listing)
#[derive(Clone, Copy, Debug, Default)]
pub struct PatternSlotInfo {
    /// Whether the slot is occupied
    pub occupied: bool,
    /// Pattern type (if occupied)
    pub pattern_type: u8,
    /// Number of frames
    pub frame_count: u8,
    /// Frame delay in milliseconds
    pub frame_delay_ms: u8,
}

/// Calculate simple CRC16 for frame data
fn calculate_crc(data: &[u8]) -> u16 {
    let mut sum: u16 = 0;
    for byte in data {
        sum = sum.wrapping_add(*byte as u16);
    }
    sum ^ 0xB5B5
}

/// Get the flash offset for a pattern slot
fn slot_offset(slot: u8) -> u32 {
    (slot as usize * SLOT_SIZE) as u32
}

/// Get the flash offset for a specific entry within a slot
fn entry_offset(slot: u8, entry: usize) -> u32 {
    slot_offset(slot) + (entry * ENTRY_SIZE) as u32
}

/// Find the latest valid entry in a slot (highest sequence number)
/// Returns (entry_index, sequence_number) or None if slot is empty
fn find_latest_entry(flash: &FlashStorage, slot: u8) -> Option<(usize, u16)> {
    let mut latest: Option<(usize, u16)> = None;

    for entry in 0..MAX_ENTRIES_PER_SLOT {
        let offset = entry_offset(slot, entry);
        let mut header_bytes = [0u8; 32];
        flash.read_at(offset, &mut header_bytes);

        if let Some(header) = PatternHeader::from_bytes(&header_bytes) {
            if header.slot == slot {
                match latest {
                    None => latest = Some((entry, header.sequence)),
                    Some((_, seq)) if header.sequence > seq => {
                        latest = Some((entry, header.sequence));
                    }
                    _ => {}
                }
            }
        }
    }

    latest
}

/// Find the next empty entry in a slot
/// Returns entry index or None if slot is full
fn find_empty_entry(flash: &FlashStorage, slot: u8) -> Option<usize> {
    for entry in 0..MAX_ENTRIES_PER_SLOT {
        let offset = entry_offset(slot, entry);
        let mut magic = [0u8; 2];
        flash.read_at(offset, &mut magic);

        // Empty entry has 0xFF magic bytes
        if magic[0] == 0xFF && magic[1] == 0xFF {
            return Some(entry);
        }
    }
    None
}

/// Save a pattern to flash
///
/// Saves the current grid as a static pattern in the specified slot.
/// Uses wear leveling - writes to next empty entry, only erases when full.
pub fn save_pattern(slot: u8, grid: &Grid) -> bool {
    if slot >= MAX_PATTERN_SLOTS as u8 {
        return false;
    }

    let mut flash = FlashStorage::new(PATTERN_STORAGE_ADDR, PATTERN_STORAGE_SIZE as u32);

    // Find current sequence number (if any pattern exists)
    let next_sequence = match find_latest_entry(&flash, slot) {
        Some((_, seq)) => seq.wrapping_add(1),
        None => 1,
    };

    // Find empty entry, or erase slot if full
    let entry_idx = match find_empty_entry(&flash, slot) {
        Some(idx) => idx,
        None => {
            // Slot is full, erase and start fresh
            let base_offset = slot_offset(slot);
            for page in 0..PAGES_PER_SLOT {
                let page_offset = base_offset + (page * PAGE_SIZE) as u32;
                if flash.erase_sector(page_offset).is_err() {
                    return false;
                }
            }
            0 // Start at first entry after erase
        }
    };

    let pattern = StoredPattern::from_grid(slot, grid, next_sequence);
    save_pattern_at_entry(&mut flash, &pattern, entry_idx)
}

fn save_pattern_at_entry(flash: &mut FlashStorage, pattern: &StoredPattern, entry: usize) -> bool {
    let offset = entry_offset(pattern.header.slot, entry);

    // Write header (first 32 bytes)
    let header_bytes = pattern.header.to_bytes();
    if flash.write_at(offset, &header_bytes).is_err() {
        return false;
    }

    // Write frame data starting at offset 32
    // For static patterns, we have 306 bytes of frame data
    // Entry size is 512, so we have room for header (32) + frame (306) = 338 bytes
    let frame_offset = offset + 32;

    // Write first 256 bytes of frame
    if flash.write_at(frame_offset, &pattern.frames[0][..256]).is_err() {
        return false;
    }

    // Write remaining 50 bytes (306 - 256 = 50)
    // Pad to 256 for write alignment
    let mut remaining = [0xFFu8; 256];
    remaining[..50].copy_from_slice(&pattern.frames[0][256..]);
    if flash.write_at(frame_offset + 256, &remaining).is_err() {
        return false;
    }

    true
}

/// Load a pattern from flash
///
/// Finds the latest valid entry in the slot and loads it.
pub fn load_pattern(slot: u8) -> Option<StoredPattern> {
    if slot >= MAX_PATTERN_SLOTS as u8 {
        return None;
    }

    let flash = FlashStorage::new(PATTERN_STORAGE_ADDR, PATTERN_STORAGE_SIZE as u32);

    // Find the latest entry
    let (entry_idx, _) = find_latest_entry(&flash, slot)?;
    let offset = entry_offset(slot, entry_idx);

    // Read header
    let mut header_bytes = [0u8; 32];
    flash.read_at(offset, &mut header_bytes);

    let header = PatternHeader::from_bytes(&header_bytes)?;

    // Read frame data
    let mut frames = [[0u8; FRAME_SIZE]; MAX_ANIMATION_FRAMES];
    let frame_offset = offset + 32;

    // Read first 256 bytes
    flash.read_at(frame_offset, &mut frames[0][..256]);

    // Read remaining 50 bytes
    let mut remaining = [0u8; 256];
    flash.read_at(frame_offset + 256, &mut remaining);
    frames[0][256..].copy_from_slice(&remaining[..50]);

    // Verify CRC
    let calc_crc = calculate_crc(&frames[0]);
    if calc_crc != header.data_crc {
        return None;
    }

    Some(StoredPattern { header, frames })
}

/// Delete a pattern from a slot
///
/// Erases all entries in the slot.
pub fn delete_pattern(slot: u8) -> bool {
    if slot >= MAX_PATTERN_SLOTS as u8 {
        return false;
    }

    let mut flash = FlashStorage::new(PATTERN_STORAGE_ADDR, PATTERN_STORAGE_SIZE as u32);
    let base_offset = slot_offset(slot);

    // Erase the slot's pages
    for page in 0..PAGES_PER_SLOT {
        let page_offset = base_offset + (page * PAGE_SIZE) as u32;
        if flash.erase_sector(page_offset).is_err() {
            return false;
        }
    }

    true
}

/// List all pattern slots
///
/// Finds the latest entry in each slot to report status.
pub fn list_patterns() -> [PatternSlotInfo; MAX_PATTERN_SLOTS] {
    let mut result = [PatternSlotInfo::default(); MAX_PATTERN_SLOTS];
    let flash = FlashStorage::new(PATTERN_STORAGE_ADDR, PATTERN_STORAGE_SIZE as u32);

    for slot in 0..MAX_PATTERN_SLOTS as u8 {
        // Find the latest entry in this slot
        if let Some((entry_idx, _)) = find_latest_entry(&flash, slot) {
            let offset = entry_offset(slot, entry_idx);
            let mut header_bytes = [0u8; 32];
            flash.read_at(offset, &mut header_bytes);

            if let Some(header) = PatternHeader::from_bytes(&header_bytes) {
                result[slot as usize] = PatternSlotInfo {
                    occupied: true,
                    pattern_type: header.pattern_type as u8,
                    frame_count: header.frame_count,
                    frame_delay_ms: header.frame_delay_ms,
                };
            }
        }
    }

    result
}
