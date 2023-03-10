## C1 Minimal Input Module

It's a very minimal input module. Many GPIO pins are exposed so that headers
can be soldered onto them. Additionally there are pads for a WS2812/Neopixel
compatible RGB LED.

When booting up this LED is lit in green color.
Its color and brightness can be controlled via the commands:

```sh
> ./control.py --brightness 255
> ./control.py --get-brightness
Current brightness: 255

> ./control.py --set-color yellow
> ./control.py --get-color
Current color: RGB:(255, 255, 0)
```
