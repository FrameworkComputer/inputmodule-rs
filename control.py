#!/usr/bin/env python3
import argparse
import sys
import serial

FWK_MAGIC = [0x32, 0xAC]
PATTERNS = ['full', 'lotus', 'gradient', 'double-gradient', 'zigzag', 'panic', 'lotus2']

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--bootloader", help="Bootloader",
                    action="store_true")
    #parser.add_argument('--sleep', action=argparse.BooleanOptionalAction)
    parser.add_argument("--brightness", help="Brightness",
                    type=int)
    parser.add_argument('--animate', action=argparse.BooleanOptionalAction)
    parser.add_argument("--pattern", help="Pattern",
                    type=str, choices=PATTERNS)
    parser.add_argument("--percentage", help="Percentage",
                    type=int)
    parser.add_argument("--panic", help="Panic",
                    action="store_true")
    parser.add_argument("--gui", help="GUI",
                    action="store_true")
    args = parser.parse_args()

    if args.bootloader:
        bootloader()
    #elif args.sleep is not None:
    #    print(f"sleep: {args.sleep}")
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
        #send_command(command)
    elif args.gui:
        gui()
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
        [sg.Slider((0,255), orientation='h', k='-BRIGHTNESS-', enable_events=True)],

        [sg.Text("Animation")],
        [sg.Button("Start Animation"), sg.Button("Stop Animation")],

        [sg.Text("Pattern")],
        [sg.Combo(PATTERNS, k='-COMBO-', enable_events=True)],

        [sg.Text("Display Percentage")],
        [sg.Slider((0,100), orientation='h', k='-PERCENTAGE-', enable_events=True)],

        #[sg.Text("Sleep")],
        #[sg.Button("Sleep"), sg.Button("Wake")]
        #[sg.Button("Panic")]

        [sg.Button("Quit")]
    ]
    window = sg.Window("Lotus LED Matrix Control", layout)
    while True:
        event, values = window.read()
        print('Event', event)
        print('Values', values)

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

    window.close()

if __name__ == "__main__":
    main()
