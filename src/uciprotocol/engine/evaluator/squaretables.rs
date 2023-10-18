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
        let evaluation = parse_table(&position.board().by_color(Color::White).flip_vertical(), table);

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

const PAWN_SQUARE_TABLE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, // rank 8
    50, 50, 50, 50, 50, 50, 50, 50, // rank 7
    10, 10, 20, 30, 30, 20, 10, 10, // rank 6
    5, 5, 10, 25, 25, 10, 5, 5, // rank 5
    0, 0, 0, 50, 50, 0, 0, 0, // rank 4
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

fn parse_table(bitboard: &Bitboard, square_table: [i32; 64]) -> i32 {
    bitboard
        .into_iter()
        .map(|square| square_table[square as usize])
        .sum()
}

pub fn parse_tables(board: &Board) -> i32 {
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
                        Color::White => parse_table(&combined_bitboard.flip_vertical(), evaluation_table),
                        Color::Black => {
                            -parse_table(&combined_bitboard, evaluation_table)
                        }
                    }
                })
                .sum::<i32>()
        })
        .sum::<i32>()
}
