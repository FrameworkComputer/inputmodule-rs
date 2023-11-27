import sys

from inputmodule.inputmodule import send_command, CommandVals, FWK_MAGIC

B1_WIDTH = 300
B1_HEIGHT = 400
GREYSCALE_DEPTH = 32

SCREEN_FPS = ["quarter", "half", "one", "two",
              "four", "eight", "sixteen", "thirtytwo"]
HIGH_FPS_MASK = 0b00010000
LOW_FPS_MASK = 0b00000111


def b1image_bl(dev, image_file):
    """Display an image in black and white
    Confirmed working with PNG and GIF.
    Must be 300x400 in size.
    Sends one 400px column in a single commands and a flush at the end
    """

    from PIL import Image

    im = Image.open(image_file).convert("RGB")
    width, height = im.size
    assert width == B1_WIDTH
    assert height == B1_HEIGHT
    pixel_values = list(im.getdata())

    for x in range(B1_WIDTH):
        vals = [0 for _ in range(50)]

        byte = None
        for y in range(B1_HEIGHT):
            pixel = pixel_values[y * B1_WIDTH + x]
            brightness = sum(pixel) / 3
            black = brightness < 0xFF / 2

            bit = y % 8

            if bit == 0:
                byte = 0
            if black:
                byte |= 1 << bit

            if bit == 7:
                vals[int(y / 8)] = byte

        column_le = list((x).to_bytes(2, "little"))
        command = FWK_MAGIC + [0x16] + column_le + vals
        send_command(dev, command)

    # Flush
    command = FWK_MAGIC + [0x17]
    send_command(dev, command)


def display_string(dev, disp_str):
    b = [ord(x) for x in disp_str]
    send_command(dev, CommandVals.SetText, [len(disp_str)] + b)


def display_on_cmd(dev, on):
    send_command(dev, CommandVals.DisplayOn, [on])


def invert_screen_cmd(dev, invert):
    send_command(dev, CommandVals.InvertScreen, [invert])


def screen_saver_cmd(dev, on):
    send_command(dev, CommandVals.ScreenSaver, [on])


def set_fps_cmd(dev, mode):
    res = send_command(dev, CommandVals.SetFps, with_response=True)
    current_fps = res[0]

    if mode == "quarter":
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b000
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("low")
    elif mode == "half":
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b001
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("low")
    elif mode == "one":
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b010
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("low")
    elif mode == "two":
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b011
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("low")
    elif mode == "four":
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b100
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("low")
    elif mode == "eight":
        fps = current_fps & ~LOW_FPS_MASK
        fps |= 0b101
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("low")
    elif mode == "sixteen":
        fps = current_fps & ~HIGH_FPS_MASK
        fps |= 0b00000000
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("high")
    elif mode == "thirtytwo":
        fps = current_fps & ~HIGH_FPS_MASK
        fps |= 0b00010000
        send_command(dev, CommandVals.SetFps, [fps])
        set_power_mode_cmd("high")


def set_power_mode_cmd(dev, mode):
    if mode == "low":
        send_command(dev, CommandVals.SetPowerMode, [0])
    elif mode == "high":
        send_command(dev, CommandVals.SetPowerMode, [1])
    else:
        print("Unsupported power mode")
        sys.exit(1)


def get_power_mode_cmd(dev):
    res = send_command(dev, CommandVals.SetPowerMode, with_response=True)
    current_mode = int(res[0])
    if current_mode == 0:
        print("Current Power Mode: Low Power")
    elif current_mode == 1:
        print("Current Power Mode: High Power")


def get_fps_cmd(dev):
    res = send_command(dev, CommandVals.SetFps, with_response=True)
    current_fps = res[0]
    res = send_command(dev, CommandVals.SetPowerMode, with_response=True)
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
