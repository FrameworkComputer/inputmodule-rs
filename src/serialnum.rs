// Get serial number from last 4K block of the first 1M
const FLASH_OFFSET: usize = 0x10000000;
const LAST_4K_BLOCK: usize = 0xff000;
const SERIALNUM_LEN: usize = 18;

pub fn get_serialnum() -> Option<&'static str> {
    // Flash is mapped into memory, just read it from there
    let ptr: *const u8 = (FLASH_OFFSET + LAST_4K_BLOCK) as *const u8;
    unsafe {
        let slice: &[u8] = core::slice::from_raw_parts(ptr, SERIALNUM_LEN);
        if slice[0] == 0xFF || slice[0] == 0x00 {
            return None;
        }
        core::str::from_utf8(slice).ok()
    }
}
