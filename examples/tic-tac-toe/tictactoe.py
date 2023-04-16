from yummy import model
from yummy import room
from yummy import constants
from yummy import fail
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

    return (False, None)


def pre_create_room(model: model.CreateRoom):
    metas = model.get_metas()

    if metas is None:
        metas = {}
        
    # 2 player is more than enough
    model.set_max_user(2)

    # First player is X
    metas["player-1"]    = model.get_user_id()
    metas["player-2"]    = None
    metas["next-player"] = 'X'

    # Copy play board
    metas["board"] = BASE_BOARD.copy()

    model.set_metas(metas)

def post_create_room(model: model.CreateRoom, success: bool):
    if success:
        # Ok, successfully room created
        print("ROOM CREATED")

def post_join_to_room(model: model.JoinToRoom, success: bool):
    if success:
        room_id = model.get_room_id()
        metas   = room.get_room_metas(model.get_room_id())

        if metas is None:
            raise Exception("Something went wrong. Sorry.")
        
        room.set_room_meta(room_id, "player-2", model.get_user_id())

        # Lets find who will start first
        value = random.randint(0, 1)

        if value == 1:
            room.set_room_meta(room_id, "X", room.get_room_meta(room_id, "player-1"))
            room.set_room_meta(room_id, "O", room.get_room_meta(room_id, "player-2"))
        else:
            room.set_room_meta(room_id, "O", room.get_room_meta(room_id, "player-1"))
            room.set_room_meta(room_id, "X", room.get_room_meta(room_id, "player-2"))
        
        room.set_room_meta(room_id, "next-mark", "X")
        room.message_to_room_user(room_id, room.get_room_meta(room_id, "X"), {
            "type": "Start",
            "mark": "X",
            "next-mark": "X"
        })
        room.message_to_room_user(room_id, room.get_room_meta(room_id, "O"), {
            "type": "Start",
            "mark": "O",
            "next-mark": "X"
        })

def pre_room_list_request(model: model.RoomListRequest):
    model.set_members([constants.ROOM_INFO_TYPE_ROOM_NAME, constants.ROOM_INFO_TYPE_USER_LENGTH])

def pre_message_to_room(model: model.MessageToRoom):
    message      = model.get_message()
    message_type = message.get("type")
    metas        = room.get_room_metas(model.get_room_id())
    
    # Is game started?
    if "player-2" not in metas:
        fail("Game not started yet.")
        
    next_player  = metas[metas["next-mark"]]

    if message_type == "play" and next_player != model.get_user_id():
        fail("It is not your turn")

def post_message_to_room(model: model.MessageToRoom, success: bool):
    if success:
        room_id      = model.get_room_id()
        message      = model.get_message()
        message_type = message.get("type")
        metas        = room.get_room_metas(room_id)

        if message_type == "play":
            next_mark = metas["next-mark"]
            slot      = message.get("slot")

            # User can play
            print("Fine to play")
            board = metas.get("board")
            print(board, slot / 3, slot % 3, next_mark)
            (finished, won) = play(board, int(slot / 3), int(slot % 3), next_mark)

            if finished is False:
                new_next_mark = "O" if next_mark == "X" else "X"
                room.set_room_meta(room_id, "next-mark", new_next_mark)

                room.message_to_room_user(room_id, room.get_room_meta(room_id, new_next_mark), {
                    "type": "YourTurn"
                })
            else:
                player_1 = room.get_room_meta(room_id, "player-1")
                player_2 = room.get_room_meta(room_id, "player-2")

                if won == GAME_STATUS_PLAYER_1_WIN:
                    room.message_to_room_user(room_id, player_1, { "type": "Win" })
                    room.message_to_room_user(room_id, player_2, { "type": "Lose" })

                elif won == GAME_STATUS_PLAYER_2_WIN:
                    room.message_to_room_user(room_id, player_2, { "type": "Win" })
                    room.message_to_room_user(room_id, player_1, { "type": "Lose" })

                else:
                    room.message_to_room_user(room_id, player_2, { "type": "Draw" })
                    room.message_to_room_user(room_id, player_1, { "type": "Draw" })
