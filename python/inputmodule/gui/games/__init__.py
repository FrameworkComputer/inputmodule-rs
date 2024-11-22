from getkey import getkey, keys
import random
from datetime import datetime, timedelta
import time
import threading

from inputmodule.inputmodule import (
    GameControlVal,
    send_command,
    CommandVals,
    Game,
)
from inputmodule.inputmodule.ledmatrix import (
    show_string,
    WIDTH,
    HEIGHT,
    render_matrix,
)

# Constants
ARG_UP = 0
ARG_DOWN = 1
ARG_LEFT = 2
ARG_RIGHT = 3
ARG_QUIT = 4
ARG_2LEFT = 5
ARG_2RIGHT = 6


def snake_embedded_keyscan(dev):
    while True:
        key_arg = None
        key = getkey()
        if key == keys.UP:
            key_arg = GameControlVal.Up
        elif key == keys.DOWN:
            key_arg = GameControlVal.Down
        elif key == keys.LEFT:
            key_arg = GameControlVal.Left
        elif key == keys.RIGHT:
            key_arg = GameControlVal.Right
        elif key == "q":
            # Quit
            key_arg = GameControlVal.Quit
        if key_arg is not None:
            send_command(dev, CommandVals.GameControl, [key_arg])


def pong_embedded(dev):
    # Start game
    send_command(dev, CommandVals.StartGame, [Game.Pong])

    while True:
        key_arg = None
        key = getkey()
        if key == keys.LEFT:
            key_arg = ARG_LEFT
        elif key == keys.RIGHT:
            key_arg = ARG_RIGHT
        elif key == "a":
            key_arg = ARG_2LEFT
        elif key == "d":
            key_arg = ARG_2RIGHT
        elif key == "q":
            # Quit
            key_arg = ARG_QUIT
        if key_arg is not None:
            send_command(dev, CommandVals.GameControl, [key_arg])


def game_of_life_embedded(dev, arg):
    # Start game
    # TODO: Add a way to stop it
    print("Game", int(arg))
    send_command(dev, CommandVals.StartGame, [Game.GameOfLife, int(arg)])


def snake_embedded(dev):
    # Start game
    send_command(dev, CommandVals.StartGame, [Game.Snake])

    snake_embedded_keyscan(dev)


def wpm_demo(dev):
    """Capture keypresses and calculate the WPM of the last 10 seconds
    TODO: I'm not sure my calculation is right."""
    start = datetime.now()
    keypresses = []
    while True:
        _ = getkey()

        now = datetime.now()
        keypresses = [x for x in keypresses if (now - x).total_seconds() < 10]
        keypresses.append(now)
        # Word is five letters
        wpm = (len(keypresses) / 5) * 6

        total_time = (now - start).total_seconds()
        if total_time < 10:
            wpm = wpm / (total_time / 10)

        show_string(dev, " " + str(int(wpm)))
