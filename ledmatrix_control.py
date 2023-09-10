#!/usr/bin/env python3
import argparse
import math
import os
import random
import sys
import threading
import time
from datetime import datetime, timedelta
from enum import IntEnum

# Need to install
import serial
from serial.tools import list_ports

# Optional dependencies:
# from PIL import Image
# import PySimpleGUI as sg

FWK_MAGIC = [0x32, 0xAC]
FWK_VID = 0x32AC
LED_MATRIX_PID = 0x20
INPUTMODULE_PIDS = [LED_MATRIX_PID]


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


class Game(IntEnum):
    Snake = 0x00
    Pong = 0x01
    Tetris = 0x02
    GameOfLife = 0x03


class PatternVals(IntEnum):
    Percentage = 0x00
    Gradient = 0x01
    DoubleGradient = 0x02
    DisplayLotus = 0x03
    ZigZag = 0x04
    FullBrightness = 0x05
    DisplayPanic = 0x06
    DisplayLotus2 = 0x07


class GameOfLifeStartParam(IntEnum):
    Currentmatrix = 0x00
    Pattern1 = 0x01
    Blinker = 0x02
    Toad = 0x03
    Beacon = 0x04
    Glider = 0x05

    def __str__(self):
        return self.name.lower()

    def __repr__(self):
        return str(self)

    @staticmethod
    def argparse(s):
        try:
            return GameOfLifeStartParam[s.lower().capitalize()]
        except KeyError:
            return s


class GameControlVal(IntEnum):
    Up = 0
    Down = 1
    Left = 2
    Right = 3
    Quit = 4

PWM_FREQUENCIES = [
    '29kHz',
    '3.6kHz',
    '1.8kHz',
    '900Hz',
]

PATTERNS = [
    'All LEDs on',
    '"LOTUS" sideways',
    'Gradient (0-13% Brightness)',
    'Double Gradient (0-7-0% Brightness)',
    'Zigzag',
    '"PANIC"',
    '"LOTUS" Top Down',
    'All brightness levels (1 LED each)',
    'Every Second Row',
    'Every Third Row',
    'Every Fourth Row',
    'Every Fifth Row',
    'Every Sixth Row',
    'Every Second Col',
    'Every Third Col',
    'Every Fourth Col',
    'Every Fifth Col',
    'Checkerboard',
    'Double Checkerboard',
    'Triple Checkerboard',
    'Quad Checkerboard'
]
DRAW_PATTERNS = ['off', 'on', 'foo']
GREYSCALE_DEPTH = 32
RESPONSE_SIZE = 32
WIDTH = 9
HEIGHT = 34
B1_WIDTH = 300
B1_HEIGHT = 400

ARG_UP = 0
ARG_DOWN = 1
ARG_LEFT = 2
ARG_RIGHT = 3
ARG_QUIT = 4
ARG_2LEFT = 5
ARG_2RIGHT = 6

RGB_COLORS = ['white', 'black', 'red', 'green',
              'blue', 'cyan', 'yellow', 'purple']
SCREEN_FPS = ['quarter', 'half', 'one', 'two',
              'four', 'eight', 'sixteen', 'thirtytwo']
HIGH_FPS_MASK = 0b00010000
LOW_FPS_MASK = 0b00000111

# Global variables
STOP_THREAD = False
DISCONNECTED_DEVS = []


