import time
from datetime import datetime
# from inputmodule.inputmodule.ledmatrix import LedMatrix, Pattern

FWK_MAGIC = [0x32, 0xAC]
FWK_VID = 0x32AC
LED_MATRIX_PID = 0x20
QTPY_PID = 0x001F
INPUTMODULE_PIDS = [LED_MATRIX_PID, QTPY_PID]
RESPONSE_SIZE = 32

import serial

# TODO: Import
from enum import IntEnum


class PatternVals(IntEnum):
    Percentage = 0x00
    Gradient = 0x01
    DoubleGradient = 0x02
    DisplayLotus = 0x03
    ZigZag = 0x04
    FullBrightness = 0x05
    DisplayPanic = 0x06
    DisplayLotus2 = 0x07


class CommandVals(IntEnum):
    Brightness = 0x00
    Pattern = 0x01
    BootloaderReset = 0x02
    Sleep = 0x03
    Animate = 0x04
    Panic = 0x05
    Draw = 0x06
    StageGreyCol = 0x07
    DrawGreyColBuffer = 0x08
    SetText = 0x09
    StartGame = 0x10
    GameControl = 0x11
    GameStatus = 0x12
    SetColor = 0x13
    DisplayOn = 0x14
    InvertScreen = 0x15
    SetPixelColumn = 0x16
    FlushFramebuffer = 0x17
    ClearRam = 0x18
    ScreenSaver = 0x19
    SetFps = 0x1A
    SetPowerMode = 0x1B
    PwmFreq = 0x1E
    DebugMode = 0x1F
    Version = 0x20


class Pattern:
    width = 9
    height = 34

    def __init__(self):
        """Empty pattern with all LEDs off"""
        self._vals = [[0 for _ in range(self.height)] for _ in range(self.width)]

    def percentage(p):
        """A percentage of LEDs on, increasing vertically from the bottom"""
        pattern = Pattern()
        pattern._vals = [
            [(0xFF if (y * 100 / 34 > p) else 0) for y in range(pattern.height)]
            for _ in range(pattern.width)
        ]
        return pattern

    def from_string(s):
        # TODO
        return Pattern()

    def set(self, x, y, val):
        """Set a specific LED to a brightness value"""
        assert val >= 0 and val <= 0xFF
        assert x >= 0 and x <= self.width
        assert y >= 0 and y <= self.height
        self._vals[x][y] = val

    def to_bw_vals(self):
        """To list of 39 byte values [Int]"""
        vals = [0x00 for _ in range(39)]
        for x, col in enumerate(self._vals):
            for y, val in enumerate(col):
                if val == 0xFF:
                    i = x + self.width * y
                    vals[int(i / 8)] |= 1 << i % 8
        return vals

    def to_gray_vals(self):
        """To [[]]"""
        return self._vals


class ModuleNotFoundException(Exception):
    pass


