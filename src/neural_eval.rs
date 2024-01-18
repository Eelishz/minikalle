use shakmaty::{Chess, Position, Color};

const SCALE: [i32; 5] = [
    51,
    43,
    41,
    85,
    497,
];

const L0: usize = 768;
const L1: usize = 512;
const L2: usize = 256;
const L3: usize = 32;
const L4: usize = 32;
const L5: usize = 1;

const W0: [i8; L0*L1] = include!("model/W0.in");
const W1: [i8; L1*L2] = include!("model/W1.in");
const W2: [i8; L2*L3] = include!("model/W2.in");
const W3: [i8; L3*L4] = include!("model/W3.in");
const W4: [i8; L4] = include!("model/W4.in");

const B0: [i8; L1] = include!("model/B0.in");
const B1: [i8; L2] = include!("model/B1.in");
const B2: [i8; L3] = include!("model/B2.in");
const B3: [i8; L4] = include!("model/B3.in");
const B4: [i8; L5] = include!("model/B4.in");

#[inline]
fn dot(x: &[i32], y: &[i8]) -> i32 {
    assert!(!x.is_empty());
    assert_eq!(x.len(), y.len());

    let mut sum = 0;

    for i in 0..x.len() {
        sum += x[i] * y[i] as i32;
    }

    sum
}

#[inline]
fn relu(x: i32) -> i32 {
    x.max(0)
}

fn feed_forward(input: &[i32; L0]) -> i32 {
    // Layer 0

    let mut h0 = [0; L1];
    for i in 0..L1 {
        h0[i] = relu(dot(input, &W0[L0*i..L0*(i+1)]) + B0[i] as i32) / SCALE[0];
    }

    // Layer 1

    let mut h1 = [0; L2];
    for i in 0..L2 {
        h1[i] = relu(dot(&h0, &W1[L1*i..L1*(i+1)]) + B1[i] as i32) / SCALE[1];
    }

    // Layer 2

    let mut h2 = [0; L3];
    for i in 0..L3 {
        h2[i] = relu(dot(&h1, &W2[L2*i..L2*(i+1)]) + B2[i] as i32) / SCALE[2];
    }

    // Layer 3

    let mut h3 = [0; L4];
    for i in 0..L4 {
        h3[i] = relu(dot(&h2, &W3[L3*i..L3*(i+1)]) + B3[i] as i32) / SCALE[3];
    }

    // Output Layer
    let output = (dot(&h3, &W4) + B4[0] as i32) / SCALE[4];

    output
}

fn serialize(position: &Chess) -> [i32; L0] {
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
                result[index + sq] = 100;
            }
            index += 64;
        }
    }

    return result;
}

pub fn predict(position: &Chess) -> i16 {
    let input = serialize(position);
    feed_forward(&input) as i16 * match position.turn() {
        Color::White => 1,
        Color::Black => -1,
    }
}
