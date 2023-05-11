#!/usr/bin/env python
# Strip path names from rust binaries
# Ideally would build with RUSTFLAGS set to:
# RUSTFLAGS="--remap-path-prefix=$$PWD=. --remap-path-prefix=$$CARGO_HOME=home --remap-path-prefix=$$HOME=home" \
# But there's a bug that makes it nuke almost the entire binary, so we can't use it.
# This is a home-grown replacement.

import sys
import os

# Get content of these environment variables, search the binaries for them and
# replace it with a string of Xs with the same length
remap_vars = ['PWD', 'HOME', 'CARGO_HOME']

# Read all of binary in memory
# Should be much less less than 500K
with open(sys.argv[1], "rb") as f:
    binary = f.read()

for var in remap_vars:
    if var in os.environ:
        val = os.environ[var]
        binval = val.encode('ascii')
        replacement = b'X' * len(binval)
        print(var, val, binval, replacement)
        binary = binary.replace(binval, replacement)

with open(sys.argv[1], "wb") as f:
    f.write(binary)
