import chess
import chess.pgn
import numpy as np
import os

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

    return arr

def has_captures(board):
    legal_moves = board.legal_moves
    for m in legal_moves:
        if board.is_capture(m):
            return True
    return False

def get_dataset(num_samples=None):
    X,y = [], []
    gn = 0
    values = {'1/2-1/2':0, '0-1':-1, '1-0':1}
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
                if has_captures(board):
                    continue

                board.push(move)
                ser = state(board)
                X.append(ser)
                y.append(value)
            print(f'parsing game {gn}, got {len(X)} examples')
            if num_samples is not None and len(X) > num_samples:
                X = np.array(X)
                y = np.array(y)
                return X, y
            gn += 1

            del game
            del board
    X = np.array(X)
    y = np.array(y)
    return X, y

if __name__ == "__main__":
    X, y = get_dataset(4_000_000)
    np.savez('processed/dataset_B_4M_no_cap.npz', X, y)
