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
| Version      | 0x20 |   ` D ` |  3 Bytes |            | Get firmware version     |

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

```
Byte 0: USB bcdDevice MSB
Byte 1: USB bcdDevice LSB
Byte 2: 1 if pre-release version, 0 otherwise
```
