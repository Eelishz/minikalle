use shakmaty::{Bitboard, Board, Color, Role};

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, Chess, Position};

    #[test]
    fn test_square_table() {
        let position = Chess::new();

        let table = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10, 10,
        ];
        let evaluation = parse_table(
            &position.board().by_color(Color::White).flip_vertical(),
            table,
        );

        assert_eq!(evaluation, 160);

        let evaluation = parse_table(&position.board().by_color(Color::Black), table);

        assert_eq!(evaluation, 160);
    }

    #[test]
    fn test_square_tables() {
        let position = Chess::new();

        let evaluation = parse_tables(&position.board());

        assert_eq!(evaluation, 0);

        let fen: Fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let evaluation = parse_tables(&position.board());

        assert!(evaluation > 0); // White has better position: eval > 0

        let fen: Fen = "rnbqkbnr/ppp2ppp/8/3pp3/4P3/1P6/P1PP1PPP/RNBQKBNR w KQkq - 0 3"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let evaluation = parse_tables(&position.board());

        assert!(evaluation < 0); // black has better position < 0
    }

    //#[test]
    //fn test_prase_function() {
    //    let mut position = Chess::new();
    //
    //    for _ in 1..100 {
    //        let board = position.board();
    //        let new_eval = parse_tables(board);
    //        let old_eval = _parse_tables(board);
    //
    //        assert_eq!(new_eval, old_eval);
    //        position = position.clone().play(&position.legal_moves()[0]).unwrap();
    //    }
    //}
}

const PAWN_SQUARE_TABLE: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];

const KNIGHT_SQUARE_TABLE: [i16; 64] = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];

const BISHOP_SQUARE_TABLE: [i16; 64] = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];

const ROOK_SQUARE_TABLE: [i16; 64] = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];

const QUEEN_SQUARE_TABLE: [i16; 64] = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];

const KING_SQUARE_TABLE: [i16; 64] = [
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6,
    22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14,
    -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24,
    14,
];

const SQUARE_TABLES: [[i16; 64]; 6] = [
    PAWN_SQUARE_TABLE,
    KNIGHT_SQUARE_TABLE,
    BISHOP_SQUARE_TABLE,
    ROOK_SQUARE_TABLE,
    QUEEN_SQUARE_TABLE,
    KING_SQUARE_TABLE,
];

#[inline]
fn parse_table(bitboard: &Bitboard, square_table: [i16; 64]) -> i16 {
    bitboard
        .into_iter()
        .map(|square| square_table[square as usize])
        .sum()
}

pub fn parse_tables(board: &Board) -> i16 {
    Color::ALL
        .iter()
        .map(|color| {
            let color_mask = board.by_color(*color);
            Role::ALL
                .iter()
                .enumerate()
                .map(|(i, role)| {
                    let evaluation_table = SQUARE_TABLES[i];
                    let combined_bitboard = board.by_role(*role).intersect(color_mask);
                    match color {
                        Color::White => {
                            parse_table(&combined_bitboard.flip_vertical(), evaluation_table)
                        }
                        Color::Black => -parse_table(&combined_bitboard, evaluation_table),
                    }
                })
                .sum::<i16>()
        })
        .sum::<i16>()
}
