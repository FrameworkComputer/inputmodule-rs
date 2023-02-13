#!/usr/bin/env python3
import argparse
import sys
import time
from datetime import datetime

# Need to install
import serial

# Optional dependencies:
# from PIL import Image
# import PySimpleGUI as sg

FWK_MAGIC = [0x32, 0xAC]
PATTERNS = ['full', 'lotus', 'gradient',
            'double-gradient', 'zigzag', 'panic', 'lotus2']
DRAW_PATTERNS = ['off', 'on', 'foo']


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
    # parser.add_argument("--draw", help="Draw",
    #                    type=str, choices=DRAW_PATTERNS)
    parser.add_argument("--image", help="Display a PNG or GIF image (black and white only)",
                        type=argparse.FileType('rb'))
    parser.add_argument("--percentage", help="Fill a percentage of the screen",
                        type=int)
    parser.add_argument("--clock", help="Display the current time",
                        action="store_true")
    parser.add_argument("--gui", help="Launch the graphical version of the program",
                        action="store_true")
    parser.add_argument("--panic", help="Crash the firmware (TESTING ONLY)",
                        action="store_true")
    args = parser.parse_args()

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
    # elif args.draw is not None:
    #    draw(args.draw)
    elif args.image is not None:
        image(args.image)
    elif args.gui:
        gui()
    elif args.clock:
        clock()
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


def draw(p):
    if p == 'off':
        vals = [0x00 for _ in range(39)]
        command = FWK_MAGIC + [0x06] + vals
        send_command(command)
    elif p == 'on':
        vals = [0xFF for _ in range(39)]
        command = FWK_MAGIC + [0x06] + vals
        send_command(command)
    elif p == 'foo':
        vals = [0xFF for _ in range(39)]
        vals[1] = 0x00
        vals[2] = 0x00
        vals[3] = 0x00
        vals[8] = 0x00
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


def clock():
    while True:
        vals = [0x00 for _ in range(39)]

        now = datetime.now()
        current_time = now.strftime("%H:%M")
        print("Current Time =", current_time)
        for digit_i, digit in enumerate(current_time):
            offset = digit_i * 7
            digit_pixels = number(digit)
            for pixel_x in range(5):
                for pixel_y in range(6):
                    pixel_value = digit_pixels[pixel_x + pixel_y*5]
                    i = (2+pixel_x) + (9*(pixel_y+offset))
                    if pixel_value:
                        vals[int(i/8)] = vals[int(i/8)] | (1 << i % 8)

        command = FWK_MAGIC + [0x06] + vals
        send_command(command)
        time.sleep(1)


def send_command(command):
    print(f"Sending command: {command}")
    with serial.Serial('/dev/ttyACM0', 9600) as s:
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

        if event == 'Sleep':
            command = FWK_MAGIC + [0x03, True]
            send_command(command)

        if event == 'Wake':
            command = FWK_MAGIC + [0x03, False]
            send_command(command)

    window.close()


def number(num):
    numbers = {
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
        ]
    }
    return numbers[num]


if __name__ == "__main__":
    main()
