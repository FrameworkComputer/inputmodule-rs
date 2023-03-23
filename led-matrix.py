#!/usr/bin/env python3
# Script to generate the mapping of LEDs to the CS and SW registers of the
# IS31FL3741A controller.
#
# The output looks like:
# (0x00, 0), // x:1, y:1, sw:1, cs:1, id:1
# (0x1e, 0), // x:2, y:1, sw:2, cs:1, id:2
# [...]

from dataclasses import dataclass

WIDTH = 9
HEIGHT = 34


@dataclass
class Led:
    id: int
    x: int
    y: int
    # Values from 1 to 9
    sw: int
    # Values from 1 to 34
    cs: int

    def led_register(self):
        # See the IS31FL3741A for how the data pages are separated
        if self.cs <= 30 and self.sw >= 7:
            page = 1
            register = self.cs - 1 + (self.sw-7) * 30
        if self.cs <= 30 and self.sw <= 6:
            page = 0
            register = self.cs - 1 + (self.sw-1) * 30
        if self.cs >= 31:
            page = 1
            register = 0x5A + self.cs - 31 + (self.sw-1) * 9
        return (register, page)


def get_leds():
    leds = []

    # Generate LED mapping as how they are mapped in the Framework Laptop 16 LED Matrix Module

    # First down and then right
    # CS1 through CS4
    for cs in range(1, 5):
        for sw in range(1, WIDTH+1):
            leds.append(Led(id=WIDTH * (cs-1) + sw, x=sw, y=cs, sw=sw, cs=cs))

    # First right and then down
    # CS5 through CS7
    base_cs = 4
    base_id = WIDTH * base_cs
    for cs in range(1, 5):
        for sw in range(1, WIDTH+1):
            leds.append(Led(id=base_id + 4 * (sw-1) + cs, x=sw,
                        y=cs+base_cs, sw=sw, cs=cs+base_cs))

    # First right and then down
    # CS9 through CS16
    base_cs = 8
    base_id = WIDTH * base_cs
    for cs in range(1, 9):
        for sw in range(1, WIDTH+1):
            leds.append(Led(id=base_id + 8 * (sw-1) + cs, x=sw,
                        y=cs+base_cs, sw=sw, cs=cs+base_cs))

    # First right and then down
    # CS17 through CS32
    base_cs = 16
    base_id = WIDTH * base_cs
    for cs in range(1, 17):
        for sw in range(1, WIDTH+1):
            leds.append(Led(id=base_id + 16 * (sw-1) + cs, x=sw,
                        y=cs+base_cs, sw=sw, cs=cs+base_cs))

    # First down and then right
    # CS33 through CS34
    base_cs = 32
    base_id = WIDTH * base_cs
    for cs in range(1, 3):
        for sw in range(1, WIDTH+1):
            leds.append(Led(id=base_id + 9 * (cs-1) + sw, x=sw,
                        y=cs+base_cs, sw=sw, cs=cs+base_cs))

    return leds


def main():
    # Assumes that the index in the LEDs list is: index = x + y * 9
    leds = get_leds()

    debug = False

    for led in leds:
        (register, page) = led.led_register()
        if debug:
            print(led, "(0x{:02x}, {})".format(register, page))
        else:
            print("(0x{:02x}, {}), // x:{}, y:{}, sw:{}, cs:{}, id:{}".format(
                register, page, led.x, led.y, led.sw, led.cs, led.id))
    # print_led(leds, 0, 30)

# For debugging


def get_led(leds, x, y):
    return leds[x + y * WIDTH]


# For debugging
def print_led(leds, x, y):
    led = get_led(leds, x, y)
    (register, page) = led.led_register()
    print(led, "(0x{:02x}, {})".format(register, page))


if __name__ == "__main__":
    main()
