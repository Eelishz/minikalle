use shakmaty::{Board, Chess, Color, Outcome, Position};
mod squaretables;
use squaretables::parse_tables;

const POSITIVE_INFINITY: i32 = 100000;
const NEGATIVE_INFINITY: i32 = -100000;

fn count_pieces(board: &Board) -> i32 {
    let white_material = board.material_side(Color::White);
    let black_material = board.material_side(Color::Black);

    white_material.pawn as i32 * 100
        + white_material.knight as i32 * 300
        + white_material.bishop as i32 * 300
        + white_material.rook as i32 * 500
        + white_material.queen as i32 * 900
        - black_material.pawn as i32 * 100
        - black_material.knight as i32 * 300
        - black_material.bishop as i32 * 300
        - black_material.rook as i32 * 500
        - black_material.queen as i32 * 900
}

pub fn evaluate(position: &Chess) -> i32 {
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
    if position.is_stalemate() {
        return 0;
    }
    match position.turn() {
        Color::Black => -evaluation,
        Color::White => evaluation,
    }
}
