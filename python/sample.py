import time
from datetime import datetime
#from inputmodule.ledmatrix import LedMatrix, Pattern

# TODO: Import
from enum import IntEnum
class PatternVals(IntEnum):
    FullBrightness = 0x05
class CommandVals(IntEnum):
    Brightness = 0x00
    Pattern = 0x01

class Pattern:
    width = 9
    height = 34
    vals = []

    def percentage(p):
        Pattern()

    def from_string(s):
        Pattern()

    def set(self, x, y, val):
        pass

class InputModule:
    pass

class LedMatrix(object):
    def __init__(self, name):
        self.name = name
        self.fw_version = "0.1.9"
        self.sleeping = True
        self.brightness = 100

    def find_all():
        return [1, 2]

    def __enter__(self):
        #print('entering')
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        pass
        #print('leaving')

    def enter_bootloader(self):
        pass

    def raw_command(self, command, vals):
        pass

    def set_bw(self, pattern):
        pass

    def set_grayscale(self, pattern):
        pass

    # TODO: Properties for things like sleeping, brightness, ...
    @property
    def radius(self):
        """The radius property."""
        print("Get radius")
        return self._radius

    @radius.setter
    def radius(self, value):
        print("Set radius")
        self._radius = value


# 
matrices = LedMatrix.find_all()
print(f"{len(matrices)} LED Matrices connected to the system")

with LedMatrix("COM1") as matrix:
    print(f"Firmware version: {matrix.fw_version}")

    print(f"Sleep status: {matrix.sleeping}")
    print(f"Going to sleep and back")
    matrix.sleeping = True
    matrix.sleeping = False

    print(f"Current brightness: {matrix.brightness}")
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
        #PatternVals.Gradient,
        #PatternVals.DoubleGradient,
        #PatternVals.DisplayLotus,
        #PatternVals.ZigZag,
        #PatternVals.DisplayPanic,
        #PatternVals.DisplayLotus2
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
