# B1 Display

A transmissive, mono-color (black/white) screen that's 300x400px in size.
It's 4.2 inches in size and mounted in portrait orientation.
Because it's optimized for power, the recommended framerate is 1 FPS.
But it can go up to 32 FPS.

## Controlling

### Display System Status

For a similar type of display, there's an
[open-source software](https://github.com/mathoudebine/turing-smart-screen-python)
to get systems stats, render them into an image file and send it to the screen.

For this display, we have [a fork](https://github.com/FrameworkComputer/lotus-smart-screen-python).
To run it, just install Python and the dependencies, then run `main.py`.
The configuration (`config.yaml`) is already adapted for this display - 
it should be able to find the display by itself (Windows or Linux).

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
      --image-bw <IMAGE_BW>
          Display black&white image (300x400px)
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
