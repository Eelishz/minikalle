use rand::seq::SliceRandom;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{uci::Uci, CastlingMode, Chess, Move, Position};
use shakmaty::{Role, Square};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{collections::HashMap, time::Instant};
mod evaluator;
mod openings;
mod transpositiontable;
use evaluator::evaluate;
use openings::OPENINGS;
use transpositiontable::{EvaluationType, TranspositionTable};

const POSITIVE_INFINITY: i16 = i16::MAX - 1;
const NEGATIVE_INFINITY: i16 = i16::MIN + 1;

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
    nodes_searched: u64,
}

impl Engine {
    pub fn new() -> Engine {
        let book = serde_json::from_str(&OPENINGS).unwrap();
        Engine {
            tt: TranspositionTable::new(64),
            book,
            nodes_searched: 0,
        }
    }

    fn iterative_deepening(
        &mut self,
        position: &mut Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, i16) {
        let start_time = Instant::now();

        let mut best_move = position.legal_moves()[0].clone();
        let mut best_evaluation = NEGATIVE_INFINITY;

        let mut depth: u8 = 1;

        self.nodes_searched = 0;

        while depth < max_depth {
            let search = root_search(position, depth, &mut self.tt, max_time);

            match search {
                Some(s) => {
                    best_move = s.clone().0;
                    best_evaluation = s.1;
                }
                None => break,
            }

            let nps = self.nodes_searched / (start_time.elapsed().as_millis() as u64 + 1) * 1000;
            println!(
                "info nodes {0} nps {nps} depth {depth}",
                self.nodes_searched
            );
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

        let nps = self.nodes_searched / (start_time.elapsed().as_millis() as u64 + 1) * 1000;
        println!(
            "info nodes {} nps {} depth {}",
            self.nodes_searched, nps, depth
        );
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

fn unmake(position: &mut Chess, m: &Move) {
    let opposite_move = Move::Normal { role: m.role(), from: m.to(), capture: None, to: m.from().unwrap(), promotion: None };
    position.play_unchecked(&opposite_move);
    todo!();
}


fn order_moves(position: &Chess, tt: &TranspositionTable) -> Vec<Move> {
    // MVV-LVA (most valuable capture, least valuable attacker)
    // Hash move
    let legal_moves = position.legal_moves().to_vec();

    let hash_move = match tt.get(
        &position
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0,
    ) {
        Some(transposition) => transposition.best_move,
        None => NULL_MOVE,
    };

    let mut scores = vec![0; legal_moves.len()];

    for (i, chess_move) in legal_moves.iter().enumerate() {
        if chess_move == &hash_move {
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

    let mut sorted_moves: Vec<(&Move, i16)> = legal_moves.iter().zip(scores).collect();
    sorted_moves.sort_by(|a, b| a.1.cmp(&b.1));

    return sorted_moves.iter().map(|x| x.0.clone()).collect();
}

fn quiesce(position: &mut Chess, mut alpha: i16, beta: i16, depth_from_root: u8, max_time: u64, start_time: SystemTime) -> Option<i16> {
    if start_time.elapsed().unwrap() >= Duration::from_millis(max_time) {
        return None;
    }

    let stand_pat = evaluate(&position);

    if stand_pat >= beta {
        return Some(beta);
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }

    for chess_move in position.capture_moves() {
        let mut new_position = position.clone();
        new_position.play_unchecked(&chess_move);
        let evaluation = -quiesce(&mut new_position, -beta, -alpha, depth_from_root + 1, max_time, start_time)?;

        if evaluation >= beta {
            return Some(beta);
        }
        if evaluation > alpha {
            alpha = evaluation;
        }
    }

    return Some(alpha);
}

fn alpha_beta(
    position: &mut Chess,
    mut alpha: i16,
    beta: i16,
    depth_left: u8,
    depth_from_root: u8,
    tt: &mut TranspositionTable,
    max_time: u64,
    start_time: SystemTime,
) -> Option<i16> {
    if start_time.elapsed().unwrap() >= Duration::from_millis(max_time) {
        return None;
    }
    let zobrist = position
        .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
        .0;

    let table_lookup = tt.probe_table(&zobrist, depth_left, alpha, beta);
    if table_lookup.is_some() {
        let mut new_position = position.clone();
        let chess_move = table_lookup.clone().unwrap().0;
        new_position.play_unchecked(&chess_move);
        return Some(table_lookup.unwrap().1);
    }

    if depth_left == 0 {
        let evaluation = quiesce(position, alpha, beta, depth_from_root + 1, max_time, start_time)?;

        tt.insert(
            zobrist,
            NULL_MOVE,
            evaluation,
            depth_left,
            EvaluationType::Exact,
        );
        return Some(evaluation);
    }

    let moves = order_moves(&position, &tt);

    if moves.len() == 0 {
        return Some(0);
    }

    let mut best_move = NULL_MOVE;

    for chess_move in moves {
        let mut new_position = position.clone();
        new_position.play_unchecked(&chess_move);
        let evaluation = -alpha_beta(
            &mut new_position,
            -beta,
            -alpha,
            depth_left - 1,
            depth_from_root + 1,
            tt,
            max_time,
            start_time,
        )?;
        if evaluation >= beta {
            tt.insert(
                zobrist,
                chess_move.clone(),
                beta,
                depth_left,
                EvaluationType::Beta,
            );
            return Some(beta);
        }
        if evaluation > alpha {
            alpha = evaluation;
            best_move = chess_move;
        }
    }

    tt.insert(zobrist, best_move, alpha, depth_left, EvaluationType::Alpha);
    return Some(alpha);
}

pub fn root_search(
    position: &mut Chess,
    depth_left: u8,
    tt: &mut TranspositionTable,
    max_time: u64,
) -> Option<(Move, i16)> {
    let start_time = SystemTime::now();
    let zobrist = position
        .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
        .0;

    let mut alpha = NEGATIVE_INFINITY;
    let beta = POSITIVE_INFINITY;

    let moves = order_moves(&position, tt);

    let mut best_move = moves[0].clone();

    for chess_move in moves {
        let mut new_position = position.clone();
        new_position.play_unchecked(&chess_move);
        let evaluation = -alpha_beta(
            &mut new_position,
            -beta,
            -alpha,
            depth_left - 1,
            1,
            tt,
            max_time,
            start_time,
        )?;

        if evaluation >= beta {
            tt.insert(
                zobrist,
                chess_move.clone(),
                beta,
                depth_left,
                EvaluationType::Beta,
            );
            return Some((chess_move, beta));
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
    return Some((best_move, alpha));
}

extern crate test;

#[cfg(test)]
mod tests {
    use shakmaty::{fen::Fen, Chess};
    use test::Bencher;

    use super::*;

    #[test]
    fn test_evaluation() {
        let position = Chess::new();
        let evauation = evaluate(&position);
        assert_eq!(evauation, 0);
    }

    #[test]
    fn test_alpha_beta() {
        // Create a test position
        let mut position = Chess::new(); // You may want to set up a specific test position here
        let mut tt = TranspositionTable::new(64);

        // Call your alpha-beta function
        let evaluation = alpha_beta(
            &mut position,
            NEGATIVE_INFINITY,
            POSITIVE_INFINITY,
            3,
            0,
            &mut tt,
            1000,
            SystemTime::now(),
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
        let (_, evaluation) = root_search(&mut position, 3, &mut tt, 1000).unwrap();

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

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let mut engine = Engine::new();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 40);

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

        let result = order_moves(&position, &tt);

        assert_eq!(result.len(), position.legal_moves().len());
    }

    #[bench]
    fn bench_search(b: &mut Bencher) {
        let position = Chess::new();
        let mut tt = TranspositionTable::new(64);

        let alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;

        b.iter(|| {
            alpha_beta(
                &mut position.clone(),
                alpha,
                beta,
                3,
                0,
                &mut tt,
                1000,
                SystemTime::now(),
            )
        })
    }
}