def update_brightness_slider(window, devices):
    average_brightness = None
    for dev in devices:
        if not average_brightness:
            average_brightness = 0

        br = get_brightness(dev)
        average_brightness += br
        print(f"Brightness: {br}")
    if average_brightness:
        window['-BRIGHTNESS-'].update(average_brightness / len(devices))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-l", "--list", help="List all compatible devices", action="store_true")
    parser.add_argument("--bootloader", help="Jump to the bootloader to flash new firmware",
                        action="store_true")
    parser.add_argument('--sleep', help='Simulate the host going to sleep or waking up',
                        action=argparse.BooleanOptionalAction)
    parser.add_argument('--is-sleeping', help='Check current sleep state',
                        action='store_true')
    parser.add_argument("--brightness", help="Adjust the brightness. Value 0-255",
                        type=int)
    parser.add_argument("--get-brightness", help="Get current brightness",
                        action="store_true")
    parser.add_argument('--animate', action=argparse.BooleanOptionalAction,
                        help='Start/stop vertical scrolling')
    parser.add_argument('--get-animate', action='store_true',
                        help='Check if currently animating')
    parser.add_argument("--pwm", help="Adjust the PWM frequency. Value 0-255",
                        type=int, choices=[29000, 3600, 1800, 900])
    parser.add_argument("--get-pwm", help="Get current PWM Frequency",
                        action="store_true")
    parser.add_argument("--pattern", help='Display a pattern',
                        type=str, choices=PATTERNS)
    parser.add_argument("--image", help="Display a PNG or GIF image in black and white only)",
                        type=argparse.FileType('rb'))
    parser.add_argument("--image-grey", help="Display a PNG or GIF image in greyscale",
                        type=argparse.FileType('rb'))
    parser.add_argument("--percentage", help="Fill a percentage of the screen",
                        type=int)
    parser.add_argument("--clock", help="Display the current time",
                        action="store_true")
    parser.add_argument("--string", help="Display a string or number, like FPS",
                        type=str)
    parser.add_argument("--symbols", help="Show symbols (degF, degC, :), snow, cloud, ...)",
                        nargs='+')
    parser.add_argument("--gui", help="Launch the graphical version of the program",
                        action="store_true")
    parser.add_argument("--panic", help="Crash the firmware (TESTING ONLY)",
                        action="store_true")
    parser.add_argument("--blink", help="Blink the current pattern",
                        action="store_true")
    parser.add_argument("--breathing", help="Breathing of the current pattern",
                        action="store_true")
    parser.add_argument("--eq", help="Equalizer", nargs='+', type=int)
    parser.add_argument(
        "--random-eq", help="Random Equalizer", action="store_true")
    parser.add_argument("--wpm", help="WPM Demo", action="store_true")
    parser.add_argument("--snake", help="Snake", action="store_true")
    parser.add_argument("--snake-embedded",
                        help="Snake on the module", action="store_true")
    parser.add_argument("--pong-embedded",
                        help="Pong on the module", action="store_true")
    parser.add_argument("--game-of-life-embedded",
                        help="Game of Life", type=GameOfLifeStartParam.argparse, choices=list(GameOfLifeStartParam))
    parser.add_argument("--quit-embedded-game",
                        help="Quit the current game", action="store_true")
    parser.add_argument(
        "--all-brightnesses", help="Show every pixel in a different brightness", action="store_true")
    parser.add_argument(
        "--set-color", help="Set RGB color (C1 Minimal Input Module)", choices=RGB_COLORS)
    parser.add_argument(
        "--get-color", help="Get RGB color (C1 Minimal Input Module)", action="store_true")
    parser.add_argument("-v", "--version",
                        help="Get device version", action="store_true")
    parser.add_argument(
        "--serial-dev", help="Change the serial dev. Probably /dev/ttyACM0 on Linux, COM0 on Windows")

    parser.add_argument(
        "--disp-str", help="Display a string on the LCD Display", type=str)
    parser.add_argument("--display-on", help="Control display power",
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("--invert-screen", help="Invert display",
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("--screen-saver", help="Turn on/off screensaver",
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("--set-fps", help="Set screen FPS",
                        choices=SCREEN_FPS)
    parser.add_argument("--set-power-mode", help="Set screen power mode",
                        choices=['high', 'low'])
    parser.add_argument("--get-fps", help="Set screen FPS",
                        action='store_true')
    parser.add_argument("--get-power-mode", help="Set screen power mode",
                        action='store_true')
    parser.add_argument("--b1image", help="On the B1 display, show a PNG or GIF image in black and white only)",
                        type=argparse.FileType('rb'))

    args = parser.parse_args()

    # Selected device
    dev = None
    ports = find_devs()

    if args.list:
        print_devs(ports)
        sys.exit(0)

    if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
        # Force GUI in pyinstaller bundled app
        args.gui = True

    if not ports:
        print("No device found")
        popup(args.gui, "No device found")
        sys.exit(1)
    elif args.serial_dev is not None:
        dev = [x for x in ports if ports.name == args.serial_dev]
    elif len(ports) == 1:
        dev = ports[0]
    elif len(ports) >= 1 and not args.gui:
        popup(args.gui, "More than 1 compatibles devices found. Please choose from the commandline with --serial-dev COMX.\nConnected ports:\n- {}".format("\n- ".join([port.device for port in ports])))
        print("More than 1 compatible device found. Please choose with --serial-dev ...")
        print("Example on Windows: --serial-dev COM3")
        print("Example on Linux:   --serial-dev /dev/ttyACM0")
        print_devs(ports)
        sys.exit(1)
    elif args.gui:
        # TODO: Allow selection in GUI
        print("Select in GUI")

    if not args.gui and dev is None:
        print("No device selected")
        popup(args.gui, "No device selected")
        sys.exit(1)

    if args.bootloader:
        bootloader(dev)
    elif args.sleep is not None:
        send_command(dev, CommandVals.Sleep, [args.sleep])
    elif args.is_sleeping:
        res = send_command(dev, CommandVals.Sleep, with_response=True)
        sleeping = bool(res[0])
        print(f"Currently sleeping: {sleeping}")
    elif args.brightness is not None:
        if args.brightness > 255 or args.brightness < 0:
            print("Brightness must be 0-255")
            sys.exit(1)
        brightness(dev, args.brightness)
    elif args.get_brightness:
        br = get_brightness(dev)
        print(f"Current brightness: {br}")
    elif args.pwm is not None:
        if args.pwm == 29000:
            pwm_freq(dev, '29kHz')
        elif args.pwm == 3600:
            pwm_freq(dev, '3.6kHz')
        elif args.pwm == 1800:
            pwm_freq(dev, '1.8kHz')
        elif args.pwm == 900:
            pwm_freq(dev, '900Hz')
    elif args.get_pwm:
        p = get_pwm_freq(dev)
        print(f"Current PWM Frequency: {p} Hz")
    elif args.percentage is not None:
        if args.percentage > 100 or args.percentage < 0:
            print("Percentage must be 0-100")
            sys.exit(1)
        percentage(dev, args.percentage)
    elif args.pattern is not None:
        pattern(dev, args.pattern)
    elif args.animate is not None:
        animate(dev, args.animate)
    elif args.get_animate:
        animating = get_animate(dev)
        print(f"Currently animating: {animating}")
    elif args.panic:
        send_command(dev, CommandVals.Panic, [0x00])
    elif args.image is not None:
        image_bl(dev, args.image)
    elif args.image_grey is not None:
        image_greyscale(dev, args.image_grey)
    elif args.all_brightnesses:
        all_brightnesses(dev)
    elif args.set_color:
        set_color(dev, args.set_color)
    elif args.get_color:
        (red, green, blue) = get_color(dev)
        print(f"Current color: RGB:({red}, {green}, {blue})")
    elif args.gui:
        devices = find_devs()#show=False, verbose=False)
        print("Found {} devices".format(len(devices)))
        gui(devices)
    elif args.blink:
        blinking(dev)
    elif args.breathing:
        breathing(dev)
    elif args.wpm:
        wpm_demo(dev)
    elif args.snake:
        snake(dev)
    elif args.snake_embedded:
        snake_embedded(dev)
    elif args.game_of_life_embedded is not None:
        game_of_life_embedded(dev, args.game_of_life_embedded)
    elif args.quit_embedded_game:
        send_command(dev, CommandVals.GameControl, [GameControlVal.Quit])
    elif args.pong_embedded:
        pong_embedded(dev)
    elif args.eq is not None:
        eq(dev, args.eq)
    elif args.random_eq:
        random_eq(dev)
    elif args.clock:
        clock(dev)
    elif args.string is not None:
        show_string(dev, args.string)
    elif args.symbols is not None:
        show_symbols(dev, args.symbols)
    elif args.disp_str is not None:
        display_string(dev, args.disp_str)
    elif args.display_on is not None:
        display_on_cmd(dev, args.display_on)
    elif args.invert_screen is not None:
        invert_screen_cmd(dev, args.invert_screen)
    elif args.screen_saver is not None:
        screen_saver_cmd(dev, args.screen_saver)
    elif args.set_fps is not None:
        set_fps_cmd(dev, args.set_fps)
    elif args.set_power_mode is not None:
        set_power_mode_cmd(dev, args.set_power_mode)
    elif args.get_fps:
        get_fps_cmd(dev)
    elif args.get_power_mode:
        get_power_mode_cmd(dev)
    elif args.b1image is not None:
        b1image_bl(dev, args.b1image)
    elif args.version:
        version = get_version(dev)
        print(f"Device version: {version}")
    else:
        parser.print_help(sys.stderr)
        sys.exit(1)


def resource_path():
    """ Get absolute path to resource, works for dev and for PyInstaller"""
    try:
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.abspath(".")

    return base_path


def find_devs():
    ports = list_ports.comports()
    return [port for port in ports if port.vid == 0x32AC and port.pid in INPUTMODULE_PIDS]


def print_devs(ports):
    for port in ports:
        print(f"{port.device}")
        print(f"  VID:     0x{port.vid:04X}")
        print(f"  PID:     0x{port.pid:04X}")
        print(f"  SN:      {port.serial_number}")
        print(f"  Product: {port.product}")


def bootloader(dev):
    """Reboot into the bootloader to flash new firmware"""
    send_command(dev, CommandVals.BootloaderReset, [0x00])


def percentage(dev, p):
    """Fill a percentage of the screen. Bottom to top"""
    send_command(dev, CommandVals.Pattern, [PatternVals.Percentage, p])


def brightness(dev, b: int):
    """Adjust the brightness scaling of the entire screen.
    """
    send_command(dev, CommandVals.Brightness, [b])


def get_brightness(dev):
    """Adjust the brightness scaling of the entire screen.
    """
    res = send_command(dev, CommandVals.Brightness, with_response=True)
    return int(res[0])


def get_pwm_freq(dev):
    """Adjust the brightness scaling of the entire screen.
    """
    res = send_command(dev, CommandVals.PwmFreq, with_response=True)
    freq = int(res[0])
    if freq == 0:
        return 29000
    elif freq == 1:
        return 3600
    elif freq == 2:
        return 1800
    elif freq == 3:
        return 900
    else:
        return None


def get_version(dev):
    """Get the device's firmware version"""
    res = send_command(dev, CommandVals.Version, with_response=True)
    major = res[0]
    minor = (res[1] & 0xF0) >> 4
    patch = res[1] & 0xF
    pre_release = res[2]

    version = f"{major}.{minor}.{patch}"
    if pre_release:
        version += " (Pre-release)"
    return version


def animate(dev, b: bool):
    """Tell the firmware to start/stop animation.
    Scrolls the currently saved grid vertically down."""
    send_command(dev, CommandVals.Animate, [b])


def get_animate(dev):
    """Tell the firmware to start/stop animation.
    Scrolls the currently saved grid vertically down."""
    res = send_command(dev, CommandVals.Animate, with_response=True)
    return bool(res[0])


def b1image_bl(dev, image_file):
    """ Display an image in black and white
    Confirmed working with PNG and GIF.
    Must be 300x400 in size.
    Sends one 400px column in a single commands and a flush at the end
    """

    from PIL import Image
    im = Image.open(image_file).convert("RGB")
    width, height = im.size
    assert (width == B1_WIDTH)
    assert (height == B1_HEIGHT)
    pixel_values = list(im.getdata())

    for x in range(B1_WIDTH):
        vals = [0 for _ in range(50)]

        byte = None
        for y in range(B1_HEIGHT):
            pixel = pixel_values[y*B1_WIDTH + x]
            brightness = sum(pixel) / 3
            black = brightness < 0xFF/2

            bit = y % 8

            if bit == 0:
                byte = 0
            if black:
                byte |= 1 << bit

            if bit == 7:
                vals[int(y/8)] = byte

        column_le = list((x).to_bytes(2, 'little'))
        command = FWK_MAGIC + [0x16] + column_le + vals
        send_command(dev, command)

    # Flush
    command = FWK_MAGIC + [0x17]
    send_command(dev, command)


def image_bl(dev, image_file):
    """Display an image in black and white
    Confirmed working with PNG and GIF.
    Must be 9x34 in size.
    Sends everything in a single command
    """
    vals = [0 for _ in range(39)]

    from PIL import Image
    im = Image.open(image_file).convert("RGB")
    width, height = im.size
    assert (width == 9)
    assert (height == 34)
    pixel_values = list(im.getdata())
    for i, pixel in enumerate(pixel_values):
        brightness = sum(pixel) / 3
        if brightness > 0xFF/2:
            vals[int(i/8)] |= (1 << i % 8)

    send_command(dev, CommandVals.Draw, vals)


def pixel_to_brightness(pixel):
    """Calculate pixel brightness from an RGB triple"""
    assert (len(pixel) == 3)
    brightness = sum(pixel) / len(pixel)

    # Poor man's scaling to make the greyscale pop better.
    # Should find a good function.
    if brightness > 200:
        brightness = brightness
    elif brightness > 150:
        brightness = brightness * 0.8
    elif brightness > 100:
        brightness = brightness * 0.5
    elif brightness > 50:
        brightness = brightness
    else:
        brightness = brightness * 2

    return int(brightness)


def image_greyscale(dev, image_file):
    """Display an image in greyscale
    Sends each 1x34 column and then commits => 10 commands
    """
    with serial.Serial(dev.device, 115200) as s:
        from PIL import Image
        im = Image.open(image_file).convert("RGB")
        width, height = im.size
        assert (width == 9)
        assert (height == 34)
        pixel_values = list(im.getdata())
        for x in range(0, WIDTH):
            vals = [0 for _ in range(HEIGHT)]

            for y in range(HEIGHT):
                vals[y] = pixel_to_brightness(pixel_values[x+y*WIDTH])

            send_col(s, x, vals)
        commit_cols(s)


def send_col(s, x, vals):
    """Stage greyscale values for a single column. Must be committed with commit_cols()"""
    command = FWK_MAGIC + [CommandVals.StageGreyCol, x] + vals
    send_serial(s, command)


def commit_cols(s):
    """Commit the changes from sending individual cols with send_col(), displaying the matrix.
    This makes sure that the matrix isn't partially updated."""
    command = FWK_MAGIC + [CommandVals.DrawGreyColBuffer, 0x00]
    send_serial(s, command)


def get_color():
    res = send_command(dev, CommandVals.SetColor, with_response=True)
    return (int(res[0]), int(res[1]), int(res[2]))


def set_color(color):
    rgb = None
    if color == 'white':
        rgb = [0xFF, 0xFF, 0xFF]
    elif color == 'black':
        rgb = [0x00, 0x00, 0x00]
    elif color == 'red':
        rgb = [0xFF, 0x00, 0x00]
    elif color == 'green':
        rgb = [0x00, 0xFF, 0x00]
    elif color == 'blue':
        rgb = [0x00, 0x00, 0xFF]
    elif color == 'yellow':
        rgb = [0xFF, 0xFF, 0x00]
    elif color == 'cyan':
        rgb = [0x00, 0xFF, 0xFF]
    elif color == 'purple':
        rgb = [0xFF, 0x00, 0xFF]
    else:
        print(f"Unknown color: {color}")
        return

    if rgb:
        send_command(dev, CommandVals.SetColor, rgb)


def checkerboard(dev, n):
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = (([0xFF] * n) + ([0x00] * n)) * int(HEIGHT/2)
            if x % (n*2) < n:
                # Rotate once
                vals = vals[n:] + vals[:n]

            send_col(s, x, vals)
        commit_cols(s)


def every_nth_col(dev, n):
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = [(0xFF if x % n == 0 else 0) for _ in range(HEIGHT)]

            send_col(s, x, vals)
        commit_cols(s)


def every_nth_row(dev, n):
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = [(0xFF if y % n == 0 else 0) for y in range(HEIGHT)]

            send_col(s, x, vals)
        commit_cols(s)


def all_brightnesses(dev):
    """Increase the brightness with each pixel.
    Only 0-255 available, so it can't fill all 306 LEDs"""
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = [0 for _ in range(HEIGHT)]

            for y in range(HEIGHT):
                brightness = x + WIDTH * y
                if brightness > 255:
                    vals[y] = 0
                else:
                    vals[y] = brightness

            send_col(s, x, vals)
        commit_cols(s)


def countdown(dev, seconds):
    """ Run a countdown timer. Lighting more LEDs every 100th of a seconds.
    Until the timer runs out and every LED is lit"""
    start = datetime.now()
    target = seconds * 1_000_000
    global STOP_THREAD
    while True:
        if STOP_THREAD or dev.device in DISCONNECTED_DEVS:
            STOP_THREAD = False
            return
        now = datetime.now()
        passed_time = (now - start) / timedelta(microseconds=1)

        ratio = passed_time / target
        if passed_time >= target:
            break

        leds = int(306 * ratio)
        light_leds(dev, leds)

        time.sleep(0.01)

    light_leds(dev, 306)
    breathing(dev)
    #blinking(dev)


def blinking(dev):
    """Blink brightness high/off every second.
    Keeps currently displayed grid"""
    global STOP_THREAD
    while True:
        if STOP_THREAD or dev.device in DISCONNECTED_DEVS:
            STOP_THREAD = False
            return
        brightness(dev, 0)
        time.sleep(0.5)
        brightness(dev, 200)
        time.sleep(0.5)


def breathing(dev):
    """Animate breathing brightness.
    Keeps currently displayed grid"""
    # Bright ranges appear similar, so we have to go through those faster
    while True:
        # Go quickly from 250 to 50
        for i in range(10):
            time.sleep(0.03)
            brightness(dev, 250 - i*20)

        # Go slowly from 50 to 0
        for i in range(10):
            time.sleep(0.06)
            brightness(dev, 50 - i*5)

        # Go slowly from 0 to 50
        for i in range(10):
            time.sleep(0.06)
            brightness(dev, i*5)

        # Go quickly from 50 to 250
        for i in range(10):
            time.sleep(0.03)
            brightness(dev, 50 + i*20)


direction = None
body = []


def opposite_direction(direction):
    from getkey import keys
    if direction == keys.RIGHT:
        return keys.LEFT
    elif direction == keys.LEFT:
        return keys.RIGHT
    elif direction == keys.UP:
        return keys.DOWN
    elif direction == keys.DOWN:
        return keys.UP
    return direction


def snake_keyscan():
    from getkey import getkey, keys
    global direction
    global body

    while True:
        current_dir = direction
        key = getkey()
        if key in [keys.RIGHT, keys.UP, keys.LEFT, keys.DOWN]:
            # Don't allow accidental suicide if we have a body
            if key == opposite_direction(current_dir) and body:
                continue
            direction = key


def snake_embedded_keyscan():
    from getkey import getkey, keys

    while True:
        key_arg = None
        key = getkey()
        if key == keys.UP:
            key_arg = GameControlVal.Up
        elif key == keys.DOWN:
            key_arg = GameControlVal.Down
        elif key == keys.LEFT:
            key_arg = GameControlVal.Left
        elif key == keys.RIGHT:
            key_arg = GameControlVal.Right
        elif key == 'q':
            # Quit
            key_arg = GameControlVal.Quit
        if key_arg is not None:
            send_command(dev, CommandVals.GameControl, [key_arg])


def game_over(dev):
    global body
    while True:
        show_string(dev, 'GAME ')
        time.sleep(0.75)
        show_string(dev, 'OVER!')
        time.sleep(0.75)
        score = len(body)
        show_string(dev, f'{score:>3} P')
        time.sleep(0.75)


def pong_embedded():
    # Start game
    send_command(dev, CommandVals.StartGame, [Game.Pong])

    from getkey import getkey, keys

    while True:
        key_arg = None
        key = getkey()
        if key == keys.LEFT:
            key_arg = ARG_LEFT
        elif key == keys.RIGHT:
            key_arg = ARG_RIGHT
        elif key == 'a':
            key_arg = ARG_2LEFT
        elif key == 'd':
            key_arg = ARG_2RIGHT
        elif key == 'q':
            # Quit
            key_arg = ARG_QUIT
        if key_arg is not None:
            send_command(dev, CommandVals.GameControl, [key_arg])


def game_of_life_embedded(arg):
    # Start game
    # TODO: Add a way to stop it
    print("Game", int(arg))
    send_command(dev, CommandVals.StartGame, [Game.GameOfLife, int(arg)])


def snake_embedded():
    # Start game
    send_command(dev, CommandVals.StartGame, [Game.Snake])

    snake_embedded_keyscan()


def snake(dev):
    from getkey import keys
    global direction
    global body
    head = (0, 0)
    direction = keys.DOWN
    food = (0, 0)
    while food == head:
        food = (random.randint(0, WIDTH-1),
                random.randint(0, HEIGHT-1))

    # Setting
    WRAP = False

    thread = threading.Thread(target=snake_keyscan, args=(), daemon=True)
    thread.start()

    prev = datetime.now()
    while True:
        now = datetime.now()
        delta = (now - prev) / timedelta(milliseconds=1)

        if delta > 200:
            prev = now
        else:
            continue

        # Update position
        (x, y) = head
        oldhead = head
        if direction == keys.RIGHT:
            head = (x+1, y)
        elif direction == keys.LEFT:
            head = (x-1, y)
        elif direction == keys.UP:
            head = (x, y-1)
        elif direction == keys.DOWN:
            head = (x, y+1)

        # Detect edge condition
        (x, y) = head
        if head in body:
            return game_over(dev)
        elif x >= WIDTH or x < 0 or y >= HEIGHT or y < 0:
            if WRAP:
                if x >= WIDTH:
                    x = 0
                elif x < 0:
                    x = WIDTH-1
                elif y >= HEIGHT:
                    y = 0
                elif y < 0:
                    y = HEIGHT-1
                head = (x, y)
            else:
                return game_over(dev)
        elif head == food:
            body.insert(0, oldhead)
            while food == head:
                food = (random.randint(0, WIDTH-1),
                        random.randint(0, HEIGHT-1))
        elif body:
            body.pop()
            body.insert(0, oldhead)

        # Draw on screen
        matrix = [[0 for _ in range(HEIGHT)] for _ in range(WIDTH)]
        matrix[x][y] = 1
        matrix[food[0]][food[1]] = 1
        for bodypart in body:
            (x, y) = bodypart
            matrix[x][y] = 1
        render_matrix(dev, matrix)


def wpm_demo():
    """Capture keypresses and calculate the WPM of the last 10 seconds
    TODO: I'm not sure my calculation is right."""
    from getkey import getkey, keys
    start = datetime.now()
    keypresses = []
    while True:
        _ = getkey()

        now = datetime.now()
        keypresses = [x for x in keypresses if (now - x).total_seconds() < 10]
        keypresses.append(now)
        # Word is five letters
        wpm = (len(keypresses) / 5) * 6

        total_time = (now-start).total_seconds()
        if total_time < 10:
            wpm = wpm / (total_time / 10)

        show_string(dev, ' ' + str(int(wpm)))


def random_eq(dev):
    """Display an equlizer looking animation with random values.
    """
    global STOP_THREAD
    while True:
        if STOP_THREAD or dev.device in DISCONNECTED_DEVS:
            STOP_THREAD = False
            return
        # Lower values more likely, makes it look nicer
        weights = [i*i for i in range(33, 0, -1)]
        population = list(range(1, 34))
        vals = random.choices(population, weights=weights, k=9)
        eq(dev, vals)
        time.sleep(0.2)


def eq(dev, vals):
    """Display 9 values in equalizer diagram starting from the middle, going up and down"""
    matrix = [[0 for _ in range(34)] for _ in range(9)]

    for (col, val) in enumerate(vals[:9]):
        row = int(34 / 2)
        above = int(val / 2)
        below = val - above

        for i in range(above):
            matrix[col][row+i] = 0xFF
        for i in range(below):
            matrix[col][row-1-i] = 0xFF

    render_matrix(dev, matrix)


def render_matrix(dev, matrix):
    """Show a black/white matrix
    Send everything in a single command"""
    vals = [0x00 for _ in range(39)]

    for x in range(9):
        for y in range(34):
            i = x + 9*y
            if matrix[x][y]:
                vals[int(i/8)] = vals[int(i/8)] | (1 << i % 8)

    send_command(dev, CommandVals.Draw, vals)


def light_leds(dev, leds):
    """ Light a specific number of LEDs """
    vals = [0x00 for _ in range(39)]
    for byte in range(int(leds / 8)):
        vals[byte] = 0xFF
    for i in range(leds % 8):
        vals[int(leds / 8)] += 1 << i
    send_command(dev, CommandVals.Draw, vals)


def pwm_freq(dev, freq):
    """Display a pattern that's already programmed into the firmware"""
    if freq == '29kHz':
        send_command(dev, CommandVals.PwmFreq, [0])
    elif freq == '3.6kHz':
        send_command(dev, CommandVals.PwmFreq, [1])
    elif freq == '1.8kHz':
        send_command(dev, CommandVals.PwmFreq, [2])
    elif freq == '900Hz':
        send_command(dev, CommandVals.PwmFreq, [3])


def pattern(dev, p):
    """Display a pattern that's already programmed into the firmware"""
    if p == 'All LEDs on':
        send_command(dev, CommandVals.Pattern, [PatternVals.FullBrightness])
    elif p == 'Gradient (0-13% Brightness)':
        send_command(dev, CommandVals.Pattern, [PatternVals.Gradient])
    elif p == 'Double Gradient (0-7-0% Brightness)':
        send_command(dev, CommandVals.Pattern, [PatternVals.DoubleGradient])
    elif p == '"LOTUS" sideways':
        send_command(dev, CommandVals.Pattern, [PatternVals.DisplayLotus])
    elif p == 'Zigzag':
        send_command(dev, CommandVals.Pattern, [PatternVals.ZigZag])
    elif p == '"PANIC"':
        send_command(dev, CommandVals.Pattern, [PatternVals.DisplayPanic])
    elif p == '"LOTUS" Top Down':
        send_command(dev, CommandVals.Pattern, [PatternVals.DisplayLotus2])
    elif p == 'All brightness levels (1 LED each)':
        all_brightnesses(dev)
    elif p == 'Every Second Row':
        every_nth_row(dev, 2)
    elif p == 'Every Third Row':
        every_nth_row(dev, 3)
    elif p == 'Every Fourth Row':
        every_nth_row(dev, 4)
    elif p == 'Every Fifth Row':
        every_nth_row(dev, 5)
    elif p == 'Every Sixth Row':
        every_nth_row(dev, 6)
    elif p == 'Every Second Col':
        every_nth_col(dev, 2)
    elif p == 'Every Third Col':
        every_nth_col(dev, 3)
    elif p == 'Every Fourth Col':
        every_nth_col(dev, 4)
    elif p == 'Every Fifth Col':
        every_nth_col(dev, 4)
    elif p == 'Checkerboard':
        checkerboard(dev, 1)
    elif p == 'Double Checkerboard':
        checkerboard(dev, 2)
    elif p == 'Triple Checkerboard':
        checkerboard(dev, 3)
    elif p == 'Quad Checkerboard':
        checkerboard(dev, 4)
    else:
        print("Invalid pattern")


def show_string(dev, s):
    """Render a string with up to five letters"""
    show_font(dev, [convert_font(letter) for letter in str(s)[:5]])


def show_font(dev, font_items):
    """Render up to five 5x6 pixel font items"""
    vals = [0x00 for _ in range(39)]

    for digit_i, digit_pixels in enumerate(font_items):
        offset = digit_i * 7
        for pixel_x in range(5):
            for pixel_y in range(6):
                pixel_value = digit_pixels[pixel_x + pixel_y*5]
                i = (2+pixel_x) + (9*(pixel_y+offset))
                if pixel_value:
                    vals[int(i/8)] = vals[int(i/8)] | (1 << i % 8)

    send_command(dev, CommandVals.Draw, vals)


def show_symbols(dev, symbols):
    """Render a list of up to five symbols
    Can use letters/numbers or symbol names, like 'sun', ':)'"""
    font_items = []
    for symbol in symbols:
        s = convert_symbol(symbol)
        if not s:
            s = convert_font(symbol)
        font_items.append(s)

    show_font(dev, font_items)


def clock(dev):
    """Render the current time and display.
    Loops forever, updating every second"""
    global STOP_THREAD
    while True:
        if STOP_THREAD or dev.device in DISCONNECTED_DEVS:
            STOP_THREAD = False
            return
        now = datetime.now()
        current_time = now.strftime("%H:%M")
        print("Current Time =", current_time)

        show_string(dev, current_time)
        time.sleep(1)


def send_command(dev, command, parameters=[], with_response=False):
    return send_command_raw(dev, FWK_MAGIC + [command] + parameters, with_response)


def send_command_raw(dev, command, with_response=False):
    """Send a command to the device.
    Opens new serial connection every time"""
    # print(f"Sending command: {command}")
    try:
        with serial.Serial(dev.device, 115200) as s:
            s.write(command)

            if with_response:
                res = s.read(RESPONSE_SIZE)
                # print(f"Received: {res}")
                return res
    except (IOError, OSError) as ex:
        global DISCONNECTED_DEVS
        DISCONNECTED_DEVS.append(dev.device)
        #print("Error: ", ex)


def send_serial(s, command):
    """Send serial command by using existing serial connection"""
    try:
        s.write(command)
    except (IOError, OSError) as ex:
        global DISCONNECTED_DEVS
        DISCONNECTED_DEVS.append(dev.device)
        #print("Error: ", ex)


def popup(gui, message):
    if not gui:
        return
    import PySimpleGUI as sg
    sg.Popup(message, title="Framework Laptop 16 LED Matrix")


def gui(devices):
    import PySimpleGUI as sg

    device_checkboxes = []
    for dev in devices:
        version = get_version(dev)
        device_info = f"{dev.name}\nSerial No: {dev.serial_number}\nFW Version:{version}"
        checkbox = sg.Checkbox(device_info, default=True, key=f'-CHECKBOX-{dev.name}-', enable_events=True)
        device_checkboxes.append([checkbox])


    layout = [
        [sg.Text("Detected Devices")],
    ] + device_checkboxes + [
        [sg.HorizontalSeparator()],
        [sg.Text("Device Control")],
        [sg.Button("Bootloader"), sg.Button("Sleep"), sg.Button("Wake")],

        [sg.HorizontalSeparator()],
        [sg.Text("Brightness")],
        # TODO: Get default from device
        [sg.Slider((0, 255), orientation='h', default_value=120,
                   k='-BRIGHTNESS-', enable_events=True)],

        [sg.HorizontalSeparator()],
        [sg.Text("Animation")],
        [sg.Button("Start Animation"), sg.Button("Stop Animation")],

        [sg.HorizontalSeparator()],
        [sg.Text("Pattern")],
        [sg.Combo(PATTERNS, k='-PATTERN-', enable_events=True)],

        [sg.HorizontalSeparator()],
        [sg.Text("Fill screen X% (could be volume indicator)")],
        [sg.Slider((0, 100), orientation='h',
                   k='-PERCENTAGE-', enable_events=True)],

        [sg.HorizontalSeparator()],
        [sg.Text("Countdown Timer")],
        [
            sg.Spin([i for i in range(1, 60)],
                    initial_value=10, k='-COUNTDOWN-'),
            sg.Text("Seconds"),
            sg.Button("Start", k='-START-COUNTDOWN-'),
            sg.Button("Stop", k='-STOP-COUNTDOWN-'),
        ],

        [sg.HorizontalSeparator()],
        [
            sg.Column([
            [sg.Text("Black&White Image")],
            [sg.Button("Send stripe.gif", k='-SEND-BL-IMAGE-')]
            ]),
            sg.VSeperator(),
            sg.Column([
            [sg.Text("Greyscale Image")],
            [sg.Button("Send greyscale.gif", k='-SEND-GREY-IMAGE-')]
            ])
        ],

        [sg.HorizontalSeparator()],
        [sg.Text("Display Current Time")],
        [
            sg.Button("Start", k='-START-TIME-'),
            sg.Button("Stop", k='-STOP-TIME-')
        ],

        [sg.HorizontalSeparator()],
        [
            sg.Column([
                [sg.Text("Custom Text")],
                [sg.Input(k='-CUSTOM-TEXT-', s=7), sg.Button("Show", k='SEND-CUSTOM-TEXT')],
            ]),
            sg.VSeperator(),
            sg.Column([
                [sg.Text("Display Text with Symbols")],
                [sg.Button("Send '2 5 degC thunder'", k='-SEND-TEXT-')],
            ])
        ],
        [sg.HorizontalSeparator()],
        [sg.Text("PWM Frequency")],
        [sg.Combo(PWM_FREQUENCIES, k='-PWM-FREQ-', enable_events=True)],


        # TODO
        # [sg.Text("Play Snake")],
        # [sg.Button("Start Game", k='-PLAY-SNAKE-')],

        [sg.HorizontalSeparator()],
        [sg.Text("Equalizer")],
        [
            sg.Button("Start random equalizer", k='-RANDOM-EQ-'),
            sg.Button("Stop", k='-STOP-EQ-')
        ],
        # [sg.Button("Panic")]
    ]

    window = sg.Window("LED Matrix Control", layout, finalize=True)
    selected_devices = []
    global STOP_THREAD
    global DISCONNECTED_DEVS

    update_brightness_slider(window, devices)

    try:
        while True:
            event, values = window.read()
            # print('Event', event)
            # print('Values', values)

            # TODO
            for dev in devices:
                #print("Dev {} disconnected? {}".format(dev.name, dev.device in DISCONNECTED_DEVS))
                if dev.device in DISCONNECTED_DEVS:
                    window['-CHECKBOX-{}-'.format(dev.name)].update(False, disabled=True)

            selected_devices = [
                dev for dev in devices if
                values and values['-CHECKBOX-{}-'.format(dev.name)]
            ]
            # print("Selected {} devices".format(len(selected_devices)))

            if event == sg.WIN_CLOSED:
                break
            if len(selected_devices) == 1:
                dev = selected_devices[0]
                if event == '-START-COUNTDOWN-':
                    thread = threading.Thread(target=countdown, args=(dev,
                        int(values['-COUNTDOWN-']),), daemon=True)
                    thread.start()

                if event == '-START-TIME-':
                    thread = threading.Thread(target=clock, args=(dev,), daemon=True)
                    thread.start()

                if event == '-PLAY-SNAKE-':
                    snake()

                if event == '-RANDOM-EQ-':
                    thread = threading.Thread(target=random_eq, args=(dev,), daemon=True)
                    thread.start()
            else:
                if event in ['-START-COUNTDOWN-', '-PLAY-SNAKE-', '-RANDOM-EQ-', '-START-TIME-']:
                    sg.Popup('Select exactly 1 device for this action')
            if event in ['-STOP-COUNTDOWN-', '-STOP-EQ-', '-STOP-TIME-']:
                STOP_THREAD = True

            for dev in selected_devices:
                if event == "Bootloader":
                    bootloader(dev)

                if event == '-PATTERN-':
                    pattern(dev, values['-PATTERN-'])

                if event == '-PWM-FREQ-':
                    pwm_freq(dev, values['-PWM-FREQ-'])

                if event == 'Start Animation':
                    animate(dev, True)

                if event == 'Stop Animation':
                    animate(dev, False)

                if event == '-BRIGHTNESS-':
                    brightness(dev, int(values['-BRIGHTNESS-']))

                if event == '-PERCENTAGE-':
                    percentage(dev, int(values['-PERCENTAGE-']))

                if event == '-SEND-BL-IMAGE-':
                    path = os.path.join(resource_path(), 'res', 'stripe.gif')
                    image_bl(dev, path)

                if event == '-SEND-GREY-IMAGE-':
                    path = os.path.join(resource_path(), 'res', 'greyscale.gif')
                    image_greyscale(dev, path)

                if event == '-SEND-TEXT-':
                    show_symbols(dev, ['2', '5', 'degC', ' ', 'thunder'])

                if event == 'SEND-CUSTOM-TEXT':
                    show_string(dev, values['-CUSTOM-TEXT-'].upper())

                if event == 'Sleep':
                    send_command(dev, CommandVals.Sleep, [True])

                if event == 'Wake':
                    send_command(dev, CommandVals.Sleep, [False])

        window.close()
    except Exception as e:
        print(e)
        raise e
        pass
        #sg.popup_error_with_traceback(f'An error happened.  Here is the info:', e)


def display_string(disp_str):
    b = [ord(x) for x in disp_str]
    send_command(CommandVals.SetText, [len(disp_str)] + b)


def display_on_cmd(on):
    send_command(CommandVals.DisplayOn, [on])


def invert_screen_cmd(invert):
    send_command(CommandVals.InvertScreen, [invert])


def screen_saver_cmd(on):
    send_command(CommandVals.ScreenSaver, [on])


def set_fps_cmd(mode):
    res = send_command(CommandVals.SetFps, with_response=True)
    current_fps = res[0]

    if mode == 'quarter':
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b000
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('low')
    elif mode == 'half':
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b001
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('low')
    elif mode == 'one':
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b010
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('low')
    elif mode == 'two':
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b011
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('low')
    elif mode == 'four':
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b100
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('low')
    elif mode == 'eight':
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b101
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('low')
    elif mode == 'sixteen':
        fps = current_fps & ~HIGH_FPS_MASK
        fps |= 0b00000000
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('high')
    elif mode == 'thirtytwo':
        fps = current_fps & ~HIGH_FPS_MASK
        fps |= 0b00010000
        send_command(CommandVals.SetFps, [fps])
        set_power_mode_cmd('high')


def set_power_mode_cmd(mode):
    if mode == 'low':
        send_command(CommandVals.SetPowerMode, [0])
    elif mode == 'high':
        send_command(CommandVals.SetPowerMode, [1])
    else:
        print("Unsupported power mode")
        sys.exit(1)


def get_power_mode_cmd():
    res = send_command(CommandVals.SetPowerMode, with_response=True)
    current_mode = int(res[0])
    if current_mode == 0:
        print(f"Current Power Mode: Low Power")
    elif current_mode == 1:
        print(f"Current Power Mode: High Power")


def get_fps_cmd():
    res = send_command(CommandVals.SetFps, with_response=True)
    current_fps = res[0]
    res = send_command(CommandVals.SetPowerMode, with_response=True)
    current_mode = int(res[0])

    if current_mode == 0:
        current_fps &= LOW_FPS_MASK
        if current_fps == 0:
            fps = 0.25
        elif current_fps == 1:
            fps = 0.5
        else:
            fps = 2 ** (current_fps - 2)
    elif current_mode == 1:
        if current_fps & HIGH_FPS_MASK:
            fps = 32
        else:
            fps = 16

    print(f"Current FPS: {fps}")


# 5x6 symbol font. Leaves 2 pixels on each side empty
# We can leave one row empty below and then the display fits 5 of these digits.


def convert_symbol(symbol):
    symbols = {
        'degC': [
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 1, 1,
        ],
        'degF': [
            1, 1, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 0, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 1, 1,
            0, 0, 1, 0, 0,
        ],
        'snow': [
            0, 0, 0, 0, 0,
            1, 0, 1, 0, 1,
            0, 1, 1, 1, 0,
            1, 1, 1, 1, 1,
            0, 1, 1, 1, 0,
            1, 0, 1, 0, 1,
        ],
        'sun': [
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 1, 1, 0,
        ],
        'cloud': [
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        'rain': [
            0, 1, 1, 1, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 0, 0, 1,
            0, 0, 1, 0, 0,
            1, 0, 0, 1, 0,
        ],
        'thunder': [
            0, 1, 1, 1, 0,
            1, 1, 1, 1, 1,
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
        ],
        'batteryLow': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 0,
            1, 0, 0, 1, 1,
            1, 0, 0, 1, 1,
            1, 1, 1, 1, 0,
        ],
        '!!': [
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
        ],
        'heart': [
            0, 0, 0, 0, 0,
            1, 1, 0, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 1, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
        ],
        'heart0': [
            1, 1, 0, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 1, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        'heart2': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 1, 0, 1, 1,
            1, 1, 1, 1, 1,
            0, 1, 1, 1, 0,
            0, 0, 1, 0, 0,
        ],
        ':)': [
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],
        ':|': [
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
        ],
        ':(': [
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
        ],
        ';)': [
            0, 0, 0, 0, 0,
            1, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],
    }
    if symbol in symbols:
        return symbols[symbol]
    else:
        return None


def convert_font(num):
    """ 5x6 font. Leaves 2 pixels on each side empty
    We can leave one row empty below and then the display fits 5 of these digits."""
    font = {
        '0': [
            0, 1, 1, 0, 0,
            1, 0, 0, 1, 0,
            1, 0, 0, 1, 0,
            1, 0, 0, 1, 0,
            1, 0, 0, 1, 0,
            0, 1, 1, 0, 0,
        ],

        '1': [
            0, 0, 1, 0, 0,
            0, 1, 1, 0, 0,
            1, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
        ],

        '2': [
            1, 1, 1, 1, 0,
            0, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
        ],

        '3': [
            1, 1, 1, 1, 0,
            0, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],

        '4': [
            0, 0, 0, 1, 0,
            0, 0, 1, 1, 0,
            0, 1, 0, 1, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 0,
        ],

        '5': [
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],

        '6': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],

        '7': [
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ],

        '8': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],

        '9': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],

        ':': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
        ],

        ' ': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],

        '?': [
            0, 1, 1, 0, 0,
            0, 0, 0, 1, 0,
            0, 0, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
        ],

        '.': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],

        ',': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],

        '!': [
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 1, 0, 0,
        ],

        '/': [
            0, 0, 0, 0, 1,
            0, 0, 0, 1, 1,
            0, 0, 1, 1, 0,
            0, 1, 1, 0, 0,
            1, 1, 0, 0, 0,
            1, 0, 0, 0, 0,
        ],

        '*': [
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 1, 0, 1, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],

        '%': [
            1, 1, 0, 0, 1,
            1, 1, 0, 1, 1,
            0, 0, 1, 1, 0,
            0, 1, 1, 0, 0,
            1, 1, 0, 1, 1,
            1, 0, 0, 1, 1,
        ],

        '+': [
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 0, 0,
        ],

        '-': [
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],

        '=': [
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            0, 0, 0, 0, 0,
            0, 0, 0, 0, 0,
        ],
        'A': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
        ],
        'B': [
            1, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],
        'C': [
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
        ],
        'D': [
            1, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],
        'E': [
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
        ],
        'F': [
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
        ],
        'G': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 0,
            1, 0, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],
        'H': [
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
        ],
        'I': [
            0, 1, 1, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 1, 1, 1, 0,
        ],
        'J': [
            0, 1, 1, 1, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            0, 1, 0, 0, 1,
            0, 0, 1, 1, 0,
        ],
        'K': [
            1, 0, 0, 1, 0,
            1, 0, 1, 0, 0,
            1, 1, 0, 0, 0,
            1, 0, 1, 0, 0,
            1, 0, 0, 1, 0,
            1, 0, 0, 0, 1,
        ],
        'L': [
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
        ],
        'M': [
            0, 0, 0, 0, 0,
            0, 1, 0, 1, 0,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
        ],
        'N': [
            1, 0, 0, 0, 1,
            1, 1, 0, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 0, 1, 1,
        ],
        'O': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            0, 1, 1, 1, 0,
        ],
        'P': [
            1, 1, 1, 0, 0,
            1, 0, 0, 1, 0,
            1, 0, 0, 1, 0,
            1, 1, 1, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
        ],
        'Q': [
            0, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 0, 1, 0,
            0, 1, 1, 0, 1,
        ],
        'R': [
            1, 1, 1, 1, 0,
            1, 0, 0, 1, 0,
            1, 1, 1, 1, 0,
            1, 1, 0, 0, 0,
            1, 0, 1, 0, 0,
            1, 0, 0, 1, 0,
        ],
        'S': [
            1, 1, 1, 1, 1,
            1, 0, 0, 0, 0,
            0, 1, 1, 1, 0,
            0, 0, 0, 0, 1,
            0, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],
        'T': [
            1, 1, 1, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ],
        'U': [
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 1,
        ],
        'V': [
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            0, 1, 0, 1, 1,
            0, 1, 0, 1, 1,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ],
        'W': [
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 1, 0, 1,
            1, 0, 1, 0, 1,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
        ],
        'Y': [
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            0, 1, 0, 1, 0,
            0, 1, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 0, 1, 0, 0,
        ],
        'Z': [
            1, 1, 1, 1, 1,
            0, 0, 0, 1, 0,
            0, 0, 1, 0, 0,
            0, 1, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 1, 1,
        ],
    }
    if num in font:
        return font[num]
    else:
        return font['?']


if __name__ == "__main__":
    main()
