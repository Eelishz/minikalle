import chess
import chess.pgn
import numpy as np
import os
from tqdm import tqdm

def state(board):
    arr = []
    arr.extend(board.pieces(1, 1).tolist())
    arr.extend(board.pieces(2, 1).tolist())
    arr.extend(board.pieces(3, 1).tolist())
    arr.extend(board.pieces(4, 1).tolist())
    arr.extend(board.pieces(5, 1).tolist())
    arr.extend(board.pieces(6, 1).tolist())

    arr.extend(board.pieces(1, 0).tolist())
    arr.extend(board.pieces(2, 0).tolist())
    arr.extend(board.pieces(3, 0).tolist())
    arr.extend(board.pieces(4, 0).tolist())
    arr.extend(board.pieces(5, 0).tolist())
    arr.extend(board.pieces(6, 0).tolist())

    arr.append(board.turn)
    arr.append(board.fullmove_number)

    return np.asarray(arr, dtype=np.int16)

def has_captures(board):
    legal_moves = board.legal_moves
    for m in legal_moves:
        if board.is_capture(m):
            return True
    return False

def get_dataset(num_samples=None):
    X = np.zeros((num_samples, 770), dtype=np.int16)
    y = np.zeros((num_samples), dtype=np.int16)
    gn = 0
    sn = 0
    values = {'1/2-1/2':0, '0-1':-1, '1-0':1}

    pbar = tqdm(total=num_samples)

    # pgn files in the data folder
    for fn in os.listdir('data'):
        pgn = open(os.path.join('data', fn))
        while 1:
            game = chess.pgn.read_game(pgn)
            if game is None:
                break
            res = game.headers['Result']
            if res not in values:
                continue
            value = values[res]
            board = game.board()
            for move in game.mainline_moves():
                # if has_captures(board):
                #     continue

                if num_samples is not None and sn > num_samples:
                    return X, y

                board.push(move)
                ser = state(board)
                X[gn, :] = ser
                y[gn] = value
                pbar.update(1)
                sn += 1                

            gn += 1

            del game
            del board
    return X, y

if __name__ == "__main__":
    X, y = get_dataset(15_000_000)
    np.savez('processed/dataset_15M.npz', X, y)
