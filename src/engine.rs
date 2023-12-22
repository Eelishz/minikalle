use crate::evaluation::evaluate;
use crate::openings::OPENINGS;
use crate::transpositiontable::{EvaluationType, TranspositionTable};
use rand::seq::SliceRandom;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{uci::Uci, CastlingMode, Chess, Move, Position};
use shakmaty::{MoveList, Role, Square};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime};

pub const POSITIVE_INFINITY: i16 = i16::MAX - 1;
pub const NEGATIVE_INFINITY: i16 = i16::MIN + 1;

const NULL_MOVE: Move = Move::Normal {
    role: Role::Pawn,
    from: Square::A1,
    capture: None,
    to: Square::A1,
    promotion: None,
};

pub struct Engine {
    tt: TranspositionTable,
    book: HashMap<u64, Vec<String>>,
}

impl Engine {
    pub fn new() -> Engine {
        let book = serde_json::from_str(&OPENINGS).unwrap();
        Engine {
            tt: TranspositionTable::new(512),
            book,
        }
    }

    pub fn new_game(&mut self) {}

    pub fn set_hash(&mut self, value: usize) {
        self.tt = TranspositionTable::new(value);
    }

    fn iterative_deepening(
        &mut self,
        position: &mut Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, i16) {
        let start_time = SystemTime::now();

        let mut best_move = position.legal_moves()[0].clone();
        let mut best_evaluation = NEGATIVE_INFINITY;
        let mut nodes_searched = 0;

        let mut depth: u8 = 1;

        while depth < max_depth {
            let search = root_search(position, depth, &mut self.tt, max_time, &start_time);

            match search {
                Some(s) => {
                    best_move = s.clone().0;
                    best_evaluation = s.1;
                    nodes_searched += s.2;
                }
                None => break,
            }

            let nps =
                nodes_searched / (start_time.elapsed().unwrap().as_millis() as u64 + 1) * 1000;
            println!("info nodes {0} nps {nps} depth {depth}", nodes_searched);
            println!("info score cp {}", best_evaluation);
            match best_evaluation {
                POSITIVE_INFINITY => {
                    println!("info score mate {depth}");
                    break;
                }
                NEGATIVE_INFINITY => {
                    println!("info score mate -{depth}");
                    break;
                }
                _ => println!("info score cp {}", best_evaluation),
            }
            depth += 1;
        }

        return (best_move, best_evaluation);
    }

    pub fn find_best_move(
        &mut self,
        position: &Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, Uci, i16) {
        let zobrist = position.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal);
        if self.book.contains_key(&zobrist.0) {
            let moves = self.book.get(&zobrist.0).unwrap();
            let move_string = moves.choose(&mut rand::thread_rng()).unwrap();
            let uci = Uci::from_str(move_string).unwrap();
            let chess_move = uci.to_move(position).unwrap();
            return (chess_move, uci, 0);
        }
        let mut position = position.clone();
        let (best_move, evaluation) = self.iterative_deepening(&mut position, max_time, max_depth);
        (
            best_move.clone(),
            best_move.clone().to_uci(CastlingMode::Standard),
            evaluation,
        )
    }
}

impl Default for Engine {
    fn default() -> Engine {
        Engine::new()
    }
}

#[inline]
fn sort_moves(scores: &mut [i16], moves: &mut MoveList) {
    for i in 1..scores.len() {
        let mut j = i;
        while j > 0 && scores[j - 1] > scores[j] {
            unsafe {
                scores.swap_unchecked(j, j - 1);
                moves.swap_unchecked(j, j - 1);
            }
            j -= 1;
        }
    }
}

#[inline]
fn order_moves(position: &Chess, tt: &TranspositionTable, capture_moves: bool) -> MoveList {
    // MVV-LVA (most valuable capture, least valuable attacker)
    // Hash move
    let mut legal_moves = match capture_moves {
        false => position.legal_moves(),
        true => position.capture_moves(),
    };

    let mut hash_move = NULL_MOVE;

    if !capture_moves {
        hash_move = match tt.get(
            &position
                .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
                .0,
        ) {
            Some(transposition) => transposition.best_move,
            None => NULL_MOVE,
        };
    }

    let mut scores = [0; 256]; // 256 should be large enough

    for (i, chess_move) in legal_moves.iter().enumerate() {
        if !capture_moves && (chess_move == &hash_move) {
            scores[i] = -9999;
        } else if chess_move.is_capture() {
            let attacker = match chess_move.role() {
                Role::Pawn => 100,
                Role::Knight => 300,
                Role::Bishop => 300,
                Role::Rook => 500,
                Role::Queen => 900,
                Role::King => 2000,
            };
            let victim = match chess_move.capture().unwrap() {
                Role::Pawn => 100,
                Role::Knight => 300,
                Role::Bishop => 300,
                Role::Rook => 500,
                Role::Queen => 900,
                Role::King => 2000,
            };

            scores[i] = -(victim - attacker);
        }
    }

    let score_slice = &mut scores[0..legal_moves.len()];
    sort_moves(score_slice, &mut legal_moves);

    return legal_moves;
}

