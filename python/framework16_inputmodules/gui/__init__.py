import os
import threading
import sys

import PySimpleGUI as sg

from inputmodule import (
    send_command,
    get_version,
    brightness,
    get_brightness,
    bootloader,
    CommandVals,
)
from gui.games import snake
from gui.ledmatrix import countdown, random_eq, clock
from gui.gui_threading import stop_thread, is_dev_disconnected
from inputmodule.ledmatrix import (
    percentage,
    pattern,
    animate,
    PATTERNS,
    PWM_FREQUENCIES,
    show_symbols,
    show_string,
    pwm_freq,
    image_bl,
    image_greyscale,
)


def update_brightness_slider(window, devices):
    average_brightness = None
    for dev in devices:
        if not average_brightness:
            average_brightness = 0

        br = get_brightness(dev)
        average_brightness += br
        print(f"Brightness: {br}")
    if average_brightness:
        window["-BRIGHTNESS-"].update(average_brightness / len(devices))


def popup(has_gui, message):
    if not has_gui:
        return
    import PySimpleGUI as sg

    sg.Popup(message, title="Framework Laptop 16 LED Matrix")


def run_gui(devices):
    device_checkboxes = []
    for dev in devices:
        version = get_version(dev)
        device_info = (
            f"{dev.name}\nSerial No: {dev.serial_number}\nFW Version:{version}"
        )
        checkbox = sg.Checkbox(
            device_info, default=True, key=f"-CHECKBOX-{dev.name}-", enable_events=True
        )
        device_checkboxes.append([checkbox])

    layout = (
        [
            [sg.Text("Detected Devices")],
        ]
        + device_checkboxes
        + [
            [sg.HorizontalSeparator()],
            [sg.Text("Device Control")],
            [sg.Button("Bootloader"), sg.Button("Sleep"), sg.Button("Wake")],
            [sg.HorizontalSeparator()],
            [sg.Text("Brightness")],
            # TODO: Get default from device
            [
                sg.Slider(
                    (0, 255),
                    orientation="h",
                    default_value=120,
                    k="-BRIGHTNESS-",
                    enable_events=True,
                )
            ],
            [sg.HorizontalSeparator()],
            [sg.Text("Animation")],
            [sg.Button("Start Animation"), sg.Button("Stop Animation")],
            [sg.HorizontalSeparator()],
            [sg.Text("Pattern")],
            [sg.Combo(PATTERNS, k="-PATTERN-", enable_events=True)],
            [sg.HorizontalSeparator()],
            [sg.Text("Fill screen X% (could be volume indicator)")],
            [
                sg.Slider(
                    (0, 100), orientation="h", k="-PERCENTAGE-", enable_events=True
                )
            ],
            [sg.HorizontalSeparator()],
            [sg.Text("Countdown Timer")],
            [
                sg.Spin([i for i in range(1, 60)], initial_value=10, k="-COUNTDOWN-"),
                sg.Text("Seconds"),
                sg.Button("Start", k="-START-COUNTDOWN-"),
                sg.Button("Stop", k="-STOP-COUNTDOWN-"),
            ],
            [sg.HorizontalSeparator()],
            [
                sg.Column(
                    [
                        [sg.Text("Black&White Image")],
                        [sg.Button("Send stripe.gif", k="-SEND-BL-IMAGE-")],
                    ]
                ),
                sg.VSeperator(),
                sg.Column(
                    [
                        [sg.Text("Greyscale Image")],
                        [sg.Button("Send greyscale.gif", k="-SEND-GREY-IMAGE-")],
                    ]
                ),
            ],
            [sg.HorizontalSeparator()],
            [sg.Text("Display Current Time")],
            [sg.Button("Start", k="-START-TIME-"), sg.Button("Stop", k="-STOP-TIME-")],
            [sg.HorizontalSeparator()],
            [
                sg.Column(
                    [
                        [sg.Text("Custom Text")],
                        [
                            sg.Input(k="-CUSTOM-TEXT-", s=7),
                            sg.Button("Show", k="SEND-CUSTOM-TEXT"),
                        ],
                    ]
                ),
                sg.VSeperator(),
                sg.Column(
                    [
                        [sg.Text("Display Text with Symbols")],
                        [sg.Button("Send '2 5 degC thunder'", k="-SEND-TEXT-")],
                    ]
                ),
            ],
            [sg.HorizontalSeparator()],
            [sg.Text("PWM Frequency")],
            [sg.Combo(PWM_FREQUENCIES, k="-PWM-FREQ-", enable_events=True)],
            # TODO
            # [sg.Text("Play Snake")],
            # [sg.Button("Start Game", k='-PLAY-SNAKE-')],
            [sg.HorizontalSeparator()],
            [sg.Text("Equalizer")],
            [
                sg.Button("Start random equalizer", k="-RANDOM-EQ-"),
                sg.Button("Stop", k="-STOP-EQ-"),
            ],
            # [sg.Button("Panic")]
        ]
    )

    window = sg.Window("LED Matrix Control", layout, finalize=True)
    selected_devices = []

    update_brightness_slider(window, devices)

    try:
        while True:
            event, values = window.read()
            # print('Event', event)
            # print('Values', values)

            # TODO
            for dev in devices:
                # print("Dev {} disconnected? {}".format(dev.name, dev.device in DISCONNECTED_DEVS))
                if is_dev_disconnected(dev.device):
                    window["-CHECKBOX-{}-".format(dev.name)].update(
                        False, disabled=True
                    )

            selected_devices = [
                dev
                for dev in devices
                if values and values["-CHECKBOX-{}-".format(dev.name)]
            ]
            # print("Selected {} devices".format(len(selected_devices)))

            if event == sg.WIN_CLOSED:
                break
            if len(selected_devices) == 1:
                dev = selected_devices[0]
                if event == "-START-COUNTDOWN-":
                    print("Starting countdown")
                    thread = threading.Thread(
                        target=countdown,
                        args=(
                            dev,
                            int(values["-COUNTDOWN-"]),
                        ),
                        daemon=True,
                    )
                    thread.start()

                if event == "-START-TIME-":
                    thread = threading.Thread(target=clock, args=(dev,), daemon=True)
                    thread.start()

                if event == "-PLAY-SNAKE-":
                    snake()

                if event == "-RANDOM-EQ-":
                    thread = threading.Thread(
                        target=random_eq, args=(dev,), daemon=True
                    )
                    thread.start()
            else:
                if event in [
                    "-START-COUNTDOWN-",
                    "-PLAY-SNAKE-",
                    "-RANDOM-EQ-",
                    "-START-TIME-",
                ]:
                    sg.Popup("Select exactly 1 device for this action")
            if event in ["-STOP-COUNTDOWN-", "-STOP-EQ-", "-STOP-TIME-"]:
                stop_thread()

            for dev in selected_devices:
                if event == "Bootloader":
                    bootloader(dev)

                if event == "-PATTERN-":
                    pattern(dev, values["-PATTERN-"])

                if event == "-PWM-FREQ-":
                    pwm_freq(dev, values["-PWM-FREQ-"])

                if event == "Start Animation":
                    animate(dev, True)

                if event == "Stop Animation":
                    animate(dev, False)

                if event == "-BRIGHTNESS-":
                    brightness(dev, int(values["-BRIGHTNESS-"]))

                if event == "-PERCENTAGE-":
                    percentage(dev, int(values["-PERCENTAGE-"]))

                if event == "-SEND-BL-IMAGE-":
                    path = os.path.join(resource_path(), "res", "stripe.gif")
                    image_bl(dev, path)

                if event == "-SEND-GREY-IMAGE-":
                    path = os.path.join(resource_path(), "res", "greyscale.gif")
                    image_greyscale(dev, path)

                if event == "-SEND-TEXT-":
                    show_symbols(dev, ["2", "5", "degC", " ", "thunder"])

                if event == "SEND-CUSTOM-TEXT":
                    show_string(dev, values["-CUSTOM-TEXT-"].upper())

                if event == "Sleep":
                    send_command(dev, CommandVals.Sleep, [True])

                if event == "Wake":
                    send_command(dev, CommandVals.Sleep, [False])

        window.close()
    except Exception as e:
        print(e)
        raise e
        pass
        # sg.popup_error_with_traceback(f'An error happened.  Here is the info:', e)


def resource_path():
    """Get absolute path to resource, works for dev and for PyInstaller"""
    try:
        # PyInstaller creates a temp folder and stores path in _MEIPASS
        base_path = sys._MEIPASS
    except Exception:
        base_path = os.path.abspath("../../")

    return base_path
