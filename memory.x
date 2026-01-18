MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    /* Firmware code - reduced to leave room for storage before serial number */
    FLASH : ORIGIN = 0x10000100, LENGTH = 896K - 0x100
    /* Pattern storage: 60KB (15 x 4KB pages) for ~8 pattern slots */
    PATTERN_STORAGE : ORIGIN = 0x100E0000, LENGTH = 60K
    /* Config storage: 64KB (16 x 4KB pages) for wear-leveled configuration */
    CONFIG_STORAGE : ORIGIN = 0x100EF000, LENGTH = 64K
    /* Serial number - programmed at manufacturing, read-only */
    SERIALNUM : ORIGIN = 0x100FF000, LENGTH = 4K
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

/*
 * Note: The FLASH region is sized to leave room for:
 * - PATTERN_STORAGE at 0x100E0000 (60KB)
 * - CONFIG_STORAGE at 0x100EF000 (64KB)
 * - SERIALNUM at 0x100FF000 (4KB, read-only)
 * If firmware grows too large, the linker will error.
 */

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;