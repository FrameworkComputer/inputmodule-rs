#!/usr/bin/env python3
import argparse
import sys

# Need to install
from serial.tools import list_ports

# Local dependencies
from inputmodule import gui
from inputmodule.inputmodule import (
    INPUTMODULE_PIDS,
    send_command,
    get_version,
    brightness,
    get_brightness,
    CommandVals,
    bootloader_jump,
    GameOfLifeStartParam,
    GameControlVal,
)
from inputmodule.games import (
    snake,
    snake_embedded,
    pong_embedded,
    game_of_life_embedded,
    wpm_demo,
)
from inputmodule.gui.ledmatrix import random_eq, clock, blinking
from inputmodule.inputmodule.ledmatrix import (
    eq,
    breathing,
    camera,
    video,
    all_brightnesses,
    percentage,
    pattern,
    animate,
    get_animate,
    pwm_freq,
    get_pwm_freq,
    show_string,
    show_symbols,
    PATTERNS,
    image_bl,
    image_greyscale,
)
from inputmodule.inputmodule.b1display import (
    b1image_bl,
    invert_screen_cmd,
    screen_saver_cmd,
    set_fps_cmd,
    set_power_mode_cmd,
    get_power_mode_cmd,
    get_fps_cmd,
    SCREEN_FPS,
    display_on_cmd,
    display_string,
)
from inputmodule.inputmodule.c1minimal import (
    set_color,
    get_color,
    RGB_COLORS,
)

# Optional dependencies:
# from PIL import Image
# import PySimpleGUI as sg


