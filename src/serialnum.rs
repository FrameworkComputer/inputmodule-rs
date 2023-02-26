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

/// Get the firmware version in a format for USB Device Release
/// The value is in binary coded decimal with a format of 0xJJMN where JJ is the major version number, M is the minor version number and N is the sub minor version number. e.g. USB 2.0 is reported as 0x0200, USB 1.1 as 0x0110 and USB 1.0 as 0x0100.
pub fn device_release() -> u16 {
    (env!("CARGO_PKG_VERSION_MAJOR").parse::<u16>().unwrap() << 8)
        + (env!("CARGO_PKG_VERSION_MINOR").parse::<u16>().unwrap() << 4)
        + env!("CARGO_PKG_VERSION_PATCH").parse::<u16>().unwrap()
}

pub fn is_pre_release() -> bool {
    !env!("CARGO_PKG_VERSION_PRE").is_empty()
}
