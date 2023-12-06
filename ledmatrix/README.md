# LED Matrix

It's a 9x34 (306) LED matrix, controlled by RP2040 MCU and IS31FL3741A LED controller.

Connection to the host system is via USB 2.0 and currently there is a USB Serial API to control it without reflashing.

- Commands
  - Display various pre-programmed patterns
  - Light up a percentage of the screen
  - Change brightness
  - Send a black/white image to the display
  - Send a greyscale image to the display
  - Scroll and loop the display content vertically
  - A commandline script and graphical application to control it
- Sleep Mode
  - Transition slowly turns off/on the LEDs

## Controlling

### Commandline

```
> inputmodule-control led-matrix
LED Matrix

Usage: ipc led-matrix [OPTIONS]

Options:
      --brightness [<BRIGHTNESS>]
          Set LED max brightness percentage or get, if no value provided
      --sleeping [<SLEEPING>]
          Set sleep status or get, if no value provided [possible values: true, false]
      --bootloader
          Jump to the bootloader
      --percentage <PERCENTAGE>
          Display a percentage (0-100)
      --animate [<ANIMATE>]
          Start/stop animation [possible values: true, false]
      --pattern <PATTERN>
          Display a pattern [possible values: percentage, gradient, double-gradient, lotus-sideways, zigzag, all-on, panic, lotus-top-down]
      --all-brightnesses
          Show every brightness, one per pixel
      --blinking
          Blink the current pattern once a second
      --breathing
          Breathing brightness of the current pattern
      --image-bw <IMAGE_BW>
          Display black&white image (9x34px)
      --image-gray <IMAGE_GRAY>
          Display grayscale image
      --random-eq
          Random EQ
      --eq <EQ> <EQ> <EQ> <EQ> <EQ> <EQ> <EQ> <EQ> <EQ>
          EQ with custom values
      --clock
          Show the current time
      --string <STRING>
          Display a string (max 5 chars)
      --symbols [<SYMBOLS>...]
          Display a string (max 5 symbols)
      --start-game <START_GAME>
          Start a game [possible values: snake, pong, tetris, game-of-life]
      --game-param <GAME_PARAM>
          Paramater for starting the game. Required for some games [possible values: current-matrix, pattern1, blinker, toad, beacon, glider]
      --stop-game
          Stop the currently running game
      --animation-fps [<ANIMATION_FPS>]
          Set/get animation FPS
      --panic
          Crash the firmware (TESTING ONLY!)
  -v, --version
          Get the device version
  -h, --help
          Print help
```

### Non-trivial Examples

Most commandline arguments should be self-explanatory.
If not, please open an issue.
Those that require an argument or setup have examples here:

###### Percentage

Light up a percentage of the module. From bottom to top.
This could be used to show volume level, progress of something, or similar.

```sh
inputmodule-control led-matrix --percentage 30
```

###### Display an Image

Display an image (tested with PNG and GIF). It must be 9x34 pixels in size. It
doesn't have to be black/white or grayscale. The program will calculate the
brightness of each pixel. But if the brightness doesn't vary enough, it won't
look good.
Two example images are included in the repository.

```sh
# Convert image to black/white and display
inputmodule-control led-matrix --image-bw stripe.gif

# Convert image to grayscale and display
inputmodule-control led-matrix --image-gray grayscale.gif
```

###### Random equalizer
To show off the equalizer use-case, this command generates a
random but authentic looking equalizer pattern until the command is terminated.

Alternatively you can provide 9 EQ values yourself. A script might capture
audio input and feed it into this command.

```sh
inputmodule-control led-matrix --random-eq
inputmodule-control led-matrix --eq 1 2 3 4 5 4 3 2 1
```

###### Input equalizer

