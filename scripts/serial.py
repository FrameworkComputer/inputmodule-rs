#!/usr/bin/env python3
import zlib

ledmatrix_1 = b'FRAKDEAM1100000000' # POC 1
ledmatrix_2 = b'FRAKDEBZ4100000000' # EVT 1, config 1
ledmatrix_3 = b'FRAKDEBZ4200000000' # EVT 1, config 2 (27k resistor)

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
