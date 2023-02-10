#!/usr/bin/env python3
import argparse
import sys
import serial

FWK_MAGIC = [0x32, 0xAC]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--bootloader", help="Bootloader",
                    action="store_true")
    parser.add_argument("--sleep", help="Sleep",
                    type=bool)
    parser.add_argument("--brightness", help="Brightness",
                    type=int)
    parser.add_argument("--animate", help="Animate",
                    type=bool)
    parser.add_argument("--pattern", help="Pattern",
                    type=str, choices=['full', 'lotus', 'gradient', 'double-gradient', 'zigzag'])
    parser.add_argument("--percentage", help="Percentage",
                    type=int)
    args = parser.parse_args()

    if args.bootloader:
        print("bootloader")
        command = FWK_MAGIC + [0x02]
        send_command(command)
    elif args.sleep:
        pass
    elif args.brightness:
        if args.brightness > 255 or args.brightness < 0:
            print("Brightness must be 0-255")
            sys.exit(1)
        command = FWK_MAGIC + [0x00, args.brightness]
        send_command(command)
    elif args.percentage is not None:
        if args.percentage > 100 or args.percentage < 0:
            print("Percentage must be 0-100")
            sys.exit(1)
        command = FWK_MAGIC + [0x01, 0x00, args.percentage]
        send_command(command)
    elif 'pattern' in args:
        if args.pattern == 'full':
            command = FWK_MAGIC + [0x01, 5]
            send_command(command)
        elif args.pattern == 'gradient':
            command = FWK_MAGIC + [0x01, 1]
            send_command(command)
        elif args.pattern == 'double-gradient':
            command = FWK_MAGIC + [0x01, 2]
            send_command(command)
        elif args.pattern == 'lotus':
            command = FWK_MAGIC + [0x01, 3]
            send_command(command)
        elif args.pattern == 'zigzag':
            command = FWK_MAGIC + [0x01, 4]
            send_command(command)
    else:
        print("Provide arg")

def send_command(command):
    print(f"Sending command: {command}")
    with serial.Serial('/dev/ttyACM0', 9600) as s:
        s.write(command)

if __name__ == "__main__":
    main()
