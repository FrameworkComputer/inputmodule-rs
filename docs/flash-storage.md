# LED Matrix Flash Storage

This document describes the persistent flash storage feature for the Framework Laptop 16 LED Matrix input module. This feature allows saving custom patterns and configuration settings that persist across power cycles and firmware updates.

## Table of Contents

- [Overview](#overview)
- [Flash Memory Layout](#flash-memory-layout)
- [Configuration Storage](#configuration-storage)
  - [Stored Settings](#stored-settings)
  - [Wear Leveling](#wear-leveling)
  - [Startup Behavior](#startup-behavior)
- [Pattern Storage](#pattern-storage)
  - [Pattern Slots](#pattern-slots)
  - [Pattern Format](#pattern-format)
- [CLI Reference](#cli-reference)
  - [Rust CLI](#rust-cli)
  - [Python CLI](#python-cli)
- [USB Protocol](#usb-protocol)
- [Usage Examples](#usage-examples)

## Overview

The LED Matrix module uses the RP2040 microcontroller with 2MB of flash memory. The storage implementation reserves space at the end of flash (before the serial number region) for:

- **Pattern Storage**: Save up to 8 custom display patterns
- **Configuration Storage**: Persist settings like brightness, sleep timeout, and startup behavior

This placement ensures that storage data survives firmware updates, as the firmware occupies the beginning of flash while storage uses the end.

## Flash Memory Layout

```
Flash Address Map (2MB total):

0x10000000 ┌─────────────────────────┐
           │      Bootloader         │ 256 bytes
0x10000100 ├─────────────────────────┤
           │                         │
           │      Firmware           │ ~896KB
           │      (FLASH region)     │
           │                         │
0x100E0000 ├─────────────────────────┤
           │                         │
           │   Pattern Storage       │ 60KB (8 slots × 7.5KB each)
           │                         │
0x100EF000 ├─────────────────────────┤
           │                         │
           │   Config Storage        │ 64KB (wear-leveled)
           │                         │
0x100FF000 ├─────────────────────────┤
           │   Serial Number         │ 4KB (read-only, factory set)
0x10100000 └─────────────────────────┘
```

### Memory Regions

| Region          | Address      | Size  | Purpose                          |
|-----------------|--------------|-------|----------------------------------|
| BOOT2           | 0x10000000   | 256B  | RP2040 boot stage 2              |
| FLASH           | 0x10000100   | 896KB | Firmware code and data           |
| PATTERN_STORAGE | 0x100E0000   | 60KB  | 8 pattern slots                  |
| CONFIG_STORAGE  | 0x100EF000   | 64KB  | Wear-leveled configuration       |
| SERIALNUM       | 0x100FF000   | 4KB   | Factory-programmed serial number |

## Configuration Storage

### Stored Settings

The configuration storage saves the following settings:

| Setting            | Type   | Range       | Default    | Description                              |
|--------------------|--------|-------------|------------|------------------------------------------|
| `brightness`       | u8     | 0-255       | 120        | Default LED brightness on startup        |
| `sleep_timeout`    | u16    | 0-65535     | 0          | Seconds before auto-sleep (0 = disabled) |
| `animation_period` | u32    | microseconds| 31250      | Animation frame period (~32 FPS)         |
| `pwm_freq`         | u8     | 0-3         | 0          | PWM frequency index (0=29kHz)            |
| `startup_animation`| bool   | true/false  | true       | Enable random animation on startup       |
| `startup_pattern`  | u8     | 0-7 or 0xFF | 0xFF       | Pattern slot to display on startup       |

### Wear Leveling

Flash memory has limited write endurance (typically 10,000-100,000 cycles per sector). To extend the lifespan of the configuration storage, a simple wear-leveling scheme is used:

1. **Slot Rotation**: The 64KB config region is divided into 16KB pages, each containing multiple 16-byte config slots
2. **Sequential Writes**: New configs are written to the next available slot rather than erasing and rewriting the same location
3. **Page Cycling**: When a page fills up, the next page is erased and used
4. **Checksum Validation**: Each config slot includes a CRC8 checksum to detect corruption

```
Config Page Structure (4KB sectors, 16 pages total):

┌────────────────────────────────────────┐
│ Slot 0: [Magic][Config Data][Checksum] │ 16 bytes
│ Slot 1: [Magic][Config Data][Checksum] │ 16 bytes
│ Slot 2: [Magic][Config Data][Checksum] │ 16 bytes
│ ...                                     │
│ Slot 255: [Empty - 0xFF]               │
└────────────────────────────────────────┘
```

When reading, the firmware scans for the last valid slot. When writing, it finds the next empty slot or erases the next page if the current one is full.

### Startup Behavior

On power-up, the firmware:

1. Loads the stored configuration from flash (or uses defaults if none found)
2. Applies the saved brightness, animation period, and PWM frequency
3. Checks `startup_pattern`:
   - If set (0-7): Loads and displays that pattern slot
   - If unset (0xFF): Falls through to animation check
4. If `startup_animation` is enabled and no pattern was loaded: Plays a random built-in animation

## Pattern Storage

### Pattern Slots

The pattern storage provides 8 independent slots for saving custom displays:

| Slot | Address      | Size   |
|------|--------------|--------|
| 0    | 0x100E0000   | 8KB    |
| 1    | 0x100E2000   | 8KB    |
| 2    | 0x100E4000   | 8KB    |
| 3    | 0x100E6000   | 8KB    |
| 4    | 0x100E8000   | 8KB    |
| 5    | 0x100EA000   | 8KB    |
| 6    | 0x100EC000   | 8KB    |
| 7    | 0x100ED800   | ~6KB   |

Each slot can store either:
- A **static pattern**: Single 9×34 greyscale frame (306 bytes)
- An **animation**: Up to 16 frames with configurable delay (not yet fully implemented in CLI)

### Pattern Wear Leveling

Like configuration storage, pattern storage uses wear leveling to extend flash life:

1. **Entry Size**: Each pattern entry is 512 bytes (header + frame data)
2. **Entries Per Slot**: Each 8KB slot holds up to 16 entries
3. **Sequential Writes**: New patterns are written to the next empty entry position
4. **Sequence Numbers**: Each entry has a sequence number to identify the latest version
5. **Erase on Full**: Only when a slot is completely full is it erased

```
Pattern Slot Structure (8KB per slot):

┌─────────────────────────────────────────────────────┐
│ Entry 0: [Header 32B][Frame 306B][Padding]  512B    │
│ Entry 1: [Header 32B][Frame 306B][Padding]  512B    │
│ Entry 2: [Empty - 0xFF]                     512B    │
│ ...                                                  │
│ Entry 15: [Empty - 0xFF]                    512B    │
└─────────────────────────────────────────────────────┘
```

When saving a pattern:
1. Find current highest sequence number in the slot
2. Find next empty entry position
3. If slot is full, erase both pages and start at entry 0
4. Write new entry with incremented sequence number

When loading:
1. Scan all entries for valid headers (magic bytes)
2. Find entry with highest sequence number
3. Verify CRC and return pattern data

This means you can write to the same slot ~16 times before triggering an erase cycle.

### Pattern Format

```
Pattern Header (32 bytes):
┌─────────────────────────────────────────────────────┐
│ magic[0]     │ 0xAA - First magic byte              │
│ magic[1]     │ 0x01 - Second magic byte             │
│ slot         │ Slot number (0-7)                    │
│ pattern_type │ 0 = static, 1 = animation            │
│ frame_count  │ Number of frames (1-16)              │
│ frame_delay  │ Delay between frames in ms           │
│ data_crc     │ CRC16 of frame data (2 bytes)        │
│ sequence     │ Sequence number for wear leveling    │
│ reserved     │ Padding to 32 bytes                  │
└─────────────────────────────────────────────────────┘

Frame Data (306 bytes per frame):
┌─────────────────────────────────────────────────────┐
│ pixels[0][0..34]   │ Column 0, rows 0-33 (greyscale)│
│ pixels[1][0..34]   │ Column 1, rows 0-33            │
│ ...                │                                 │
│ pixels[8][0..34]   │ Column 8, rows 0-33            │
└─────────────────────────────────────────────────────┘
```

## CLI Reference

### Rust CLI

The `inputmodule-control` tool provides the following storage-related options:

#### Pattern Commands

```bash
# Save the current display to a flash slot
inputmodule-control ledmatrix --save-pattern <SLOT>
# SLOT: 0-7

# Load and display a pattern from flash
inputmodule-control ledmatrix --load-pattern <SLOT>
# SLOT: 0-7

# Delete a pattern from flash
inputmodule-control ledmatrix --delete-pattern <SLOT>
# SLOT: 0-7

# List all pattern slots and their status
inputmodule-control ledmatrix --list-patterns
```

#### Configuration Commands

```bash
# Save current runtime settings to flash
inputmodule-control ledmatrix --save-config

# Display stored configuration
inputmodule-control ledmatrix --get-config

# Reset configuration to factory defaults
inputmodule-control ledmatrix --reset-config

# Set default brightness (saves immediately)
inputmodule-control ledmatrix --set-default-brightness <0-255>

# Set sleep timeout in seconds (0 = disabled)
inputmodule-control ledmatrix --set-sleep-timeout <SECONDS>

# Set startup pattern slot (0-7, or 255 for none)
inputmodule-control ledmatrix --set-startup-pattern <SLOT>

# Enable/disable startup animation
inputmodule-control ledmatrix --set-startup-animation <true|false>
```

### Python CLI

The Python library provides equivalent functions:

```python
from inputmodule.inputmodule import find_devs
from inputmodule import ledmatrix

# Find the LED matrix device
devs = find_devs()
dev = devs[0]

# Pattern functions
ledmatrix.save_pattern(dev, slot=0)      # Save current display
ledmatrix.load_pattern(dev, slot=0)      # Load pattern
ledmatrix.delete_pattern(dev, slot=0)    # Delete pattern
ledmatrix.list_patterns(dev)             # List all slots

# Configuration functions
ledmatrix.save_config(dev)               # Save current settings
ledmatrix.get_config(dev)                # Get stored config (returns dict)
ledmatrix.reset_config(dev)              # Reset to defaults

# Individual setting functions (save immediately)
ledmatrix.set_default_brightness(dev, 200)
ledmatrix.set_sleep_timeout(dev, 300)    # 5 minutes
ledmatrix.set_startup_pattern(dev, 0)    # Use slot 0 on startup
ledmatrix.set_startup_animation(dev, False)  # Disable random animation
```

## USB Protocol

All storage commands use the standard Framework input module protocol:

```
Request: [0x32, 0xAC, CommandID, ...parameters]
Response: 32 bytes (command-specific)
```

### Command Reference

| Command       | ID   | Parameters        | Response                              |
|---------------|------|-------------------|---------------------------------------|
| SavePattern   | 0x30 | 1B: slot (0-7)    | 1B: success (1) or failure (0)        |
| LoadPattern   | 0x31 | 1B: slot (0-7)    | 1B: success (1) or not found (0)      |
| DeletePattern | 0x32 | 1B: slot (0-7)    | 1B: success (1) or failure (0)        |
| ListPatterns  | 0x33 | none              | 32B: 8 slots × 4 bytes each           |
| SaveConfig    | 0x34 | none              | 1B: success (1) or failure (0)        |
| GetConfig     | 0x35 | none              | 16B: StoredConfig struct              |
| ResetConfig   | 0x36 | none              | 1B: success (1) or failure (0)        |
| SetConfigValue| 0x37 | 1B key + 2B value | 1B: success (1) or failure (0)        |

### ListPatterns Response Format

```
For each slot (4 bytes):
  Byte 0: occupied (1) or empty (0)
  Byte 1: type (0=static, 1=animation)
  Byte 2: frame count
  Byte 3: frame delay in ms
```

### GetConfig Response Format

```
Byte  0:    Config version (currently 1)
Byte  1:    Default brightness (0-255)
Bytes 2-3:  Sleep timeout in seconds (little-endian u16)
Bytes 4-7:  Animation period in microseconds (little-endian u32)
Byte  8:    PWM frequency index (0-3)
Byte  9:    Startup animation enabled (0 or 1)
Byte  10:   Startup pattern slot (0-7, or 0xFF for none)
Bytes 11-15: Reserved
```

### SetConfigValue Keys

| Key  | Name              | Value Range        |
|------|-------------------|--------------------|
| 0x01 | DefaultBrightness | 0-255              |
| 0x02 | SleepTimeout      | 0-65535 seconds    |
| 0x03 | StartupPattern    | 0-7, or 255 (none) |
| 0x04 | StartupAnimation  | 0 (off) or 1 (on)  |

## Usage Examples

### Example 1: Save a Custom Boot Logo

```bash
# 1. Display your custom image
inputmodule-control ledmatrix --image-gray my-logo.png

# 2. Save it to slot 0
inputmodule-control ledmatrix --save-pattern 0

# 3. Set it as the startup pattern
inputmodule-control ledmatrix --set-startup-pattern 0

# 4. Disable the random animation
inputmodule-control ledmatrix --set-startup-animation false
```

Now your custom logo will display every time the laptop powers on.

### Example 2: Configure Sleep Behavior

```bash
# Set brightness to 50% for battery saving
inputmodule-control ledmatrix --set-default-brightness 128

# Auto-sleep after 5 minutes of inactivity
inputmodule-control ledmatrix --set-sleep-timeout 300

# View current configuration
inputmodule-control ledmatrix --get-config
```

### Example 3: Manage Multiple Patterns (Python)

```python
from inputmodule.inputmodule import find_devs
from inputmodule import ledmatrix

dev = find_devs()[0]

# Save different patterns to different slots
ledmatrix.pattern(dev, "All LEDs on")
ledmatrix.save_pattern(dev, 0)

ledmatrix.pattern(dev, "Zigzag")
ledmatrix.save_pattern(dev, 1)

ledmatrix.show_string(dev, "HELLO")
ledmatrix.save_pattern(dev, 2)

# List what we've saved
ledmatrix.list_patterns(dev)
# Output:
# Pattern slots:
#   Slot 0: static
#   Slot 1: static
#   Slot 2: static
#   Slot 3: empty
#   ...

# Switch between them
ledmatrix.load_pattern(dev, 1)  # Show zigzag
```

### Example 4: Factory Reset

```bash
# Reset all configuration to defaults
inputmodule-control ledmatrix --reset-config

# Optionally clear all saved patterns
for i in {0..7}; do
  inputmodule-control ledmatrix --delete-pattern $i
done
```

## Technical Notes

### Flash Write Considerations

- Flash erase operations require erasing entire 4KB sectors
- Write operations can only change bits from 1 to 0
- Erasing sets all bits back to 1 (0xFF)
- Interrupts are disabled during flash operations to prevent conflicts with XIP (execute-in-place)

### Firmware Update Compatibility

The storage regions are placed after the firmware region, so updating firmware via UF2 or other methods will not erase saved patterns or configuration. However:

- A firmware update that changes the storage format may need to reset configuration
- The config version field allows detection of incompatible formats
- Factory reset is always available via `--reset-config`

### Error Handling

All storage operations return success/failure status. Common failure cases:

- Invalid slot number (must be 0-7)
- Flash write error (hardware failure)
- Corrupted data (CRC mismatch) - returns defaults instead
- Empty slot on load - returns failure, no display change
