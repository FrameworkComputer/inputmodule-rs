# Commands

The input modules can be controlled by sending commands via the USB CDC-ACM
serial port. To send a command, write the two magic bytes, command ID and
parameters. Most commands don't return anything.

Simple example in Python:

```python
import serial

def send_command(command_id, parameters, with_response=False):
  with serial.Serial("/dev/ttyACM0", 115200) as s:
      s.write([0x32, 0xAC, command_id] + parameters)

      if with_response:
          res = s.read(32)
          return res

# Go to sleep and check the status
send_command(0x03, [True])
res = send_command(0x03, [], with_response=True)
print(f"Is currently sleeping: {bool(res[0])}")
```

Many commands support setting and writing a value, with the same command ID.
When no parameters are given, the current value is queried and returned.

###### Modules:

- L = LED Matrix
- D = B1 Display
- M = C1 Minimal Module

## Command overview

| Command      |   ID | Modules | Response | Parameters | Behavior                 |
| ------------ | ---- | ------- | -------- | ---------- | ------------------------ |
| Brightness   | 0x00 |   `L M` |          |            | Set LED brightness       |
| Pattern      | 0x01 |   `L  ` |          |            | Display a pattern        |
| Bootloader   | 0x02 |   `LDM` |          |            | Jump to the bootloader   |
| Sleep        | 0x03 |   `LDM` |          |       bool | Go to sleep or wake up   |
| GetSleep     | 0x03 |   `LDM` |     bool |            | Check sleep state        |
| Animate      | 0x04 |   `L  ` |          |       bool | Scroll current pattern   |
| GetAnimate   | 0x04 |   `L  ` |     bool |            | Check whether animating  |
| Panic        | 0x05 |   `LDM` |          |            | Cause a FW panic/crash   |
| DrawBW       | 0x06 |   `L  ` |          |   39 Bytes | Draw a black/white image |
| StageCol     | 0x07 |   `L  ` |          | 1+34 Bytes | Send a greyscale column  |
| FlushCols    | 0x08 |   `L  ` |          |            | Flush/draw all columns   |
| SetText      | 0x09 |   ` D ` |          |            | TODO: Remove             |
| StartGame    | 0x10 |   `L  ` |          | 1B Game ID | Start an embeded game    |
| GameCtrl     | 0x11 |   `L  ` |          | 1B Control | Send a game command      |
| GameStatus   | 0x12 |   `L  ` |      WIP |            | Check the game status    |
| SetColor     | 0x13 |   `  M` |          |    3B: RGB | Set the LED's color      |
| DisplayOn    | 0x14 |   ` D ` |          |       bool | Turn the display on/off  |
| InvertScreen | 0x15 |   ` D ` |          |       bool | Invert scren on/off      |
| SetPxCol     | 0x16 |   ` D ` |          |   50 Bytes | Send a column of pixels  |
| FlushFB      | 0x17 |   ` D ` |          |            | Flush all columns        |
| Version      | 0x20 |   `LDM` |  3 Bytes |            | Get firmware version     |
| SavePattern  | 0x30 |   `L  ` |   1 Byte |     1B Slot| Save display to flash    |
| LoadPattern  | 0x31 |   `L  ` |   1 Byte |     1B Slot| Load pattern from flash  |
| DeletePattern| 0x32 |   `L  ` |   1 Byte |     1B Slot| Delete pattern from flash|
| ListPatterns | 0x33 |   `L  ` |  32 Bytes|            | List all pattern slots   |
| SaveConfig   | 0x34 |   `L  ` |   1 Byte |            | Save config to flash     |
| GetConfig    | 0x35 |   `L  ` |  16 Bytes|            | Get stored config        |
| ResetConfig  | 0x36 |   `L  ` |   1 Byte |            | Reset config to defaults |
| SetConfigVal | 0x37 |   `L  ` |   1 Byte |   3B K+Val | Set config value + save  |

#### Pattern (0x01)

The following patterns are defined

