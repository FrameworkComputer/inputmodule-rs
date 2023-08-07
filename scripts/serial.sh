#!/bin/sh
./serial.py
../qmk_firmware/util/uf2conv.py serial.bin -o serial.uf2 -b 0x100ff000 -f rp2040 --convert
