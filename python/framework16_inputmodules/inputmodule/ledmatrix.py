import time

import serial

import font
from inputmodule import (
    send_command,
    CommandVals,
    PatternVals,
    FWK_MAGIC,
    send_serial,
    brightness,
)

WIDTH = 9
HEIGHT = 34
PATTERNS = [
    "All LEDs on",
    '"LOTUS" sideways',
    "Gradient (0-13% Brightness)",
    "Double Gradient (0-7-0% Brightness)",
    "Zigzag",
    '"PANIC"',
    '"LOTUS" Top Down',
    "All brightness levels (1 LED each)",
    "Every Second Row",
    "Every Third Row",
    "Every Fourth Row",
    "Every Fifth Row",
    "Every Sixth Row",
    "Every Second Col",
    "Every Third Col",
    "Every Fourth Col",
    "Every Fifth Col",
    "Checkerboard",
    "Double Checkerboard",
    "Triple Checkerboard",
    "Quad Checkerboard",
]
PWM_FREQUENCIES = [
    "29kHz",
    "3.6kHz",
    "1.8kHz",
    "900Hz",
]


def get_pwm_freq(dev):
    """Adjust the brightness scaling of the entire screen."""
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


def percentage(dev, p):
    """Fill a percentage of the screen. Bottom to top"""
    send_command(dev, CommandVals.Pattern, [PatternVals.Percentage, p])


def animate(dev, b: bool):
    """Tell the firmware to start/stop animation.
    Scrolls the currently saved grid vertically down."""
    send_command(dev, CommandVals.Animate, [b])


def get_animate(dev):
    """Tell the firmware to start/stop animation.
    Scrolls the currently saved grid vertically down."""
    res = send_command(dev, CommandVals.Animate, with_response=True)
    return bool(res[0])


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
    assert width == 9
    assert height == 34
    pixel_values = list(im.getdata())
    for i, pixel in enumerate(pixel_values):
        brightness = sum(pixel) / 3
        if brightness > 0xFF / 2:
            vals[int(i / 8)] |= 1 << i % 8

    send_command(dev, CommandVals.Draw, vals)


def camera(dev):
    """Play a live view from the webcam, for fun"""
    with serial.Serial(dev.device, 115200) as s:
        import cv2

        capture = cv2.VideoCapture(0)
        ret, frame = capture.read()

        scale_y = HEIGHT / frame.shape[0]

        # Scale the video to 34 pixels height
        dim = (HEIGHT, int(round(frame.shape[1] * scale_y)))
        # Find the starting position to crop the width to be centered
        # For very narrow videos, make sure to stay in bounds
        start_x = max(0, int(round(dim[1] / 2 - WIDTH / 2)))
        end_x = min(dim[1], start_x + WIDTH)

        # Pre-process the video into resized, cropped, grayscale frames
        while True:
            ret, frame = capture.read()
            if not ret:
                print("Failed to capture video frames")
                break

            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)

            resized = cv2.resize(gray, (dim[1], dim[0]))
            cropped = resized[0:HEIGHT, start_x:end_x]

            for x in range(0, cropped.shape[1]):
                vals = [0 for _ in range(HEIGHT)]

                for y in range(0, HEIGHT):
                    vals[y] = cropped[y, x]

                send_col(dev, s, x, vals)
            commit_cols(dev, s)


def video(dev, video_file):
    """Resize and play back a video"""
    with serial.Serial(dev.device, 115200) as s:
        import cv2

        capture = cv2.VideoCapture(video_file)
        ret, frame = capture.read()

        scale_y = HEIGHT / frame.shape[0]

        # Scale the video to 34 pixels height
        dim = (HEIGHT, int(round(frame.shape[1] * scale_y)))
        # Find the starting position to crop the width to be centered
        # For very narrow videos, make sure to stay in bounds
        start_x = max(0, int(round(dim[1] / 2 - WIDTH / 2)))
        end_x = min(dim[1], start_x + WIDTH)

        processed = []

        # Pre-process the video into resized, cropped, grayscale frames
        while True:
            ret, frame = capture.read()
            if not ret:
                print("Failed to read video frames")
                break

            gray = cv2.cvtColor(frame, cv2.COLOR_RGB2GRAY)

            resized = cv2.resize(gray, (dim[1], dim[0]))
            cropped = resized[0:HEIGHT, start_x:end_x]

            processed.append(cropped)

        # Write it out to the module one frame at a time
        # TODO: actually control for framerate
        for frame in processed:
            for x in range(0, cropped.shape[1]):
                vals = [0 for _ in range(HEIGHT)]

                for y in range(0, HEIGHT):
                    vals[y] = frame[y, x]

                send_col(dev, s, x, vals)
            commit_cols(dev, s)


