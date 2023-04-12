from yummy import model
from yummy import room
import random

GAME_STATUS_PLAYER_1_WIN = 0
GAME_STATUS_PLAYER_2_WIN = 1
GAME_STATUS_DRAW = 2

BASE_BOARD = [['-', '-', '-'], 
              ['-', '-', '-'],
              ['-', '-', '-']]


def fix_spot(board, row, col, player):
    board[row][col] = player


def is_player_win(board, player):
    win = None

    n = len(board)

    # checking rows
    for i in range(n):
        win = True
        for j in range(n):
            if board[i][j] != player:
                win = False
                break
        if win:
            return win

    # checking columns
    for i in range(n):
        win = True
        for j in range(n):
            if board[j][i] != player:
                win = False
                break
        if win:
            return win

    # checking diagonals
    win = True
    for i in range(n):
        if board[i][i] != player:
            win = False
            break
    if win:
        return win

    win = True
    for i in range(n):
        if board[i][n - 1 - i] != player:
            win = False
            break
    if win:
        return win
    return False


def is_board_filled(board):
    for row in board:
        for item in row:
            if item == '-':
                return False
    return True


def swap_player_turn(player):
    return 'X' if player == 'O' else 'O'


def get_random_first_player():
    return random.randint(0, 1)


def play(board, row, col, player):
    fix_spot(board, row, col, player)

    if is_player_win(board, player):
        print(f"Player {player} wins the game!")
        return (True, player)

    # checking whether the game is draw or not
    if is_board_filled(board):
        print("Match Draw!")
        return (True, None)

    return (False, swap_player_turn(board, player))


def pre_create_room(model: model.CreateRoom):
    # 2 player is more than enough
    model.set_max_user(2)


def post_create_room(model: model.CreateRoom, success: bool):
    if success:
        # Ok, successfully room created
        print("ROOM CREATED")
        
        # Get metas to update player information
        metas = model.get_metas()

        if metas is None:
            metas = {}

        # First player is X
        metas["player-1"]    = model.get_user_id()
        metas["player-2"]    = None
        metas["next-player"] = 'X'

        # Copy play board
        metas["board"] = BASE_BOARD.copy()

        model.set_metas(metas)

def post_join_to_room(model: model.JoinToRoom, success: bool):
    if success:
        metas = room.get_room_metas(model.get_room_id())

        if metas is None:
            raise Exception("Something went wrong. Sorry.")
        
        metas["player-2"] = model.get_user_id()

        # Lets find who will start first
        value = random.randint(0, 1)
        if value == 1:
            metas["X"] = metas["player-1"]
            metas["O"] = metas["player-2"]
        else:
            metas["X"] = metas["player-2"]
            metas["O"] = metas["player-1"]

        
