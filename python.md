# Python script to control Framework Laptop 16 Input Modules

Requirements: Python, [PySimpleGUI](https://www.pysimplegui.org) and optionally [pillow](https://pillow.readthedocs.io/en/stable/index.html)

Use `control.py`. Either the commandline, see `control.py --help` or the graphical version: `control.py --gui`

```
options:
  -h, --help            show this help message and exit
  --bootloader          Jump to the bootloader to flash new firmware
  --sleep, --no-sleep   Simulate the host going to sleep or waking up
  --brightness BRIGHTNESS
                        Adjust the brightness. Value 0-255
  --animate, --no-animate
                        Start/stop vertical scrolling
  --pattern {full,lotus,gradient,double-gradient,zigzag,panic,lotus2}
                        Display a pattern
  --image IMAGE         Display a PNG or GIF image in black and white only)
  --image-grey IMAGE_GREY
                        Display a PNG or GIF image in greyscale
  --percentage PERCENTAGE
                        Fill a percentage of the screen
  --clock               Display the current time
  --string STRING       Display a string or number, like FPS
  --symbols SYMBOLS [SYMBOLS ...]
                        Show symbols (degF, degC, :), snow, cloud, ...)
  --gui                 Launch the graphical version of the program
  --blink               Blink the current pattern
  --breathing           Breathing of the current pattern
  --eq EQ [EQ ...]      Equalizer
  --random-eq           Random Equalizer
  --wpm                 WPM Demo
  --snake               Snake
  --all-brightnesses    Show every pixel in a different brightness
  --set-color {white,black,red,green,blue,cyan,yellow,purple}
                        Set RGB color (C1 Minimal Input Module)
  --get-color           Get RGB color (C1 Minimal Input Module)
  -v, --version         Get device version
  --serial-dev SERIAL_DEV
                        Change the serial dev. Probably /dev/ttyACM0 on Linux, COM0 on Windows
```

Examples

```sh
# Launch graphical application
./control.py --gui

# Show current time and keep updating it
./control.py --clock

# Draw PNG or GIF
./control.py --image stripe.gif
./control.py --image stripe.png

# Change brightness (0-255)
./control.py --brightness 50
```