class LedMatrix(object):
    def __init__(self, dev_path=None):
        self.dev_path = dev_path

        if dev_path is None:
            pass

        self.fw_version = "0.1.9"
        self.sleeping = True
        # self.brightness = 100
        self.dev_path = "/dev/tty0"
        # self.dev_path = None
        self.dev = None
        # TODO: Check if it's there
        # raise ModuleNotFoundException(f"Module {port} not found")
        if False:
            raise ModuleNotFoundException("No Module found")

    def from_port(port):
        """Connect to an LED matrix by specifying the serial port name/path"""
        return LedMatrix(port)

    def left():
        """Connect to the left LED matrix"""
        # TODO
        raise ModuleNotFoundException("Left Module not found")

    def right():
        """Connect to the right LED matrix"""
        # TODO
        raise ModuleNotFoundException("Right Module not found")

    def list_ports():
        """List all serial ports with LED matrices"""
        return ["/dev/ttyACM0"]

    def __enter__(self):
        self.dev = serial.Serial(self.dev_path, 115200)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.dev.close()

    def enter_bootloader(self):
        """Put the module in bootloader mode to update the firmware"""
        self.raw_command(CommandVals.BootloaderReset, [0x00])

    def raw_command(self, command, parameters=[], with_response=False):
        """Send a raw command with command ID and payload bytes"""
        vals = FWK_MAGIC + [command] + parameters
        self.dev.write(command)
        if with_response:
            res = self.dev.read(RESPONSE_SIZE)
            return res

    def set_bw(self, pattern):
        """Draw a black and white pattern on the LED matrix"""
        vals = pattern.to_bw_vals()
        self.raw_command(CommandVals.Draw, vals)

    def set_grayscale(self, pattern):
        """Draw a greyscale pattern on the LED matrix"""
        for x in range(0, pattern.width):
            vals = pattern.to_gray_vals()[x]
            self._send_col(x, vals)
        self._commit_cols()

    def _send_col(self, x, vals):
        """Stage greyscale values for a single column. Must be committed with commit_cols()"""
        command = FWK_MAGIC + [CommandVals.StageGreyCol, x] + vals
        self.dev.write(command)

    def _commit_cols(self):
        """Commit the changes from sending individual cols with send_col(), displaying the matrix.
        This makes sure that the matrix isn't partially updated."""
        command = FWK_MAGIC + [CommandVals.DrawGreyColBuffer, 0x00]
        self.dev.write(command)

    # TODO: Properties for things like sleeping, brightness, ...
    @property
    def brightness(self):
        """Get current module brightness"""
        res = self.raw_command(CommandVals.Brightness, with_response=True)
        return int(res[0])

    @brightness.setter
    def brightness(self, value):
        """Change brightness"""
        self.raw_command(CommandVals.Brightness, [value])


def demo_interaction(matrix):
    print(f"Firmware version: {matrix.fw_version}")

    print(f"Sleep status: {matrix.sleeping}")
    print(f"Going to sleep and back")
    matrix.sleeping = True
    matrix.sleeping = False

    # print(f"Current brightness: {matrix.brightness}")
    print("Setting 100% brightness")
    matrix.brightness = 100
    print("Setting  50% brightness")
    matrix.brightness = 50

    # TODO
    # matrix.pwm_freq
    # matrix.animate
    # matrix.animate = True
    # matrix.animate = False

    # Enter bootloader to prepare for flashing
    # To exit bootloader, either flash the firmware or re-plug the device
    # matrix.enter_bootloader()

    print("Iterating through a couple of built-in patterns, 1s each")
    pattern_commands = [
        PatternVals.FullBrightness,
        PatternVals.Gradient,
        PatternVals.DoubleGradient,
        PatternVals.DisplayLotus,
        PatternVals.ZigZag,
        PatternVals.DisplayPanic,
        PatternVals.DisplayLotus2,
    ]
    for pattern in pattern_commands:
        matrix.raw_command(CommandVals.Pattern, [pattern])
        time.sleep(1)

    print("Iterating through a couple of black/white patterns, 1s each")
    bw_patterns = [
        Pattern.percentage(50),
    ]
    for pattern in bw_patterns:
        matrix.set_bw(pattern)
        time.sleep(1)

    # Demonstrate gray-scale pattern
    print("Show all 255 brightness levels, one per LED")
    pattern = Pattern()
    for x in range(0, pattern.width):
        for y in range(pattern.height):
            brightness = x + pattern.width * y
            if brightness > 255:
                pattern.set(x, y, 0)
            else:
                pattern.set(x, y, brightness)
    matrix.set_grayscale(pattern)

    # Show current time
    current_time = datetime.now().strftime("%H:%M")
    print("Current Time =", current_time)
    matrix.set_bw(Pattern.from_string(current_time))


def demo():
    matrices = LedMatrix.list_ports()
    print(f"{len(matrices)} LED Matrices connected to the system")

    try:
        # Open specific
        with LedMatrix.from_port("COM1") as matrix:
            pass
    except ModuleNotFoundException as e:
        print(e)

    # Left and right matrix, fails if only one is connected
    try:
        with LedMatrix.left() as left_matrix, LedMatrix.right() as right_matrix:
            print(f"Left: {left_matrix}, Right: {right_matrix}")
    except ModuleNotFoundException as e:
        print(e)

    # Choose first available matrix (best if just one is connected)
    try:
        with LedMatrix() as matrix:
            demo_interaction(matrix)
    except ModuleNotFoundException as e:
        print(e)


if __name__ == "__main__":
    demo()
