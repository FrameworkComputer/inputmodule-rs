# Flash Layout

The flash is 1MB large and consists of 256 4K blocks.
The last block is used to store the serial number.

###### LED Matrix

| Start    | End      | Size          | Name               |
|----------|----------|---------------|--------------------|
| 0x000000 | Dynamic  | Roughly 40K   | Firmware           |
| TBD      | 0x0FF000 | TBD           | Persistent Storage |
| 0x0FF000 | 0x100000 | 0x1000 (4K)   | Serial Number      |

###### QMK Keyboards

| Start    | End      | Size          | Name               |
|----------|----------|---------------|--------------------|
| 0x000000 | Dynamic  | Roughly 60K   | Firmware           |
| 0xef000  | 0x0FF000 | 0x10000 (16K) | Persistent Storage |
| 0x0FF000 | 0x100000 | 0x01000 (4K)  | Serial Number      |

## Serial Number

- 1 byte serial number revision (== 1)
- 18 bytes serial number
- 1 byte hardware revision
- 4 byte CRC checksum over serial number (CRC32B, same as Python's `zlib.crc32()`)

Hardware Revisions:

- B1 Display
  - 1 First Prototype, very early prototype
- LED Matrix
  - 1 First Prototype (ATC)
  - 2 Second Prototype (BizLink)
  - 3 Third Prototype, 27k Resistor
- Keyboard, Numpad, Macropad
  - 1 First Prototype
