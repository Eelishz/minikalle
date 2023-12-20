use shakmaty::{Board, Chess, Color, Outcome, Position};
mod squaretables;
use squaretables::parse_tables;
use super::{POSITIVE_INFINITY, NEGATIVE_INFINITY};

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
    let evaluation = match position.outcome() {
        Some(Outcome::Draw) => return 0,
        Some(Outcome::Decisive { winner }) => {
            if winner == Color::White {
                POSITIVE_INFINITY
            } else {
                NEGATIVE_INFINITY
            }
        }
        None => count_pieces(position.board()) + parse_tables(position.board()),
    };
    match position.turn() {
        Color::Black => -evaluation,
        Color::White => evaluation,
    }
}
