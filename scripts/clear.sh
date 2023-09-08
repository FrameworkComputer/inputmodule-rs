#!/bin/sh
./clear.py
# Flash starts at 0x10000000 in memory.
../qmk_firmware/util/uf2conv.py clear.bin -o clear.uf2 -b 0x10000000 -f rp2040 --convert