def pixel_to_brightness(pixel):
    """Calculate pixel brightness from an RGB triple"""
    assert len(pixel) == 3
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
        assert width == 9
        assert height == 34
        pixel_values = list(im.getdata())
        for x in range(0, WIDTH):
            vals = [0 for _ in range(HEIGHT)]

            for y in range(HEIGHT):
                vals[y] = pixel_to_brightness(pixel_values[x + y * WIDTH])

            send_col(dev, s, x, vals)
        commit_cols(dev, s)


def send_col(dev, s, x, vals):
    """Stage greyscale values for a single column. Must be committed with commit_cols()"""
    command = FWK_MAGIC + [CommandVals.StageGreyCol, x] + vals
    send_serial(dev, s, command)


def commit_cols(dev, s):
    """Commit the changes from sending individual cols with send_col(), displaying the matrix.
    This makes sure that the matrix isn't partially updated."""
    command = FWK_MAGIC + [CommandVals.DrawGreyColBuffer, 0x00]
    send_serial(dev, s, command)


def checkerboard(dev, n):
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = (([0xFF] * n) + ([0x00] * n)) * int(HEIGHT / 2)
            if x % (n * 2) < n:
                # Rotate once
                vals = vals[n:] + vals[:n]

            send_col(dev, s, x, vals)
        commit_cols(dev, s)


def every_nth_col(dev, n):
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = [(0xFF if x % n == 0 else 0) for _ in range(HEIGHT)]

            send_col(dev, s, x, vals)
        commit_cols(dev, s)


def every_nth_row(dev, n):
    with serial.Serial(dev.device, 115200) as s:
        for x in range(0, WIDTH):
            vals = [(0xFF if y % n == 0 else 0) for y in range(HEIGHT)]

            send_col(dev, s, x, vals)
        commit_cols(dev, s)


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

            send_col(dev, s, x, vals)
        commit_cols(dev, s)


def breathing(dev):
    """Animate breathing brightness.
    Keeps currently displayed grid"""
    # Bright ranges appear similar, so we have to go through those faster
    while True:
        # Go quickly from 250 to 50
        for i in range(10):
            time.sleep(0.03)
            brightness(dev, 250 - i * 20)

        # Go slowly from 50 to 0
        for i in range(10):
            time.sleep(0.06)
            brightness(dev, 50 - i * 5)

        # Go slowly from 0 to 50
        for i in range(10):
            time.sleep(0.06)
            brightness(dev, i * 5)

        # Go quickly from 50 to 250
        for i in range(10):
            time.sleep(0.03)
            brightness(dev, 50 + i * 20)


def eq(dev, vals):
    """Display 9 values in equalizer diagram starting from the middle, going up and down"""
    matrix = [[0 for _ in range(34)] for _ in range(9)]

    for col, val in enumerate(vals[:9]):
        row = int(34 / 2)
        above = int(val / 2)
        below = val - above

        for i in range(above):
            matrix[col][row + i] = 0xFF
        for i in range(below):
            matrix[col][row - 1 - i] = 0xFF

    render_matrix(dev, matrix)


def render_matrix(dev, matrix):
    """Show a black/white matrix
    Send everything in a single command"""
    vals = [0x00 for _ in range(39)]

    for x in range(9):
        for y in range(34):
            i = x + 9 * y
            if matrix[x][y]:
                vals[int(i / 8)] = vals[int(i / 8)] | (1 << i % 8)

    send_command(dev, CommandVals.Draw, vals)