- 0x00 - Percentage (See below, needs another parameter)
- 0x01 - Gradient (Brightness gradient from top to bottom)
- 0x02 - DoubleGradient (Brightness gradient from the middle to top and bottom)
- 0x03 - DisplayLotusHorizontal (Display "LOTUS" 90 degree rotated)
- 0x04 - ZigZag (Display a zigzag pattern)
- 0x05 - FullBrightness (Turn every LED on and set the brightness to 100%)
- 0x06 - DisplayPanic (Display the string "PANIC")
- 0x07 - DisplayLotusVertical (Display the string "LOTUS")

Pattern 0x00 is special. It needs another parameter to specify the percentage.
It will fill a percentage of the screen. It can serve as a progress indicator.

#### DrawBW (0x06)
TODO

#### StageCol (0x07)
TODO

#### FlushCols (0x08)
TODO

#### SetPxCol (0x16)
TODO

#### FlushFB (0x17)
TODO

#### Version (0x20)

Response:

```plain
Byte 0: USB bcdDevice MSB
Byte 1: USB bcdDevice LSB
Byte 2: 1 if pre-release version, 0 otherwise

+-- Major version
|        +-- Minor version
|        |   +-- Patch version
|        |   |           +-- 1 if is pre-release,
|        |   |           |   0 otherwise
MMMMMMMM mmmmPPPP 0000000p
```

## Flash Storage Commands

The LED Matrix module supports persistent storage of patterns and configuration
in flash memory. Storage is placed at the end of flash (before the serial number)
so it survives firmware updates.

**Flash Layout:**
- Pattern Storage: 0x100E0000, 60KB (8 pattern slots)
- Config Storage: 0x100EF000, 64KB (wear-leveled configuration)
- Serial Number: 0x100FF000, 4KB (read-only, untouched)

#### SavePattern (0x30)

Save the current display to a flash pattern slot.

Parameters:
- Byte 0: Slot number (0-7)

Response:
- Byte 0: 1 if successful, 0 otherwise

#### LoadPattern (0x31)

Load and display a pattern from a flash slot.

Parameters:
- Byte 0: Slot number (0-7)

Response:
- Byte 0: 1 if successful (pattern found), 0 if slot empty

#### DeletePattern (0x32)

Delete a pattern from a flash slot (erases the flash sector).

Parameters:
- Byte 0: Slot number (0-7)

Response:
- Byte 0: 1 if successful, 0 otherwise

#### ListPatterns (0x33)

List metadata for all 8 pattern slots.

Response (32 bytes):
For each slot (4 bytes per slot):
- Byte 0: 1 if occupied, 0 if empty
- Byte 1: Pattern type (0=static, 1=animation)
- Byte 2: Frame count
- Byte 3: Frame delay in milliseconds

#### SaveConfig (0x34)

Save current runtime settings to flash.

Response:
- Byte 0: 1 if successful, 0 otherwise

#### GetConfig (0x35)

Get the stored configuration from flash.

Response (16 bytes):
```plain
Byte  0: Config version (currently 1)
Byte  1: Default brightness (0-255)
Byte  2-3: Sleep timeout in seconds (little-endian, 0=disabled)
Byte  4-7: Animation period in microseconds (little-endian)
Byte  8: PWM frequency index (0-3)
Byte  9: Startup animation enabled (0 or 1)
Byte 10: Startup pattern slot (0-7, or 0xFF for none)
Byte 11-15: Reserved
```

#### ResetConfig (0x36)

Reset configuration to factory defaults and save to flash.

Response:
- Byte 0: 1 if successful, 0 otherwise

#### SetConfigValue (0x37)

Set a specific configuration value and save to flash.

Parameters:
- Byte 0: Config key
- Byte 1-2: Value (little-endian u16)

Config keys:
- 0x01: Default brightness (value: 0-255)
- 0x02: Sleep timeout (value: seconds, 0=disabled)
- 0x03: Startup pattern slot (value: 0-7, or 255 for none)
- 0x04: Startup animation (value: 0=disabled, 1=enabled)

Response:
- Byte 0: 1 if successful, 0 otherwise