fn quiesce(
    position: &Chess,
    mut alpha: i16,
    beta: i16,
    depth_from_root: u8,
    tt: &TranspositionTable,
    mut nodes_searched: u64,
    max_time: u64,
    start_time: &SystemTime,
) -> Option<(i16, u64)> {
    if start_time.elapsed().unwrap().as_millis() as u64 >= max_time {
        return None;
    }

    nodes_searched += 1;

    let stand_pat = evaluate(&position);

    if stand_pat >= beta {
        return Some((beta, nodes_searched));
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let moves = order_moves(position, tt, true);

    for chess_move in moves {
        let mut new_position = position.clone();
        new_position.play_unchecked(&chess_move);
        let (evaluation, new_searched) = quiesce(
            &new_position,
            -beta,
            -alpha,
            depth_from_root,
            tt,
            nodes_searched,
            max_time,
            start_time,
        )?;
        nodes_searched = new_searched;
        let evaluation = -evaluation;

        if evaluation >= beta {
            return Some((beta, nodes_searched));
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
    }

    return Some((alpha, nodes_searched));
}

#[inline]
fn is_passed_pawn(_position: &Chess) -> bool {
    // TODO
    // This function should get optimized away,
    // but I have left this here as a reminder to myself.
    false
}

#[inline]
fn calculate_extension(m: &Move, position: &Chess, depth_left: u8) -> u8 {
    if depth_left >= 3 {
        return 0;
    }
    if m.is_capture() {
        return 1;
    }
    if m.is_promotion() {
        return 1;
    }
    if position.is_check() {
        return 1;
    }
    if is_passed_pawn(position) {
        return 1;
    }

    return 0;
}

fn alpha_beta(
    position: &Chess,
    mut alpha: i16,
    beta: i16,
    depth_left: u8,
    depth_from_root: u8,
    tt: &mut TranspositionTable,
    mut nodes_searched: u64,
    max_time: u64,
    start_time: &SystemTime,
) -> Option<(i16, u64)> {
    if start_time.elapsed().unwrap().as_millis() as u64 >= max_time {
        return None;
    }

    nodes_searched += 1;

    let zobrist = position
        .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
        .0;

    let table_lookup = tt.probe_table(&zobrist, depth_left, alpha, beta);
    if table_lookup.is_some() {
        let mut new_position = position.clone();
        let chess_move = table_lookup.clone().unwrap().0;
        new_position.play_unchecked(&chess_move);
        return Some((table_lookup.unwrap().1, nodes_searched));
    }

    if depth_left == 0 {
        let (evaluation, new_seached) = quiesce(
            position,
            alpha,
            beta,
            depth_from_root + 1,
            tt,
            nodes_searched,
            max_time,
            start_time,
        )?;
        nodes_searched = new_seached;

        tt.insert(
            zobrist,
            NULL_MOVE,
            evaluation,
            depth_left,
            EvaluationType::Exact,
        );
        return Some((evaluation, nodes_searched));
    }

    let null_move_possible = !position.is_check();

    if null_move_possible && depth_left >= 3 {
        let r = 3;
        let new_position = position.clone();
        let new_position = new_position.swap_turn().unwrap();

        let (evaluation, new_searched) = alpha_beta(
            &new_position,
            -beta,
            1 - beta,
            (depth_left as i8 - r - 1).max(0) as u8,
            depth_from_root + 1,
            tt,
            nodes_searched,
            max_time,
            start_time,
        )?;

        let evaluation = -evaluation;
        nodes_searched = new_searched;

        if evaluation >= beta {
            return Some((beta, nodes_searched));
        }
    }

    let moves = order_moves(&position, &tt, false);

    if moves.len() == 0 {
        return Some((0, nodes_searched));
    }

    let mut best_move = NULL_MOVE;

    for chess_move in moves {
        let mut new_position = position.clone();
        new_position.play_unchecked(&chess_move);

        let extension = calculate_extension(&chess_move, position, depth_left);

        let (evaluation, new_searched) = alpha_beta(
            &new_position,
            -beta,
            -alpha,
            depth_left + extension - 1,
            depth_from_root + 1,
            tt,
            nodes_searched,
            max_time,
            start_time,
        )?;
        nodes_searched = new_searched;
        let evaluation = -evaluation;
        if evaluation >= beta {
            tt.insert(
                zobrist,
                chess_move.clone(),
                beta,
                depth_left,
                EvaluationType::Beta,
            );
            return Some((beta, nodes_searched));
        }
        if evaluation > alpha {
            alpha = evaluation;
            best_move = chess_move;
        }
    }

    tt.insert(zobrist, best_move, alpha, depth_left, EvaluationType::Alpha);
    return Some((alpha, nodes_searched));
}

pub fn root_search(
    position: &Chess,
    depth_left: u8,
    tt: &mut TranspositionTable,
    max_time: u64,
    start_time: &SystemTime,
) -> Option<(Move, i16, u64)> {
    let mut nodes_searched = 1;
    let zobrist = position
        .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
        .0;

    let mut alpha = NEGATIVE_INFINITY;
    let beta = POSITIVE_INFINITY;

    let moves = order_moves(&position, tt, false);

    let mut best_move = moves[0].clone();

    for chess_move in moves {
        let mut new_position = position.clone();
        new_position.play_unchecked(&chess_move);
        let (evaluation, new_searched) = alpha_beta(
            &new_position,
            -beta,
            -alpha,
            depth_left - 1,
            1,
            tt,
            nodes_searched,
            max_time,
            &start_time,
        )?;
        nodes_searched = new_searched;
        let evaluation = -evaluation;

        if evaluation >= beta {
            tt.insert(
                zobrist,
                chess_move.clone(),
                beta,
                depth_left,
                EvaluationType::Beta,
            );
            return Some((chess_move, beta, nodes_searched));
        }
        if evaluation > alpha {
            alpha = evaluation;
            best_move = chess_move;
        }
    }

    tt.insert(
        zobrist,
        best_move.clone(),
        alpha,
        depth_left,
        EvaluationType::Alpha,
    );
    return Some((best_move, alpha, nodes_searched));
}

extern crate test;

#[cfg(test)]
mod tests {
    use shakmaty::{fen::Fen, Chess};
    use test::Bencher;

    use super::*;

    #[test]
    fn test_alpha_beta() {
        // Create a test position
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut tt = TranspositionTable::new(64);

        // Call your alpha-beta function
        let (evaluation, _) = alpha_beta(
            &position,
            NEGATIVE_INFINITY,
            POSITIVE_INFINITY,
            3,
            0,
            &mut tt,
            0,
            1000,
            &SystemTime::now(),
        )
        .unwrap();

        // Assert that the result is as expected
        assert!(evaluation >= 0);
    }

    #[test]
    fn test_root_search() {
        // Create a test position
        let mut position = Chess::new(); // You may want to set up a specific test position here
        let mut tt = TranspositionTable::new(64);

        // Call your alpha-beta function
        let (_, evaluation, _) =
            root_search(&mut position, 3, &mut tt, 1000, &SystemTime::now()).unwrap();

        // Assert that the result is as expected
        assert!(evaluation >= 0);
    }

    #[test]
    fn test_mates() {
        // Some easy checkmates
        let mut engine = Engine::new();

        let fen: Fen = "6k1/2R5/8/8/8/3R4/2K5/8 w - - 0 1".parse().unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 3);

        assert_eq!(uci.to_string(), "d3d8".to_string());

        let fen: Fen = "6k1/2p4p/2p4b/p7/3P1p2/2P2P2/PP2b1KP/4q3 b - - 9 35"
            .parse()
            .unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 3);

        assert_eq!(uci.to_string(), "e1f1".to_string());

        let fen: Fen = "6k1/2p4p/b1p1q2b/p7/3P1pp1/2P2P2/PP4PP/4B1K1 b - - 1 29"
            .parse()
            .unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 3);

        assert_eq!(uci.to_string(), "e6e1".to_string());

        let fen: Fen = "4r1k1/ppp2ppp/5n2/6P1/1PP5/2b4P/r7/5K1R b - - 0 33"
            .parse()
            .unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 3);

        assert_eq!(uci.to_string(), "e8e1".to_string());
    }

    #[test]
    fn test_captures() {
        let fen: Fen = "7k/8/8/4p3/3Q4/8/8/K7 w - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let mut engine = Engine::new();

        let (_, uci, _) = engine.find_best_move(&position, 1_000, 40);

        assert_eq!(uci.to_string(), "d4e5".to_string());

        let fen: Fen = "7k/8/8/4q3/3Q4/8/8/K7 b - - 0 1".parse().unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 40);

        assert_eq!(uci.to_string(), "e5d4".to_string());
    }

    #[test]
    fn test_move_ordering() {
        let position = Chess::new();
        let tt = TranspositionTable::new(64);

        let result = order_moves(&position, &tt, false);

        assert_eq!(result.len(), position.legal_moves().len());

        let result = order_moves(&position, &tt, true);

        assert_eq!(result.len(), position.capture_moves().len());
    }

    #[bench]
    fn bench_search(b: &mut Bencher) {
        let position = Chess::new();
        let mut tt = TranspositionTable::new(64);

        let alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;

        b.iter(|| {
            alpha_beta(
                &position.clone(),
                alpha,
                beta,
                3,
                0,
                &mut tt,
                0,
                1000,
                &SystemTime::now(),
            )
        })
    }
}
