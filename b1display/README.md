# B1 Display

A transmissive, mono-color (black/white) screen that's 300x400px in size.
It's 4.2 inches in size and mounted in portrait orientation.
Because it's optimized for power, the recommended framerate is 1 FPS.
But it can go up to 32 FPS.

The current panel is susceptible to image retention, so the display will start
up with the screen saver. If you send a command to draw anything on the display,
the screensaver will exit.
Currently it does not re-appear after a timeout, it will only re-appear on the
next power-on or after waking from sleep.

## Controlling

### Display System Status

For a similar type of display, there's an
[open-source software](https://github.com/mathoudebine/turing-smart-screen-python)
to get systems stats, render them into an image file and send it to the screen.

For this display, we have [a fork](https://github.com/FrameworkComputer/lotus-smart-screen-python).
To run it, just install Python and the dependencies, then run `main.py`.
The configuration (`config.yaml`) is already adapted for this display - 
it should be able to find the display by itself (Windows or Linux).

###### Configuration

Check out the [upstream documentation](https://github.com/mathoudebine/turing-smart-screen-python/wiki/System-monitor-%3A-themes)
for more information about editing themes.

Currently we have two themes optimized for this display: `B1Terminal` and `B1Blank`.

`B1Terminal` comes pre-configured with lots of system stats.

`B1Blank` comes configured as rendering the text in `file1.txt` onto the screen.

Both can be fully customized by changing the background image and the displayed statistics
in `res/themes/{B1Blank,B1Terminal}/background.png` and `res/themes/{B1Blank,B1Terminal}/theme.yaml`
respectively.

### Commandline

```
> ./inputmodule-control b1-display
B1 Display

Usage: ipc b1-display [OPTIONS]

Options:
      --sleeping [<SLEEPING>]
          Set sleep status or get, if no value provided [possible values: true, false]
      --bootloader
          Jump to the bootloader
      --panic
          Crash the firmware (TESTING ONLY!)
  -v, --version
          Get the device version
      --display-on [<DISPLAY_ON>]
          Turn display on/off [possible values: true, false]
      --pattern <PATTERN>
          Display a simple pattern [possible values: white, black]
      --invert-screen [<INVERT_SCREEN>]
          Invert screen on/off [possible values: true, false]
      --screen-saver [<SCREEN_SAVER>]
          Screensaver on/off [possible values: true, false]
      --fps [<FPS>]
          Set/get FPS [possible values: quarter, half, one, two, four, eight, sixteen, thirty-two]
      --power-mode [<POWER_MODE>]
          Set/get power mode [possible values: low, high]
      --animation-fps [<ANIMATION_FPS>]
          Set/get animation FPS
      --image <IMAGE>
          Display a black&white image (300x400px)
      --animated-gif <ANIMATED_GIF>
          Display an animated black&white GIF (300x400px)
      --clear-ram
          Clear display RAM
  -h, --help
          Print help
```

### Non-trivial Examples

###### Display an Image

Display an image (tested with PNG and GIF). It must be 300x400 pixels in size.
It doesn't have to be black/white. The program will calculate the brightness of
each pixel. But if the brightness doesn't vary enough, it won't look good. One
example image is included in the repository.

```sh
# Should show the Framework Logo and a Lotus flower
inputmodule-control b1-display --image-bw b1display.gif
```

###### Invert the colors (dark-mode)

Since the screen is just black and white, you can display black text on a
white/light background. This can be turned into dark mode by inverting the
colors, making it show light text on a black background.

```sh
# Invert on
> inputmodule-control b1-display --invert-screen true

# Invert off
> inputmodule-control b1-display --invert-screen false

# Check if currently inverted
> inputmodule-control b1-display --invert-screen
Currently inverted: false
```
