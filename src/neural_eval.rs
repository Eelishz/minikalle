use shakmaty::{Chess, Position, Color};

const SCALE: i16 = 128;

const L0: usize = 770;
const L1: usize = 20;
const L2: usize = 1;

const W0: [i16; L0*L1] = include!("model/W0.in");
const W1: [i16; L1] = include!("model/W1.in");

const B0: [i16; L1] = include!("model/B0.in");
const B1: [i16; L2] = include!("model/B1.in");

#[inline]
fn dot(x: &[i16], y: &[i16]) -> i16 {
    assert!(!x.is_empty());
    assert_eq!(x.len(), y.len());

    let mut sum = 0;

    for i in 0..x.len() {
        sum += (x[i] * y[i]) / SCALE;
    }

    sum
}

#[inline]
fn relu(x: i16) -> i16 {
    x.max(0)
}

fn feed_forward(input: &[i16; L0]) -> i16 {
    let mut h0 = [0; L1];
    for i in 0..L1 {
        h0[i] = relu(dot(input, &W0[L0*i..L0*(i+1)]) + B0[i]);
    }

    let output = dot(&h0, &W1) + B1[0];

    output
}

fn serialize(position: &Chess) -> [i16; L0] {
    let board = position.board();

    let mut result = [0; L0];
    let mut index = 0;

    let white = board.white();
    let black = board.black();

    let p = board.pawns();
    let n = board.knights();
    let b = board.bishops();
    let r = board.rooks();
    let q = board.queens();
    let k = board.kings();

    for color in [white, black] {
        for piece in [p, n, b, r, q, k] {
            let bb = color.intersect(piece);

            for sq in bb {
                let sq = sq as usize;
                result[index + sq] = SCALE;
            }
            index += 64;
        }
    }

    result[768] = match position.turn() {
        Color::Black => 0,
        Color::White => 1,
    };
    result[769] = u32::from(position.fullmoves()) as i16;

    return result;
}

pub fn predict(position: &Chess) -> i16 {
    let input = serialize(position);
    feed_forward(&input) * match position.turn() {
        Color::White => 1,
        Color::Black => -1,
    }
}
