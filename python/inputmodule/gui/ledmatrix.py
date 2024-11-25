from datetime import datetime, timedelta
import time
import random

from inputmodule.gui.gui_threading import (
    reset_thread,
    is_thread_stopped,
    is_dev_disconnected,
    set_status,
    get_status,
)
from inputmodule.inputmodule.ledmatrix import (
    light_leds,
    show_string,
    eq,
    breathing,
    animate,
)
from inputmodule.inputmodule import brightness

def countdown(dev, seconds):
    """Run a countdown timer. Lighting more LEDs every 100th of a seconds.
    Until the timer runs out and every LED is lit"""
    animate(dev, False)
    set_status('countdown')
    start = datetime.now()
    target = seconds * 1_000_000
    while get_status() == 'countdown':
        if is_thread_stopped() or is_dev_disconnected(dev.device):
            reset_thread()
            return
        now = datetime.now()
        passed_time = (now - start) / timedelta(microseconds=1)

        ratio = passed_time / target
        if passed_time >= target:
            break

        leds = int(306 * ratio)
        light_leds(dev, leds)

        time.sleep(0.01)

    if get_status() == 'countdown':
        light_leds(dev, 306)
        breathing(dev)
        # blinking(dev)


def blinking(dev):
    """Blink brightness high/off every second.
    Keeps currently displayed grid"""
    set_status('blinking')
    while get_status() == 'blinking':
        if is_thread_stopped() or is_dev_disconnected(dev.device):
            reset_thread()
            return
        brightness(dev, 0)
        time.sleep(0.5)
        brightness(dev, 200)
        time.sleep(0.5)


def random_eq(dev):
    """Display an equlizer looking animation with random values."""
    animate(dev, False)
    set_status('random_eq')
    while get_status() == 'random_eq':
        if is_thread_stopped() or is_dev_disconnected(dev.device):
            reset_thread()
            return
        # Lower values more likely, makes it look nicer
        weights = [i * i for i in range(33, 0, -1)]
        population = list(range(1, 34))
        vals = random.choices(population, weights=weights, k=9)
        eq(dev, vals)
        time.sleep(0.2)


def clock(dev):
    """Render the current time and display.
    Loops forever, updating every second"""
    animate(dev, False)
    set_status('clock')
    while get_status() == 'clock':
        if is_thread_stopped() or is_dev_disconnected(dev.device):
            reset_thread()
            return
        now = datetime.now()
        current_time = now.strftime("%H:%M")
        print("Current Time =", current_time)

        show_string(dev, current_time)
        time.sleep(1)
