use shakmaty::{Board, Color, Role};

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
    -20, -10, -10, -5, -5, -10, -10, -20, // rank 8
    -10, 0, 0, 0, 0, 0, 0, -10, // rank 7
    -10, 0, 5, 5, 5, 5, 0, -10, // rank 6
    -5, 0, 5, 5, 5, 5, 0, -5, // rank 5
    0, 0, 5, 5, 5, 5, 0, -5, // rank 4
    -10, 5, 5, 5, 5, 5, 0, -10, // rank 3
    -10, 0, 5, 0, 0, 0, 0, -10, // rank 2
    -20, -10, -10, -5, -5, -10, -10, -20, // rank 1
];

const KING_SQUARE_TABLE: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, // rank 8
    -30, -40, -40, -50, -50, -40, -40, -30, // rank 7
    -30, -40, -40, -50, -50, -40, -40, -30, // rank 6
    -30, -40, -40, -50, -50, -40, -40, -30, // rank 5
    -20, -30, -30, -40, -40, -30, -30, -20, // rank 4
    -10, -20, -20, -20, -20, -20, -20, -10, // rank 3
    20, 20, 0, 0, 0, 0, 20, 20, // rank 2
    20, 30, 10, 0, 0, 10, 30, 20, // rank 1
];

const SQUARE_TABLES: [[i32; 64]; 6] = [
    PAWN_SQUARE_TABLE,
    KNIGHT_SQUARE_TABLE,
    BISHOP_SQUARE_TABLE,
    ROOK_SQUARE_TABLE,
    QUEEN_SQUARE_TABLE,
    KING_SQUARE_TABLE,
];

pub fn parse_tables(board: &Board) -> i32 {
    Role::ALL
        .into_iter()
        .enumerate()
        .flat_map(|(i, role)| {
            let role_mask = board.by_role(role);
            let table = SQUARE_TABLES[i];

            Color::ALL.iter().map(move |color| {
                let color_mask = board.by_color(*color);
                let mask = color_mask.intersect(role_mask);

                match color {
                    Color::White => mask
                        .into_iter()
                        .zip(table)
                        .map(|(square, x)| (square as i32) * x)
                        .sum::<i32>(),
                    Color::Black => -mask
                        .flip_vertical()
                        .into_iter()
                        .zip(table)
                        .map(|(square, x)| (square as i32) * x)
                        .sum::<i32>(),
                }
            })
        })
        .sum()
}
