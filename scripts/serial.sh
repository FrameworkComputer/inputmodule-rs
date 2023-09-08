#!/bin/sh
# Adjust the Python code to output what you need
./serial.py
# Flash starts at 0x10000000 in memory.
# 0x000ff000 is the offset to the last 4k sector in the flash (1MB-4KB)
../qmk_firmware/util/uf2conv.py serial.bin -o serial.uf2 -b 0x100ff000 -f rp2040 --convert
