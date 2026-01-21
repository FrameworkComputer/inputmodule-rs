// Get serial number from last 4K block of the first 1M
const FLASH_OFFSET: usize = 0x10000000;
const LAST_4K_BLOCK: usize = 0xff000;
const SERIALNUM_LEN: usize = 18;

#[repr(C, packed)]
pub struct SerialnumStructRaw {
    sn_rev: u8,
    serialnum: [u8; SERIALNUM_LEN],
    crc32: [u8; 4],
}

pub struct SerialnumStruct {
    pub serialnum: &'static str,
}

pub fn get_serialnum() -> Option<SerialnumStruct> {
    // Flash is mapped into memory, just read it from there
    let ptr: *const u8 = (FLASH_OFFSET + LAST_4K_BLOCK) as *const u8;
    let sn_raw_ptr = ptr as *const SerialnumStructRaw;
    let sn_raw = unsafe { sn_raw_ptr.as_ref()? };

    // Only rev 1 supported
    if sn_raw.sn_rev != 1 {
        return None;
    }

    let crc: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let mut digest = crc.digest();
    digest.update(&[sn_raw.sn_rev]);
    digest.update(&sn_raw.serialnum);
    let calc_checksum = digest.finalize();

    let actual_checksum = u32::from_le_bytes(sn_raw.crc32);
    // Checksum invalid, serial fall back to default serial number
    if calc_checksum != actual_checksum {
        return None;
    }

    Some(SerialnumStruct {
        serialnum: core::str::from_utf8(&sn_raw.serialnum).ok()?,
    })
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
