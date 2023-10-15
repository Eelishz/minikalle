use shakmaty::{board, Board, Chess, Color, Outcome, Position, Role};

const POSITIVE_INFINITY: i32 = 9999999;
const NEGATIVE_INFINITY: i32 = -9999999;

const PAWN_SQUARE_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    50, 50, 50, 50, 50, 50, 50, 50, // rank 7
    10, 10, 20, 30, 30, 20, 10, 10, // rank 6
    5, 5, 10, 25, 25, 10, 5, 5, // rank 5
    0, 0, 0, 50, 999, 0, 0, 0, // rank 4
    5, -5, -10, 0, 0, -10, -5, 5, // rank 3
    5, 10, 10, -50, -50, 10, 10, 5, // rank 2
    0, 0, 0, 0, 0, 0, 0, 0, // rank 1
];

const KNIGHT_SQUARE_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, // rank 8
    -40, -20, 0, 0, 0, 0, -20, -40, // rank 7
    -30, 0, 10, 15, 15, 10, 0, -30, // rank 6
    -30, 5, 15, 20, 20, 15, 5, -30, // rank 5
    -30, 0, 15, 20, 20, 15, 0, -30, // rank 4
    -30, 5, 10, 15, 15, 10, 5, -30, // rank 3
    -40, -20, 0, 5, 5, 0, -20, -40, // rank 2
    -50, -40, -30, -30, -30, -30, -40, -50, // rank 1
];

const BISHOP_SQUARE_TABLE: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, // rank 8
    -10, 0, 0, 0, 0, 0, 0, -10, // rank 7
    -10, 0, 5, 10, 10, 5, 0, -10, // rank 6
    -10, 5, 5, 10, 10, 5, 5, -10, // rank 5
    -10, 0, 10, 10, 10, 10, 0, -10, // rank 4
    -10, 10, 10, 10, 10, 10, 10, -10, // rank 3
    -10, 5, 0, 0, 0, 0, 5, -10, // rank 2
    -20, -10, -10, -10, -10, -10, -10, -20, // rank 1
];

const ROOK_SQUARE_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    5, 10, 10, 10, 10, 10, 10, 5, // rank 7
    -5, 0, 0, 0, 0, 0, 0, -5, // rank 6
    -5, 0, 0, 0, 0, 0, 0, -5, // rank 5
    -5, 0, 0, 0, 0, 0, 0, -5, // rank 4
    -5, 0, 0, 0, 0, 0, 0, -5, // rank 3
    -5, 0, 0, 0, 0, 0, 0, -5, // rank 2
    0, 0, 0, 5, 5, 0, 0, 0, // rank 1
];

const QUEEN_SQUARE_TABLE: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

const KING_SQUARE_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

const SQUARE_TABLES: [[i32; 64]; 6] = [
    PAWN_SQUARE_TABLE,
    KNIGHT_SQUARE_TABLE,
    BISHOP_SQUARE_TABLE,
    ROOK_SQUARE_TABLE,
    QUEEN_SQUARE_TABLE,
    KING_SQUARE_TABLE,
];

fn parse_square_tables(board: &Board) -> i32 {
    Color::ALL
        .iter()
        .zip(Role::ALL.iter().enumerate())
        .fold(0, |acc, (color, (i, role))| {
            let color_bitboard = board.by_color(*color);
            let role_bitboard = board.by_role(*role);
            let role_color_bitboard = role_bitboard.intersect(color_bitboard);
            let evaluation_table = SQUARE_TABLES[i];

            acc + match color {
                Color::White => role_color_bitboard
                    .flip_vertical()
                    .into_iter()
                    .fold(0, |inner_acc, square| {
                        inner_acc + evaluation_table[square as usize]
                    }),
                Color::Black => role_color_bitboard
                    .into_iter()
                    .fold(0, |inner_acc, square| {
                        inner_acc - evaluation_table[square as usize]
                    }),
            }
        })
}

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

pub fn evaluate(position: &Chess, depth_from_root: u8) -> i32 {
    let evaluation = match position.outcome() {
        Some(Outcome::Draw) => return 0,
        Some(Outcome::Decisive { winner }) => {
            if winner == Color::White {
                POSITIVE_INFINITY - depth_from_root as i32
            } else {
                NEGATIVE_INFINITY + depth_from_root as i32
            }
        }
        None => count_pieces(position.board()),
    };
    if position.turn() == Color::Black {
        -evaluation
    } else {
        evaluation
    }
}
