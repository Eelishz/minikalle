use crate::search::{NEG_INF, POS_INF};
use shakmaty::{Board, Chess, Color, Outcome, Position};

#[inline]
fn count_pieces(board: &Board) -> i16 {
    let white_material = board.material_side(Color::White);
    let black_material = board.material_side(Color::Black);

    white_material.pawn as i16 * 100
        + white_material.knight as i16 * 300
        + white_material.bishop as i16 * 300
        + white_material.rook as i16 * 500
        + white_material.queen as i16 * 900
        - black_material.pawn as i16 * 100
        - black_material.knight as i16 * 300
        - black_material.bishop as i16 * 300
        - black_material.rook as i16 * 500
        - black_material.queen as i16 * 900
}

pub fn evaluate(position: &Chess) -> i16 {
    let side = position.turn();
    let board = position.board();
    let evaluation = match position.outcome() {
        Some(Outcome::Draw) => return 0,
        Some(Outcome::Decisive { winner }) => {
            if winner == Color::White {
                POS_INF
            } else {
                NEG_INF
            }
        }
        None => count_pieces(board),
    };
    match side {
        Color::Black => -evaluation,
        Color::White => evaluation,
    }
}
