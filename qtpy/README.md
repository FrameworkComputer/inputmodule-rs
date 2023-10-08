## QT PY RP2040

**NOT** an official Framework module.
Just reference firmware that's easy to get started with, without having a
Framework module. Has GPIO and WS2812/Neopixel compatible RGB LED.

When booting up this LED is lit in green color.
Its color and brightness can be controlled via the commands:

```sh
> ./ledmatrix_control.py --brightness 255
> ./ledmatrix_control.py --get-brightness
Current brightness: 255

> ./ledmatrix_control.py --set-color yellow
> ./ledmatrix_control.py --get-color
Current color: RGB:(255, 255, 0)
```