This command generates an equalizer-like visualization of the current audio input (microphone).
It supports most platforms - for details, see [documentation of the cpal crate](https://github.com/RustAudio/cpal).

You must compile the `inputmodule-control` binary with the `audio-visualization` feature on:
`cargo build --features audio-visualizations --target x86_64-unknown-linux-gnu -p inputmodule-control`

Once compiled, you can use the `--input-eq` arg to try the visualizer:
```sh
inputmodule-control led-matrix --input-eq
```

###### Custom string

Display a custom string of up to 5 characters.
Currently only uppercase A-Z, 0-9 and some punctuation is implemented.

```sh
inputmodule-control led-matrix --string "LOTUS"
```

The symbols parameter is much more powerful, it can also show extra symbols.
The full list of symbols is defined [here](https://github.com/FrameworkComputer/led_matrix_fw/blob/main/inputmodule-control/src/font.rs).

```sh
# Show 0 °C, a snow icon and a smiley
inputmodule-control led-matrix --symbols 0 degC ' ' snow ':)'
```

###### Games

While the game commands are implemented, the controls don't take easy keyboard
input.
Instead try out the [Python script](../python.md):

```sh
# Snake
./ledmatrix_control.py --snake

# Pong (Seems broken at the moment)
./ledmatrix_control.py --pong-embedded
```

###### Game of Life

[Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life)
needs a parameter to start. Choose either one of the preprogrammed starting patterns.
Or display whatever you like using the other commands and have the game start based on that.
Font patterns generally look pretty good and survive for a while or even stay alive forever.

The game board wraps around the edges to make gliders possible that move continuously.

```sh
# Start from the currently displayed pattern
inputmodule-control led-matrix --start-game game-of-life --game-param current-matrix

# Show two gliders that move forever
inputmodule-control led-matrix --start-game game-of-life --game-param glider
```

If you want to display something else, either reset the module (unplugging) or
run the stop command.

```sh
inputmodule-control led-amtrix --stop-game
```

## Sleep Behavior

Currently sleeping means all LEDs and the LED controller are turned off.
Transitions of sleep state slowly fade the LEDs on or off.

Optionally the firmware can be configured, at build-time, to turn the LEDs
on/off immediately. Or display "SLEEP" instead of turning the LEDs off, which
is useful for debugging whether the device is sleeping or not powered.


###### Changing Sleep State

What can change the sleep state

- Hardware/OS triggers
  - `SLEEP#` pin
  - USB Suspend
- Software/Firmware Triggers
  - Sleep/Wake or other command via USB Serial
  - Idle timer

Both of the hardware/OS triggers change the sleep state if they transition from one state to another.
For example, if USB suspends, the LED matrix turns off. If it resumes, the LEDs come back on.
Same for the `SLEEP#` pin.
If either of them indicates sleep, even if they didn'td change state, the module goes to sleep.
If they're active, they don't influence module state. That way sleep state can be controlled by commands and isn't overridden immediately.

The sleep/wake command always changes the state. But it can't be received when USB is suspended.
Any other command will also wake up the device.

The idle timer will send the device to sleep after a configured timeout (default 60 seconds).
The idle timer is reset once the device wakes up or once it receives a command.

## DIP Switch

LED Matrix hardware since DVT2 (September 2023) has a DIP switch with two
switches, let's call them DIP1 and DIP2.

###### DIP2 (Bootloader)

DIP2 is the bootloader switch. To enter bootloader mode follow these steps:

1. Unplug module and flip the switch to ON
2. Plug module back in, it will appear as a flash drive with the name `RPI-RP2`
3. Copy the firmware `.uf2` file onto that drive, it will automatically flash and reappear as a flash drive
4. To exit bootloader mode, unplug the module to flip the switch back, and plug it back in
5. Now the new firmware should be running

As a side effect of being in bootloader mode, the LEDs all stay off.

###### DIP1 (General Purpose)

DIP1 could serve many purposes. Currently it is configured to enable the debug mode.
When debug mode is enabled and the module goes to sleep, it will not turn the LEDs off to save power.
Instead it will display the reason why it went to sleep. This is useful for debugging module and host system behavior.
Debug mode will start up to a fully lit matrix and never goes to sleep based on a timeout.

Sleep Reasons can be:

- `SLEEP#` pin: `SLP#`
- USB Suspend: `USB`
- Command: `CMD`
