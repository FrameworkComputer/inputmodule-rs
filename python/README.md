# Framework Laptop 16 - Input Module Software

This repository contains a python library and scripts to control the
(non-keyboard) input modules, which is currently just the LED Matrix.

## Installing

Pre-requisites: Python with pip

```sh
python3 -m pip install framework16_inputmodule
```

## Control from the host

To build your own application see the: [API command documentation](https://github.com/FrameworkComputer/inputmodule-rs/tree/main/commands.md)

###### Permissions on Linux
To ensure that the input module's port is accessible, install the `udev` rule and trigger a reload:

```
sudo cp release/50-framework-inputmodule.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger
```

##### Common commands:

###### Listing available devices

```sh
> ledmatrixctl
More than 1 compatible device found. Please choose with --serial-dev ...
Example on Windows: --serial-dev COM3
Example on Linux:   --serial-dev /dev/ttyACM0
/dev/ttyACM1
  VID:     0x32AC
  PID:     0x0020
  SN:      FRAKDEBZ0100000000
  Product: LED Matrix Input Module
/dev/ttyACM0
  VID:     0x32AC
  PID:     0x0020
  SN:      FRAKDEBZ0100000000
  Product: LED Matrix Input Module
```

###### Apply command to single device

When there are multiple devices you need to select which one to control.

```
# Example on Linux
> ledmatrixctl --serial-dev /dev/ttyACM0 --percentage 33

# Example on Windows
> ledmatrixctl --serial-dev COM5 --percentage 33
```

### Graphical Application

Launch the graphical application

```sh
# Either via the commandline
ledmatrixctl --gui

# Or using the standanlone application
ledmatrixgui
```

### Other example commands

```sh

# Show current time and keep updating it
ledmatrixctl --clock

# Draw PNG or GIF
ledmatrixctl --image stripe.gif
ledmatrixctl --image stripe.png

# Change brightness (0-255)
ledmatrixctl --brightness 50
```

### All commandline options

```
> ledmatrixctl --help
options:
  -h, --help            show this help message and exit
  -l, --list            List all compatible devices
  --bootloader          Jump to the bootloader to flash new firmware
  --sleep, --no-sleep   Simulate the host going to sleep or waking up
  --is-sleeping         Check current sleep state
  --brightness BRIGHTNESS
                        Adjust the brightness. Value 0-255
  --get-brightness      Get current brightness
  --animate, --no-animate
                        Start/stop vertical scrolling
  --get-animate         Check if currently animating
  --pwm {29000,3600,1800,900}
                        Adjust the PWM frequency. Value 0-255
  --get-pwm             Get current PWM Frequency
  --pattern {...}       Display a pattern
  --image IMAGE         Display a PNG or GIF image in black and white only)
  --image-grey IMAGE_GREY
                        Display a PNG or GIF image in greyscale
  --camera              Stream from the webcam
  --video VIDEO         Play a video
  --percentage PERCENTAGE
                        Fill a percentage of the screen
  --clock               Display the current time
  --string STRING       Display a string or number, like FPS
  --symbols SYMBOLS [SYMBOLS ...]
                        Show symbols (degF, degC, :), snow, cloud, ...)
  --gui                 Launch the graphical version of the program
  --panic               Crash the firmware (TESTING ONLY)
  --blink               Blink the current pattern
  --breathing           Breathing of the current pattern
  --eq EQ [EQ ...]      Equalizer
  --random-eq           Random Equalizer
  --wpm                 WPM Demo
  --snake               Snake
  --snake-embedded      Snake on the module
  --pong-embedded       Pong on the module
  --game-of-life-embedded {currentmatrix,pattern1,blinker,toad,beacon,glider}
                        Game of Life
  --quit-embedded-game  Quit the current game
  --all-brightnesses    Show every pixel in a different brightness
  -v, --version         Get device version
  --serial-dev SERIAL_DEV
                        Change the serial dev. Probably /dev/ttyACM0 on Linux, COM0 on Windows
```

## Update the Firmware

First, put the module into bootloader mode.

This can be done either by flipping DIP switch #2 or
by using one of the following commands: 

```sh
> ledmatrixctl --bootloader
```

Then the module will present itself in the same way as a USB thumb drive.
Copy the UF2 firmware file onto it and the device will flash and reset automatically.
```

### Check the firmware version of the device

```sh
> ledmatrixctl --version
Device Version: 0.1.7
```

###### By looking at the USB descriptor

On Linux:

```sh
> lsusb -d 32ac: -v 2> /dev/null | grep -P 'ID 32ac|bcdDevice'
Bus 003 Device 078: ID 32ac:0020 Framework Computer Inc LED Matrix Input Module
  bcdDevice            0.17
```