def light_leds(dev, leds):
    """Light a specific number of LEDs"""
    vals = [0x00 for _ in range(39)]
    for byte in range(int(leds / 8)):
        vals[byte] = 0xFF
    for i in range(leds % 8):
        vals[int(leds / 8)] += 1 << i
    send_command(dev, CommandVals.Draw, vals)


def pwm_freq(dev, freq):
    """Display a pattern that's already programmed into the firmware"""
    if freq == "29kHz":
        send_command(dev, CommandVals.PwmFreq, [0])
    elif freq == "3.6kHz":
        send_command(dev, CommandVals.PwmFreq, [1])
    elif freq == "1.8kHz":
        send_command(dev, CommandVals.PwmFreq, [2])
    elif freq == "900Hz":
        send_command(dev, CommandVals.PwmFreq, [3])


def pattern(dev, p):
    """Display a pattern that's already programmed into the firmware"""
    if p == "All LEDs on":
        send_command(dev, CommandVals.Pattern, [PatternVals.FullBrightness])
    elif p == "Gradient (0-13% Brightness)":
        send_command(dev, CommandVals.Pattern, [PatternVals.Gradient])
    elif p == "Double Gradient (0-7-0% Brightness)":
        send_command(dev, CommandVals.Pattern, [PatternVals.DoubleGradient])
    elif p == '"LOTUS" sideways':
        send_command(dev, CommandVals.Pattern, [PatternVals.DisplayLotus])
    elif p == "Zigzag":
        send_command(dev, CommandVals.Pattern, [PatternVals.ZigZag])
    elif p == '"PANIC"':
        send_command(dev, CommandVals.Pattern, [PatternVals.DisplayPanic])
    elif p == '"LOTUS" Top Down':
        send_command(dev, CommandVals.Pattern, [PatternVals.DisplayLotus2])
    elif p == "All brightness levels (1 LED each)":
        all_brightnesses(dev)
    elif p == "Every Second Row":
        every_nth_row(dev, 2)
    elif p == "Every Third Row":
        every_nth_row(dev, 3)
    elif p == "Every Fourth Row":
        every_nth_row(dev, 4)
    elif p == "Every Fifth Row":
        every_nth_row(dev, 5)
    elif p == "Every Sixth Row":
        every_nth_row(dev, 6)
    elif p == "Every Second Col":
        every_nth_col(dev, 2)
    elif p == "Every Third Col":
        every_nth_col(dev, 3)
    elif p == "Every Fourth Col":
        every_nth_col(dev, 4)
    elif p == "Every Fifth Col":
        every_nth_col(dev, 4)
    elif p == "Checkerboard":
        checkerboard(dev, 1)
    elif p == "Double Checkerboard":
        checkerboard(dev, 2)
    elif p == "Triple Checkerboard":
        checkerboard(dev, 3)
    elif p == "Quad Checkerboard":
        checkerboard(dev, 4)
    else:
        print("Invalid pattern")


def show_string(dev, s):
    """Render a string with up to five letters"""
    show_font(dev, [font.convert_font(letter) for letter in str(s)[:5]])


def show_font(dev, font_items):
    """Render up to five 5x6 pixel font items"""
    vals = [0x00 for _ in range(39)]

    for digit_i, digit_pixels in enumerate(font_items):
        offset = digit_i * 7
        for pixel_x in range(5):
            for pixel_y in range(6):
                pixel_value = digit_pixels[pixel_x + pixel_y * 5]
                i = (2 + pixel_x) + (9 * (pixel_y + offset))
                if pixel_value:
                    vals[int(i / 8)] = vals[int(i / 8)] | (1 << i % 8)

    send_command(dev, CommandVals.Draw, vals)


def show_symbols(dev, symbols):
    """Render a list of up to five symbols
    Can use letters/numbers or symbol names, like 'sun', ':)'"""
    font_items = []
    for symbol in symbols:
        s = font.convert_symbol(symbol)
        if not s:
            s = font.convert_font(symbol)
        font_items.append(s)

    show_font(dev, font_items)
