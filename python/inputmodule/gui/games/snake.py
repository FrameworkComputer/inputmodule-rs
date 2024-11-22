import pygame
import random
import time

from inputmodule import cli
from inputmodule.inputmodule import ledmatrix

# Set the screen width and height for a 34 x 9 block game
block_width = 20
block_height = 20
COLS = 9
ROWS = 34

WIDTH = COLS * block_width
HEIGHT = ROWS * block_height

# Colors
black = (0, 0, 0)
white = (255, 255, 255)

def opposite_direction(direction):
    if direction == pygame.K_RIGHT:
        return pygame.K_LEFT
    elif direction == pygame.K_LEFT:
        return pygame.K_RIGHT
    elif direction == pygame.K_UP:
        return pygame.K_DOWN
    elif direction == pygame.K_DOWN:
        return pygame.K_UP
    return direction

# Function to get the current board state
def get_board_state(board):
    temp_board = [row[:] for row in board]
    #off_x, off_y = current_pos
    #for y, row in enumerate(current_shape):
    #    for x, cell in enumerate(row):
    #        if cell:
    #            if 0 <= off_y + y < ROWS and 0 <= off_x + x < COLS:
    #                temp_board[off_y + y][off_x + x] = 1
    return temp_board

def draw_ledmatrix(board, devices):
    for dev in devices:
        matrix = [[0 for _ in range(34)] for _ in range(9)]
        for y in range(ROWS):
            for x in range(COLS):
                matrix[x][y] = board[y][x]
        ledmatrix.render_matrix(dev, matrix)
        #vals = [0 for _ in range(39)]
        #send_command(dev, CommandVals.Draw, vals)

# Function to display the score using blocks
def display_score(board, score):
    return
    score_str = str(score)
    start_x = COLS - len(score_str) * 4
    for i, digit in enumerate(score_str):
        if digit.isdigit():
            digit = int(digit)
            for y in range(5):
                for x in range(3):
                    if digit_blocks[digit][y][x]:
                        if y < ROWS and start_x + i * 4 + x < COLS:
                            board[y][start_x + i * 4 + x] = 1

