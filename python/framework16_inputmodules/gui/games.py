from getkey import getkey, keys
import random
from datetime import datetime, timedelta
import time
import threading

from framework16_inputmodules.inputmodule import (
    GameControlVal,
    send_command,
    CommandVals,
    Game,
)
from framework16_inputmodules.inputmodule.ledmatrix import (
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

# Variables
direction = None
body = []


def opposite_direction(direction):
    if direction == keys.RIGHT:
        return keys.LEFT
    elif direction == keys.LEFT:
        return keys.RIGHT
    elif direction == keys.UP:
        return keys.DOWN
    elif direction == keys.DOWN:
        return keys.UP
    return direction


def snake_keyscan():
    global direction
    global body

    while True:
        current_dir = direction
        key = getkey()
        if key in [keys.RIGHT, keys.UP, keys.LEFT, keys.DOWN]:
            # Don't allow accidental suicide if we have a body
            if key == opposite_direction(current_dir) and body:
                continue
            direction = key


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


def game_over(dev):
    global body
    while True:
        show_string(dev, "GAME ")
        time.sleep(0.75)
        show_string(dev, "OVER!")
        time.sleep(0.75)
        score = len(body)
        show_string(dev, f"{score:>3} P")
        time.sleep(0.75)


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


def snake(dev):
    global direction
    global body
    head = (0, 0)
    direction = keys.DOWN
    food = (0, 0)
    while food == head:
        food = (random.randint(0, WIDTH - 1), random.randint(0, HEIGHT - 1))

    # Setting
    WRAP = False

    thread = threading.Thread(target=snake_keyscan, args=(), daemon=True)
    thread.start()

    prev = datetime.now()
    while True:
        now = datetime.now()
        delta = (now - prev) / timedelta(milliseconds=1)

        if delta > 200:
            prev = now
        else:
            continue

        # Update position
        (x, y) = head
        oldhead = head
        if direction == keys.RIGHT:
            head = (x + 1, y)
        elif direction == keys.LEFT:
            head = (x - 1, y)
        elif direction == keys.UP:
            head = (x, y - 1)
        elif direction == keys.DOWN:
            head = (x, y + 1)

        # Detect edge condition
        (x, y) = head
        if head in body:
            return game_over(dev)
        elif x >= WIDTH or x < 0 or y >= HEIGHT or y < 0:
            if WRAP:
                if x >= WIDTH:
                    x = 0
                elif x < 0:
                    x = WIDTH - 1
                elif y >= HEIGHT:
                    y = 0
                elif y < 0:
                    y = HEIGHT - 1
                head = (x, y)
            else:
                return game_over(dev)
        elif head == food:
            body.insert(0, oldhead)
            while food == head:
                food = (random.randint(0, WIDTH - 1), random.randint(0, HEIGHT - 1))
        elif body:
            body.pop()
            body.insert(0, oldhead)

        # Draw on screen
        matrix = [[0 for _ in range(HEIGHT)] for _ in range(WIDTH)]
        matrix[x][y] = 1
        matrix[food[0]][food[1]] = 1
        for bodypart in body:
            (x, y) = bodypart
            matrix[x][y] = 1
        render_matrix(dev, matrix)


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
