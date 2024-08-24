use shakmaty::{Chess, Color, Position};
use std::simd::{i16x8, num::SimdInt};

const LANES: usize = 8;
const FIXED_POINT: i16 = 32;

const L0: usize = 768;
const L1: usize = 32;
const L2: usize = 16;
const L3: usize = 8;
const L4: usize = 8;
const L5: usize = 1;

const W0: [i16; L0 * L1] = include!("model/W0.in");
const W1: [i16; L1 * L2] = include!("model/W1.in");
const W2: [i16; L2 * L3] = include!("model/W2.in");
const W3: [i16; L3 * L4] = include!("model/W3.in");
const W4: [i16; L4] = include!("model/W4.in");

const B0: [i16; L1] = include!("model/B0.in");
const B1: [i16; L2] = include!("model/B1.in");
const B2: [i16; L3] = include!("model/B2.in");
const B3: [i16; L4] = include!("model/B3.in");
const B4: [i16; L5] = include!("model/B4.in");

macro_rules! apply_layer {
    ($h0:expr, $h1:expr, $l0:expr, $l1:expr, $b:expr, $w:expr) => {
        let fixed_point = i16x8::splat(FIXED_POINT);

        for e in $h1.iter_mut() {
            let mut zs = [0; $l1 / LANES];
            for i in 0..($l1 / LANES) {
                let a = i16x8::from_slice(&$h0[i * LANES..i * LANES + LANES]);
                let b = i16x8::from_slice(&$b[i * LANES..i * LANES + LANES]);
                let w = i16x8::from_slice(&$w[i * LANES..i * LANES + LANES]);
                zs[i] = (a * w / fixed_point + b).reduce_sum();
            }
            *e = zs.iter().map(|x| x.max(&0)).sum();
        }
    };
}

fn feed_forward(input: &[i16; L0]) -> i16 {
    // Layer 0

    let mut h0 = [0; L1];
    apply_layer!(input, h0, L0, L1, B0, W0);

    // Layer 1

    let mut h1 = [0; L2];
    apply_layer!(h0, h1, L1, L2, B1, W1);

    // Layer 2

    let mut h2 = [0; L3];
    apply_layer!(h1, h2, L2, L3, B2, W2);

    // Layer 3

    let mut h3 = [0; L4];
    apply_layer!(h2, h3, L3, L3, B3, W3);

    // Output Layer

    let fixed_point = i16x8::splat(FIXED_POINT);

    let mut zs = [0; L5 / LANES];
    for i in 0..(L5 / LANES) {
        let a = i16x8::from_slice(&h3[i * LANES..i * LANES + LANES]);
        let b = i16x8::from_slice(&B4[i * LANES..i * LANES + LANES]);
        let w = i16x8::from_slice(&W4[i * LANES..i * LANES + LANES]);
        zs[i] = (a * w / fixed_point + b).reduce_sum();
    }

    zs.iter().map(|x| x.max(&0)).sum()
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
                result[index + sq] = FIXED_POINT;
            }
            index += 64;
        }
    }

    result
}

pub fn predict(position: &Chess) -> i16 {
    let input = serialize(position);
    feed_forward(&input)
        * match position.turn() {
            Color::White => 1,
            Color::Black => -1,
        }
}
