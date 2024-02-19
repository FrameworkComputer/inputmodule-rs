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

### Pinout

The neopixel is built-in and can display RGB color.
The UART pins are for debugging, with baud rate 115200.

| QtPy Label        | RP2040 GPIO | Function      |
|-------------------|-------------|---------------|
| Built-in Neopixel | GPIO11      | Neopixel Pwr  |
| Built-in Neopixel | GPIO12      | Neopixel Data |
| TX                | GPIO20      | UART1 TX      |
| RX                | GPIO5       | UART1 RX      |
