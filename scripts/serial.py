#!/usr/bin/env python3
import zlib

ledmatrix_1    = b'FRAKDEAM1100000000' # POC 1
ledmatrix_2    = b'FRAKDEBZ4100000000' # EVT 1, config 1
ledmatrix_3    = b'FRAKDEBZ4200000000' # EVT 1, config 2 (27k resistor)
ansi_keyboard  = b'FRAKDWEN4100000000' # EVT 1, config 1 (US ANSI)
rgb_keyboard   = b'FRAKDKEN4100000000' # EVT 1, config 1 (US ANSI)
iso_keyboard   = b'FRAKDWEN4200000000' # EVT 1, config 2 (UK ISO)
jis_keyboard   = b'FRAKDWEN4J00000000' # EVT 1, config J (JIS)
numpad         = b'FRAKDMEN4100000000' # EVT 1, config 1
macropad       = b'FRAKDNEN4100000000' # EVT 1, config 1

# This section is for modifying
selected   = ledmatrix_2
year       = b'3' # 2023
week       = b'01'
day        = b'1'
part_sn    = b'0001'

config     = selected[8:10]
serial_rev = b'\x01'
snum       = selected
print(serial_rev + snum)
snum       = snum[0:8] + config + year + week + day + part_sn

checksum   = zlib.crc32(serial_rev + snum)
print(serial_rev + snum)

print('Checksum:', hex(zlib.crc32(snum)))
print('Digest:  ', hex(checksum))
with open('serial.bin', 'wb') as f:
    f.write(serial_rev)
    f.write(snum)
    f.write(checksum.to_bytes(4, 'little'))
