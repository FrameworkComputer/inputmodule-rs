#!/usr/bin/env python3
import argparse
import sys
import threading
import time
from datetime import datetime, timedelta
import random

# Need to install
import serial

# Optional dependencies:
# from PIL import Image
# import PySimpleGUI as sg

FWK_MAGIC = [0x32, 0xAC]
PATTERNS = ['full', 'lotus', 'gradient',
            'double-gradient', 'zigzag', 'panic', 'lotus2']
DRAW_PATTERNS = ['off', 'on', 'foo']

SERIAL_DEV = None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--bootloader", help="Jump to the bootloader to flash new firmware",
                        action="store_true")
    parser.add_argument('--sleep', help='Simulate the host going to sleep or waking up',
                        action=argparse.BooleanOptionalAction)
    parser.add_argument("--brightness", help="Adjust the brightness. Value 0-255",
                        type=int)
    parser.add_argument('--animate', action=argparse.BooleanOptionalAction,
                        help='Start/stop vertical scrolling')
    parser.add_argument("--pattern", help='Display a pattern',
                        type=str, choices=PATTERNS)
    parser.add_argument("--image", help="Display a PNG or GIF image (black and white only)",
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
    parser.add_argument("--wpm", help="WPM Demo", action="store_true")
    parser.add_argument(
        "--random-eq", help="Random Equalizer", action="store_true")
    parser.add_argument("--serial-dev", help="Change the serial dev. Probably /dev/ttyACM0 on Linux, COM0 on Windows",
                        default='/dev/ttyACM0')
    args = parser.parse_args()

    if args.serial_dev is not None:
        global SERIAL_DEV
        SERIAL_DEV = args.serial_dev

    if args.bootloader:
        bootloader()
    elif args.sleep is not None:
        command = FWK_MAGIC + [0x03, args.sleep]
        send_command(command)
    elif args.brightness is not None:
        if args.brightness > 255 or args.brightness < 0:
            print("Brightness must be 0-255")
            sys.exit(1)
        brightness(args.brightness)
    elif args.percentage is not None:
        if args.percentage > 100 or args.percentage < 0:
            print("Percentage must be 0-100")
            sys.exit(1)
        percentage(args.percentage)
    elif args.pattern is not None:
        pattern(args.pattern)
    elif args.animate is not None:
        animate(args.animate)
    elif args.panic:
        command = FWK_MAGIC + [0x05, 0x00]
        send_command(command)
    elif args.image is not None:
        image(args.image)
    elif args.gui:
        gui()
    elif args.blink:
        blinking()
    elif args.breathing:
        breathing()
    elif args.wpm:
        wpm_demo()
    elif args.eq is not None:
        eq(args.eq)
    elif args.random_eq:
        random_eq()
    elif args.clock:
        clock()
    elif args.string is not None:
        show_string(args.string)
    elif args.symbols is not None:
        show_symbols(args.symbols)
    else:
        print("Provide arg")


def bootloader():
    command = FWK_MAGIC + [0x02, 0x00]
    send_command(command)


def percentage(p):
    command = FWK_MAGIC + [0x01, 0x00, p]
    send_command(command)


def brightness(b):
    command = FWK_MAGIC + [0x00, b]
    send_command(command)


def animate(b):
    command = FWK_MAGIC + [0x04, b]
    send_command(command)


def image(image_file):
    from PIL import Image
    im = Image.open(image_file)
    width, height = im.size
    pixel_values = list(im.getdata())
    vals = [0 for _ in range(39)]
    for i, pixel in enumerate(pixel_values):
        # PNG has tuple, GIF has single value per pixel
        if pixel == (255, 255, 255) or pixel == 1:
            vals[int(i/8)] = vals[int(i/8)] | (1 << i % 8)
    command = FWK_MAGIC + [0x06] + vals
    send_command(command)


def countdown(seconds):
    """ Run a countdown timer. Lighting more LEDs every 100th of a seconds.
    Until the timer runs out and every LED is lit"""
    start = datetime.now()
    target = seconds * 1_000_000
    while True:
        now = datetime.now()
        passed_time = (now - start) / timedelta(microseconds=1)

        ratio = passed_time / target
        if passed_time >= target:
            break

        leds = int(306 * ratio)
        light_leds(leds)

        time.sleep(0.01)

    light_leds(306)
    # breathing()
    blinking()


def blinking():
    while True:
        brightness(0)
        time.sleep(0.5)
        brightness(200)
        time.sleep(0.5)


def breathing():
    # Bright ranges appear similar, so we have to go through those faster
    while True:
        # Go quickly from 250 to 50
        for i in range(10):
            time.sleep(0.03)
            brightness(250 - i*20)

        # Go slowly from 50 to 0
        for i in range(10):
            time.sleep(0.06)
            brightness(50 - i*5)

        # Go slowly from 0 to 50
        for i in range(10):
            time.sleep(0.06)
            brightness(i*5)

        # Go quickly from 50 to 250
        for i in range(10):
            time.sleep(0.03)
            brightness(50 + i*20)


def wpm_demo():
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

        show_string(' ' + str(int(wpm)))


def random_eq():
    while True:
        weights = [i*i for i in range(33, 0, -1)]
        population = list(range(1, 34))
        vals = random.choices(population, weights=weights, k=9)
        eq(vals)
        time.sleep(0.2)


def eq(vals):
    matrix = [[0 for _ in range(34)] for _ in range(9)]

    for (col, val) in enumerate(vals[:9]):
        row = int(34 / 2)
        above = int(val / 2)
        below = val - above

        for i in range(above):
            matrix[col][row+i] = 0xFF
        for i in range(below):
            matrix[col][row-1-i] = 0xFF

    render_matrix(matrix)


def render_matrix(matrix):
    vals = [0x00 for _ in range(39)]

    for x in range(9):
        for y in range(34):
            i = x + 9*y
            if matrix[x][y]:
                vals[int(i/8)] = vals[int(i/8)] | (1 << i % 8)

    command = FWK_MAGIC + [0x06] + vals
    send_command(command)


def light_leds(leds):
    """ Light a specific number of LEDs """
    vals = [0x00 for _ in range(39)]
    for byte in range(int(leds / 8)):
        vals[byte] = 0xFF
    for i in range(leds % 8):
        vals[int(leds / 8)] += 1 << i
    command = FWK_MAGIC + [0x06] + vals
    send_command(command)


def pattern(p):
    if p == 'full':
        command = FWK_MAGIC + [0x01, 5]
        send_command(command)
    elif p == 'gradient':
        command = FWK_MAGIC + [0x01, 1]
        send_command(command)
    elif p == 'double-gradient':
        command = FWK_MAGIC + [0x01, 2]
        send_command(command)
    elif p == 'lotus':
        command = FWK_MAGIC + [0x01, 3]
        send_command(command)
    elif p == 'zigzag':
        command = FWK_MAGIC + [0x01, 4]
        send_command(command)
    elif p == 'panic':
        command = FWK_MAGIC + [0x01, 6]
        send_command(command)
    elif p == 'lotus2':
        command = FWK_MAGIC + [0x01, 7]
        send_command(command)
    else:
        print("Invalid pattern")


def show_string(num):
    show_font([convert_font(letter) for letter in str(num)[:5]])


def show_font(font_items):
    vals = [0x00 for _ in range(39)]

    for digit_i, digit_pixels in enumerate(font_items):
        offset = digit_i * 7
        for pixel_x in range(5):
            for pixel_y in range(6):
                pixel_value = digit_pixels[pixel_x + pixel_y*5]
                i = (2+pixel_x) + (9*(pixel_y+offset))
                if pixel_value:
                    vals[int(i/8)] = vals[int(i/8)] | (1 << i % 8)

    command = FWK_MAGIC + [0x06] + vals
    send_command(command)


def show_symbols(symbols):
    font_items = []
    for symbol in symbols:
        s = convert_symbol(symbol)
        if not s:
            s = convert_font(symbol)
        font_items.append(s)

    show_font(font_items)


def clock():
    while True:
        now = datetime.now()
        current_time = now.strftime("%H:%M")
        print("Current Time =", current_time)

        show_string(current_time)
        time.sleep(1)


def send_command(command):
    print(f"Sending command: {command}")
    global SERIAL_DEV
    with serial.Serial(SERIAL_DEV, 9600) as s:
        s.write(command)


def gui():
    import PySimpleGUI as sg

    layout = [
        [sg.Text("Bootloader")],
        [sg.Button("Bootloader")],

        [sg.Text("Brightness")],
        [sg.Slider((0, 255), orientation='h',
                   k='-BRIGHTNESS-', enable_events=True)],

        [sg.Text("Animation")],
        [sg.Button("Start Animation"), sg.Button("Stop Animation")],

        [sg.Text("Pattern")],
        [sg.Combo(PATTERNS, k='-COMBO-', enable_events=True)],

        [sg.Text("Display Percentage")],
        [sg.Slider((0, 100), orientation='h',
                   k='-PERCENTAGE-', enable_events=True)],

        [sg.Text("Countdown")],
        [
            sg.Spin([i for i in range(1, 60)],
                    initial_value=10, k='-COUNTDOWN-'),
            sg.Text("Seconds"),
            sg.Button("Start", k='-START-COUNTDOWN-')
        ],

        [sg.Text("Sleep")],
        [sg.Button("Sleep"), sg.Button("Wake")],
        # [sg.Button("Panic")]

        [sg.Button("Quit")]
    ]
    window = sg.Window("Lotus LED Matrix Control", layout)
    while True:
        event, values = window.read()
        # print('Event', event)
        # print('Values', values)

        if event == "Quit" or event == sg.WIN_CLOSED:
            break

        if event == "Bootloader":
            bootloader()

        if event == '-COMBO-':
            pattern(values['-COMBO-'])

        if event == 'Start Animation':
            animate(True)

        if event == 'Stop Animation':
            animate(False)

        if event == '-BRIGHTNESS-':
            brightness(int(values['-BRIGHTNESS-']))

        if event == '-PERCENTAGE-':
            percentage(int(values['-PERCENTAGE-']))

        if event == '-START-COUNTDOWN-':
            thread = threading.Thread(target=countdown, args=(
                int(values['-COUNTDOWN-']),), daemon=True)
            thread.start()

        if event == 'Sleep':
            command = FWK_MAGIC + [0x03, True]
            send_command(command)

        if event == 'Wake':
            command = FWK_MAGIC + [0x03, False]
            send_command(command)

    window.close()

# 5x6 symbol font. Leaves 2 pixels on each side empty
# We can leave one row empty below and then the display fits 5 of these digits.


def convert_symbol(symbol):
    symbols = {
        'degC': [
            0, 0, 0, 1, 1,
            0, 0, 0, 1, 1,
            1, 1, 1, 0, 0,
            1, 0, 0, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 0, 0,
        ],
        'degF': [
            0, 0, 0, 1, 1,
            0, 0, 0, 1, 1,
            1, 1, 1, 0, 0,
            1, 0, 0, 0, 0,
            1, 1, 1, 0, 0,
            1, 0, 0, 0, 0,
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

# 5x6 font. Leaves 2 pixels on each side empty
# We can leave one row empty below and then the display fits 5 of these digits.


def convert_font(num):
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
        'D': [
            1, 1, 1, 1, 0,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 0, 0, 0, 1,
            1, 1, 1, 1, 0,
        ],
    }
    if num in font:
        return font[num]
    else:
        return font['?']


if __name__ == "__main__":
    main()