# Digit blocks for representing score
# Each number is represented in a 5x3 block matrix
digit_blocks = [
    [[1, 1, 1], [1, 0, 1], [1, 0, 1], [1, 0, 1], [1, 1, 1]],  # 0
    [[0, 1, 0], [1, 1, 0], [0, 1, 0], [0, 1, 0], [1, 1, 1]],  # 1
    [[1, 1, 1], [0, 0, 1], [1, 1, 1], [1, 0, 0], [1, 1, 1]],  # 2
    [[1, 1, 1], [0, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]],  # 3
    [[1, 0, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [0, 0, 1]],  # 4
    [[1, 1, 1], [1, 0, 0], [1, 1, 1], [0, 0, 1], [1, 1, 1]],  # 5
    [[1, 1, 1], [1, 0, 0], [1, 1, 1], [1, 0, 1], [1, 1, 1]],  # 6
    [[1, 1, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1], [0, 0, 1]],  # 7
    [[1, 1, 1], [1, 0, 1], [1, 1, 1], [1, 0, 1], [1, 1, 1]],  # 8
    [[1, 1, 1], [1, 0, 1], [1, 1, 1], [0, 0, 1], [1, 1, 1]],  # 9
]


class Snake:
    # Function to draw a grid
    def draw_grid(self):
        for y in range(ROWS):
            for x in range(COLS):
                rect = pygame.Rect(x * block_width, y * block_height, block_width, block_height)
                pygame.draw.rect(self.screen, black, rect, 1)

    # Function to draw the game based on the board state
    def draw_board(self, board, devices):
        draw_ledmatrix(board, devices)
        self.screen.fill(white)
        for y in range(ROWS):
            for x in range(COLS):
                if board[y][x]:
                    rect = pygame.Rect(x * block_width, y * block_height, block_width, block_height)
                    pygame.draw.rect(self.screen, black, rect)
        self.draw_grid()
        pygame.display.update()

    # Main game function
    def gameLoop(self, devices):
        board = [[0 for _ in range(COLS)] for _ in range(ROWS)]

        game_over = False
        body = []
        score = 0
        head = (0, 0)
        direction = pygame.K_DOWN
        food = (0, 0)
        while food == head:
            food = (random.randint(0, COLS - 1), random.randint(0, ROWS - 1))
        move_time = 0

        # Setting
        # Wrap and let the snake come out the other side
        WRAP = False
        MOVE_PERIOD = 200

        while not game_over:
            # Draw the current board state
            board_state = get_board_state(board)
            display_score(board_state, score)
            self.draw_board(board_state, devices)

            # Event handling
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    game_over = True

                if event.type == pygame.KEYDOWN:
                    if event.key == opposite_direction(direction) and body:
                        continue
                    if event.key in [pygame.K_LEFT, pygame.K_h]:
                        direction = pygame.K_LEFT
                    elif event.key in [pygame.K_RIGHT, pygame.K_l]:
                        direction = pygame.K_RIGHT
                    elif event.key in [pygame.K_DOWN, pygame.K_j]:
                        direction = pygame.K_DOWN
                    elif event.key in [pygame.K_UP, pygame.K_k]:
                        direction = pygame.K_UP

            move_time += self.clock.get_time()
            if move_time >= MOVE_PERIOD:
                move_time = 0

                # Update position
                (x, y) = head
                oldhead = head
                if direction == pygame.K_LEFT:
                    head = (x - 1, y)
                elif direction == pygame.K_RIGHT:
                    head = (x + 1, y)
                elif direction == pygame.K_DOWN:
                    head = (x, y + 1)
                elif direction == pygame.K_UP:
                    head = (x, y - 1)

                # Detect edge condition
                (x, y) = head
                if head in body:
                    game_over = True
                elif x >= COLS or x < 0 or y >= ROWS or y < 0:
                    if WRAP:
                        if x >= COLS:
                            x = 0
                        elif x < 0:
                            x = COLS - 1
                        elif y >= ROWS:
                            y = 0
                        elif y < 0:
                            y = ROWS - 1
                        head = (x, y)
                    else:
                        game_over = True
                elif head == food:
                    body.insert(0, oldhead)
                    while food == head:
                        food = (random.randint(0, COLS - 1),
                                random.randint(0, ROWS - 1))
                elif body:
                    body.pop()
                    body.insert(0, oldhead)

                # Draw on screen
                if not game_over:
                    board = [[0 for _ in range(COLS)] for _ in range(ROWS)]
                    board[y][x] = 1
                    board[food[1]][food[0]] = 1
                    for bodypart in body:
                        (x, y) = bodypart
                        board[y][x] = 1

            self.clock.tick(30)

        # Flash the screen twice before waiting for restart
        for _ in range(2):
            for dev in devices:
                ledmatrix.percentage(dev, 0)
            self.screen.fill(black)
            pygame.display.update()
            time.sleep(0.3)

            for dev in devices:
                ledmatrix.percentage(dev, 100)
            self.screen.fill(white)
            pygame.display.update()
            time.sleep(0.3)

        # Display final score and wait for restart without clearing the screen
        board_state = get_board_state(board)
        display_score(board_state, score)
        self.draw_board(board_state, devices)

        waiting = True
        while waiting:
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    waiting = False
                    game_over = True
                if event.type == pygame.KEYDOWN:
                    if event.key == pygame.K_q:
                        waiting = False
                    if event.key == pygame.K_r:
                        board = [[0 for _ in range(COLS)] for _ in range(ROWS)]
                        gameLoop()

        pygame.quit()
        quit()

    def __init__(self):
        # Initialize pygame
        pygame.init()

        # Create the screen
        self.screen = pygame.display.set_mode((WIDTH, HEIGHT))

        # Clock to control the speed of the game
        self.clock = pygame.time.Clock()

def main_devices(devices):
    snake = Snake()
    snake.gameLoop(devices)

def main():
    devices = cli.find_devs()

    snake = Snake()
    snake.gameLoop(devices)

if __name__ == "__main__":
    main()
