//! Low-level flash operations for RP2040
//!
//! Uses RP2040 ROM functions for flash erase and program operations.
//! Flash operations must be performed with interrupts disabled.

use rp2040_hal::rom_data;

/// Flash base address (XIP region)
pub const FLASH_BASE: u32 = 0x1000_0000;

/// RP2040 flash page size (256 bytes for programming)
const FLASH_PAGE_SIZE: usize = 256;

/// RP2040 flash sector size (4KB for erasing)
const FLASH_SECTOR_SIZE: usize = 4096;

/// Total flash size (2MB)
const FLASH_SIZE: usize = 2 * 1024 * 1024;

/// Flash storage wrapper for a specific region
pub struct FlashStorage {
    /// Start offset from FLASH_BASE
    start_offset: u32,
    /// Size of this storage region
    size: u32,
}

impl FlashStorage {
    /// Create a new flash storage region
    ///
    /// # Arguments
    /// * `start_addr` - Absolute start address (must be >= FLASH_BASE)
    /// * `size` - Size of the storage region in bytes
    ///
    /// # Panics
    /// Panics if start_addr is below FLASH_BASE or region exceeds flash size
    pub fn new(start_addr: u32, size: u32) -> Self {
        let start_offset = start_addr - FLASH_BASE;
        assert!(start_addr >= FLASH_BASE);
        assert!((start_offset + size) as usize <= FLASH_SIZE);

        Self { start_offset, size }
    }

    /// Get the absolute address for a given offset within this region
    fn absolute_addr(&self, offset: u32) -> u32 {
        FLASH_BASE + self.start_offset + offset
    }

    /// Get the flash offset for a given offset within this region
    fn flash_offset(&self, offset: u32) -> u32 {
        self.start_offset + offset
    }

    /// Read bytes from flash at the given offset
    ///
    /// Flash is memory-mapped, so this is a direct memory read.
    pub fn read_at(&self, offset: u32, bytes: &mut [u8]) {
        if offset + bytes.len() as u32 > self.size {
            return;
        }

        let addr = self.absolute_addr(offset);
        let ptr = addr as *const u8;

        // Flash is memory-mapped, just read directly
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = unsafe { *ptr.add(i) };
        }
    }

    /// Write bytes to flash at the given offset
    ///
    /// Data is written in 256-byte pages. If the data doesn't align to page
    /// boundaries, it will be padded with 0xFF (erased state).
    ///
    /// # Important
    /// The flash region must be erased before writing.
    /// Writing to non-erased flash may produce incorrect data.
    pub fn write_at(&mut self, offset: u32, bytes: &[u8]) -> Result<(), FlashError> {
        if offset + bytes.len() as u32 > self.size {
            return Err(FlashError::OutOfBounds);
        }

        let flash_offset = self.flash_offset(offset);

        // Prepare a page-aligned buffer
        let mut page_buf = [0xFFu8; FLASH_PAGE_SIZE];
        let data_len = bytes.len().min(FLASH_PAGE_SIZE);
        page_buf[..data_len].copy_from_slice(&bytes[..data_len]);

        // Flash operations must be done with interrupts disabled
        cortex_m::interrupt::free(|_| unsafe {
            rom_data::flash_range_program(flash_offset, page_buf.as_ptr(), FLASH_PAGE_SIZE);
        });

        Ok(())
    }

    /// Erase a sector (4KB) at the given offset
    ///
    /// The offset must be sector-aligned (multiple of 4096).
    pub fn erase_sector(&mut self, offset: u32) -> Result<(), FlashError> {
        if offset + FLASH_SECTOR_SIZE as u32 > self.size {
            return Err(FlashError::OutOfBounds);
        }

        // Must be sector-aligned
        if offset as usize % FLASH_SECTOR_SIZE != 0 {
            return Err(FlashError::NotAligned);
        }

        let flash_offset = self.flash_offset(offset);

        // Flash operations must be done with interrupts disabled
        cortex_m::interrupt::free(|_| unsafe {
            rom_data::flash_range_erase(
                flash_offset,
                FLASH_SECTOR_SIZE,
                FLASH_SECTOR_SIZE as u32,
                0,
            );
        });

        Ok(())
    }

    /// Get the capacity of this storage region
    pub fn capacity(&self) -> usize {
        self.size as usize
    }
}

/// Flash operation error
#[derive(Debug)]
pub enum FlashError {
    /// Operation would exceed storage region bounds
    OutOfBounds,
    /// Offset not properly aligned
    NotAligned,
}