def main_cli():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-l", "--list", help="List all compatible devices", action="store_true"
    )
    parser.add_argument(
        "--bootloader",
        help="Jump to the bootloader to flash new firmware",
        action="store_true",
    )
    parser.add_argument(
        "--sleep",
        help="Simulate the host going to sleep or waking up",
        action=argparse.BooleanOptionalAction,
    )
    parser.add_argument(
        "--is-sleeping", help="Check current sleep state", action="store_true"
    )
    parser.add_argument(
        "--brightness", help="Adjust the brightness. Value 0-255", type=int
    )
    parser.add_argument(
        "--get-brightness", help="Get current brightness", action="store_true"
    )
    parser.add_argument(
        "--animate",
        action=argparse.BooleanOptionalAction,
        help="Start/stop vertical scrolling",
    )
    parser.add_argument(
        "--get-animate", action="store_true", help="Check if currently animating"
    )
    parser.add_argument(
        "--pwm",
        help="Adjust the PWM frequency. Value 0-255",
        type=int,
        choices=[29000, 3600, 1800, 900],
    )
    parser.add_argument(
        "--get-pwm", help="Get current PWM Frequency", action="store_true"
    )
    parser.add_argument(
        "--pattern", help="Display a pattern", type=str, choices=PATTERNS
    )
    parser.add_argument(
        "--image",
        help="Display a PNG or GIF image in black and white only)",
        type=argparse.FileType("rb"),
    )
    parser.add_argument(
        "--image-grey",
        help="Display a PNG or GIF image in greyscale",
        type=argparse.FileType("rb"),
    )
    parser.add_argument(
        "--camera", help="Stream from the webcam", action="store_true")
    parser.add_argument("--video", help="Play a video", type=str)
    parser.add_argument(
        "--percentage", help="Fill a percentage of the screen", type=int
    )
    parser.add_argument(
        "--clock", help="Display the current time", action="store_true")
    parser.add_argument(
        "--string", help="Display a string or number, like FPS", type=str
    )
    parser.add_argument(
        "--symbols", help="Show symbols (degF, degC, :), snow, cloud, ...)", nargs="+"
    )
    parser.add_argument(
        "--gui", help="Launch the graphical version of the program", action="store_true"
    )
    parser.add_argument(
        "--panic", help="Crash the firmware (TESTING ONLY)", action="store_true"
    )
    parser.add_argument(
        "--blink", help="Blink the current pattern", action="store_true"
    )
    parser.add_argument(
        "--breathing", help="Breathing of the current pattern", action="store_true"
    )
    parser.add_argument("--eq", help="Equalizer", nargs="+", type=int)
    parser.add_argument(
        "--random-eq", help="Random Equalizer", action="store_true")
    parser.add_argument("--wpm", help="WPM Demo", action="store_true")
    parser.add_argument("--snake", help="Snake", action="store_true")
    parser.add_argument(
        "--snake-embedded", help="Snake on the module", action="store_true"
    )
    parser.add_argument(
        "--pong-embedded", help="Pong on the module", action="store_true"
    )
    parser.add_argument(
        "--game-of-life-embedded",
        help="Game of Life",
        type=GameOfLifeStartParam.argparse,
        choices=list(GameOfLifeStartParam),
    )
    parser.add_argument(
        "--quit-embedded-game", help="Quit the current game", action="store_true"
    )
    parser.add_argument(
        "--all-brightnesses",
        help="Show every pixel in a different brightness",
        action="store_true",
    )
    parser.add_argument(
        "--set-color",
        help="Set RGB color (C1 Minimal Input Module)",
        choices=RGB_COLORS,
    )
    parser.add_argument(
        "--get-color",
        help="Get RGB color (C1 Minimal Input Module)",
        action="store_true",
    )
    parser.add_argument(
        "-v", "--version", help="Get device version", action="store_true"
    )
    parser.add_argument(
        "--serial-dev",
        help="Change the serial dev. Probably /dev/ttyACM0 on Linux, COM0 on Windows",
    )

    parser.add_argument(
        "--disp-str", help="Display a string on the LCD Display", type=str
    )
    parser.add_argument(
        "--display-on",
        help="Control display power",
        action=argparse.BooleanOptionalAction,
    )
    parser.add_argument(
        "--invert-screen", help="Invert display", action=argparse.BooleanOptionalAction
    )
    parser.add_argument(
        "--screen-saver",
        help="Turn on/off screensaver",
        action=argparse.BooleanOptionalAction,
    )
    parser.add_argument("--set-fps", help="Set screen FPS", choices=SCREEN_FPS)
    parser.add_argument(
        "--set-power-mode", help="Set screen power mode", choices=["high", "low"]
    )
    parser.add_argument("--get-fps", help="Set screen FPS",
                        action="store_true")
    parser.add_argument(
        "--get-power-mode", help="Set screen power mode", action="store_true"
    )
    parser.add_argument(
        "--b1image",
        help="On the B1 display, show a PNG or GIF image in black and white only)",
        type=argparse.FileType("rb"),
    )

    args = parser.parse_args()

    # Selected device
    dev = None
    ports = find_devs()

    if args.list:
        print_devs(ports)
        sys.exit(0)

    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        # Force GUI in pyinstaller bundled app
        args.gui = True

    if not ports:
        print("No device found")
        gui.popup("No device found", gui=args.gui)
        sys.exit(1)
    elif args.serial_dev is not None:
        filtered_devs = [
            port for port in ports if port.name in args.serial_dev]
        if not filtered_devs:
            print("Failed to find requested device")
            sys.exit(1)
        dev = filtered_devs[0]
    elif len(ports) == 1:
        dev = ports[0]
    elif len(ports) >= 1 and not args.gui:
        gui.popup(
            "More than 1 compatibles devices found. Please choose from the commandline with --serial-dev COMX.\nConnected ports:\n- {}".format(
                "\n- ".join([port.device for port in ports])
            ),
            gui=args.gui,
        )
        print(
            "More than 1 compatible device found. Please choose with --serial-dev ..."
        )
        print("Example on Windows: --serial-dev COM3")
        print("Example on Linux:   --serial-dev /dev/ttyACM0")
        print_devs(ports)
        sys.exit(1)
    elif args.gui:
        # TODO: Allow selection in GUI
        print("Select in GUI")

    if not args.gui and dev is None:
        print("No device selected")
        gui.popup("No device selected", gui=args.gui)
        sys.exit(1)

    if args.bootloader:
        bootloader_jump(dev)
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
            pwm_freq(dev, "29kHz")
        elif args.pwm == 3600:
            pwm_freq(dev, "3.6kHz")
        elif args.pwm == 1800:
            pwm_freq(dev, "1.8kHz")
        elif args.pwm == 900:
            pwm_freq(dev, "900Hz")
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
    elif args.camera:
        camera(dev)
    elif args.video is not None:
        video(dev, args.video)
    elif args.all_brightnesses:
        all_brightnesses(dev)
    elif args.set_color:
        set_color(dev, args.set_color)
    elif args.get_color:
        (red, green, blue) = get_color(dev)
        print(f"Current color: RGB:({red}, {green}, {blue})")
    elif args.gui:
        devices = find_devs()  # show=False, verbose=False)
        print("Found {} devices".format(len(devices)))
        gui.run_gui(devices)
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


def find_devs():
    ports = list_ports.comports()
    return [
        port for port in ports if port.vid == 0x32AC and port.pid in INPUTMODULE_PIDS
    ]


def print_devs(ports):
    for port in ports:
        print(f"{port.device}")
        print(f"  {port.name}")
        print(f"  VID:     0x{port.vid:04X}")
        print(f"  PID:     0x{port.pid:04X}")
        print(f"  SN:      {port.serial_number}")
        print(f"  Product: {port.product}")


def main_gui():
    devices = find_devs()  # show=False, verbose=False)
    print("Found {} devices".format(len(devices)))
    gui.run_gui(devices)


if __name__ == "__main__":
    main_cli()
