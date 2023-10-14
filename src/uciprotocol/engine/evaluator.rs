use shakmaty::{Chess, Color, Outcome, Position};

const POSITIVE_INFINITY: i32 = 999999999;
const NEGATIVE_INFINITY: i32 = -999999999;

pub struct Evaluator {

}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {  }
    }

    fn count_pieces(&self, position: Chess) -> i32 {
        let white_material = position.board().material_side(Color::White);
        let black_material = position.board().material_side(Color::Black);

        white_material.pawn as i32 * 100
            + white_material.knight as i32 * 350
            + white_material.bishop as i32 * 300
            + white_material.rook as i32 * 500
            + white_material.queen as i32 * 900
            - black_material.pawn as i32 * 100
            - black_material.knight as i32 * 350
            - black_material.bishop as i32 * 300
            - black_material.rook as i32 * 500
            - black_material.queen as i32 * 900
    }

    pub fn evaluate(&self, position: Chess, depth_from_root: u8) -> i32 {
        let evaluation = match position.outcome() {
            Some(Outcome::Draw) => return 0,
            Some(Outcome::Decisive { winner }) => {
                if winner == Color::White {
                    POSITIVE_INFINITY - depth_from_root as i32
                } else {
                    NEGATIVE_INFINITY + depth_from_root as i32
                }
            }
            None => self.count_pieces(position.clone()),
        };
        if position.turn() == Color::Black {
            -evaluation
        } else {
            evaluation
        }
    }
}
