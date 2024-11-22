# Run like
# python3 ledris.py

import pygame
import random
import time

# Initialize pygame
pygame.init()

# Set the screen width and height for a 34 x 9 block Ledris game
block_width = 20
block_height = 20
cols = 9
rows = 34

width = cols * block_width
height = rows * block_height

# Colors
black = (0, 0, 0)
white = (255, 255, 255)

# Create the screen
screen = pygame.display.set_mode((width, height))

# Clock to control the speed of the game
clock = pygame.time.Clock()

# Ledrimino shapes
shapes = [
    [[1, 1, 1, 1]],  # I shape
    [[1, 1], [1, 1]],  # O shape
    [[0, 1, 0], [1, 1, 1]],  # T shape
    [[1, 1, 0], [0, 1, 1]],  # S shape
    [[0, 1, 1], [1, 1, 0]],  # Z shape
    [[1, 1, 1], [1, 0, 0]],  # L shape
    [[1, 1, 1], [0, 0, 1]]   # J shape
]

# Function to get the current board state
def get_board_state(board, current_shape, current_pos):
    temp_board = [row[:] for row in board]
    off_x, off_y = current_pos
    for y, row in enumerate(current_shape):
        for x, cell in enumerate(row):
            if cell:
                if 0 <= off_y + y < rows and 0 <= off_x + x < cols:
                    temp_board[off_y + y][off_x + x] = 1
    return temp_board

# Function to draw the game based on the board state
def draw_board(board, devices):
    screen.fill(white)
    for y in range(rows):
        for x in range(cols):
            if board[y][x]:
                rect = pygame.Rect(x * block_width, y * block_height, block_width, block_height)
                pygame.draw.rect(screen, black, rect)
    draw_grid()
    pygame.display.update()

# Function to draw a grid
def draw_grid():
    for y in range(rows):
        for x in range(cols):
            rect = pygame.Rect(x * block_width, y * block_height, block_width, block_height)
            pygame.draw.rect(screen, black, rect, 1)

# Function to check if the position is valid
def check_collision(board, shape, offset):
    off_x, off_y = offset
    for y, row in enumerate(shape):
        for x, cell in enumerate(row):
            if cell:
                if x + off_x < 0 or x + off_x >= cols or y + off_y >= rows:
                    return True
                if y + off_y >= 0 and board[y + off_y][x + off_x]:
                    return True
    return False

# Function to merge the shape into the board
def merge_shape(board, shape, offset):
    off_x, off_y = offset
    for y, row in enumerate(shape):
        for x, cell in enumerate(row):
            if cell:
                if 0 <= off_y + y < rows and 0 <= off_x + x < cols:
                    board[off_y + y][off_x + x] = 1

# Function to clear complete rows
def clear_rows(board):
    new_board = [row for row in board if any(cell == 0 for cell in row)]
    cleared_rows = rows - len(new_board)
    while len(new_board) < rows:
        new_board.insert(0, [0 for _ in range(cols)])
    return new_board, cleared_rows

# Function to display the score using blocks
def display_score(board, score):
    score_str = str(score)
    start_x = cols - len(score_str) * 4
    for i, digit in enumerate(score_str):
        if digit.isdigit():
            digit = int(digit)
            for y in range(5):
                for x in range(3):
                    if digit_blocks[digit][y][x]:
                        if y < rows and start_x + i * 4 + x < cols:
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

# Main game function
def gameLoop(devices):
    board = [[0 for _ in range(cols)] for _ in range(rows)]
    current_shape = random.choice(shapes)
    current_pos = [cols // 2 - len(current_shape[0]) // 2, 5]  # Start below the score display
    game_over = False
    fall_time = 0
    fall_speed = 500  # Falling speed in milliseconds
    score = 0

    while not game_over:
        # Adjust falling speed based on score
        fall_speed = max(100, 500 - (score * 10))

        # Draw the current board state
        board_state = get_board_state(board, current_shape, current_pos)
        display_score(board_state, score)
        draw_board(board_state, devices)

        # Event handling
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                game_over = True

            if event.type == pygame.KEYDOWN:
                if event.key in [pygame.K_LEFT, pygame.K_h]:
                    new_pos = [current_pos[0] - 1, current_pos[1]]
                    if not check_collision(board, current_shape, new_pos):
                        current_pos = new_pos
                elif event.key in [pygame.K_RIGHT, pygame.K_l]:
                    new_pos = [current_pos[0] + 1, current_pos[1]]
                    if not check_collision(board, current_shape, new_pos):
                        current_pos = new_pos
                elif event.key in [pygame.K_DOWN, pygame.K_j]:
                    new_pos = [current_pos[0], current_pos[1] + 1]
                    if not check_collision(board, current_shape, new_pos):
                        current_pos = new_pos
                elif event.key in [pygame.K_UP, pygame.K_k]:
                    rotated_shape = list(zip(*current_shape[::-1]))
                    if not check_collision(board, rotated_shape, current_pos):
                        current_shape = rotated_shape
                elif event.key == pygame.K_SPACE:  # Hard drop
                    while not check_collision(board, current_shape, [current_pos[0], current_pos[1] + 1]):
                        current_pos[1] += 1

        # Automatic falling
        fall_time += clock.get_time()
        if fall_time >= fall_speed:
            fall_time = 0
            new_pos = [current_pos[0], current_pos[1] + 1]
            if not check_collision(board, current_shape, new_pos):
                current_pos = new_pos
            else:
                merge_shape(board, current_shape, current_pos)
                board, cleared_rows = clear_rows(board)
                score += cleared_rows  # Increase score by one for each row cleared
                current_shape = random.choice(shapes)
                current_pos = [cols // 2 - len(current_shape[0]) // 2, 5]  # Start below the score display
                if check_collision(board, current_shape, current_pos):
                    game_over = True

        clock.tick(30)

    # Flash the screen twice before waiting for restart
    for _ in range(2):
        screen.fill(black)
        pygame.display.update()
        time.sleep(0.3)
        screen.fill(white)
        pygame.display.update()
        time.sleep(0.3)

    # Display final score and wait for restart without clearing the screen
    board_state = get_board_state(board, current_shape, current_pos)
    display_score(board_state, score)
    draw_board(board_state, devices)

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
                    board = [[0 for _ in range(cols)] for _ in range(rows)]
                    gameLoop()

    pygame.quit()
    quit()

gameLoop(devices)
