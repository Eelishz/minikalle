use log::info;
use rand::seq::SliceRandom;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{uci::Uci, CastlingMode, Chess, Move, Position, Role, Square};
use std::io::prelude::*;
use std::str::FromStr;
use std::{collections::HashMap, fs::File, time::Instant};
mod evaluator;
mod transpositiontable;
use evaluator::evaluate;
use transpositiontable::{EvaluationType, TranspositionTable};

extern crate test;

#[cfg(test)]
mod tests {
    use shakmaty::{fen::Fen, Chess};
    use test::{black_box, Bencher};

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
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut engine = Engine::new();

        // Call your alpha-beta function
        let result = engine.alpha_beta(
            position,
            NEGATIVE_INFINITY,
            POSITIVE_INFINITY,
            3,
            0,
            vec![],
            Instant::now(),
            1000,
        );

        // Assert that the result is as expected
        assert!(result >= Some(0));
    }

    #[test]
    fn test_quiesce() {
        // Create a test position
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut engine = Engine::new();

        // Call your alpha-beta function
        let result = engine.quiesce(
            position,
            NEGATIVE_INFINITY,
            POSITIVE_INFINITY,
            0,
            Instant::now(),
            1000,
        );

        // Assert that the result is as expected
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_root_search() {
        // Create a test position
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut engine = Engine::new();

        // Call your alpha-beta function
        let (_, result) = black_box(
            engine
                .root_search(&position, 3, vec![], Instant::now(), 1000)
                .unwrap(),
        );

        // Assert that the result is as expected
        assert!(result >= 0);
    }

    #[test]
    fn test_mates() {
        // Some easy checkmates
        let mut engine = Engine::new();

        let fen: Fen = "6k1/2R5/8/8/8/3R4/2K5/8 w - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000);

        assert_eq!(uci.to_string(), "d3d8".to_string());

        let fen: Fen = "6k1/2p4p/2p4b/p7/3P1p2/2P2P2/PP2b1KP/4q3 b - - 9 35"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000);

        assert_eq!(uci.to_string(), "e1f1".to_string());

        let fen: Fen = "6k1/2p4p/b1p1q2b/p7/3P1pp1/2P2P2/PP4PP/4B1K1 b - - 1 29"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000);

        assert_eq!(uci.to_string(), "e6e1".to_string());
    }

    #[test]
    fn test_captures() {
        let fen: Fen = "7k/8/8/4p3/3Q4/8/8/K7 w - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let mut engine = Engine::new();

        let (_, uci, _) = engine.find_best_move(position, 1_000);

        assert_eq!(uci.to_string(), "d4e5".to_string());

        let fen: Fen = "7k/8/8/4q3/3Q4/8/8/K7 b - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000);

        assert_eq!(uci.to_string(), "e5d4".to_string());
    }

    #[test]
    fn test_move_ordering() {
        let position = Chess::new();
        let engine = Engine::new();

        let result = engine.order_moves(&position);

        assert_eq!(result.len(), position.legal_moves().len());
    }

    #[bench]
    fn bench_root_search(b: &mut Bencher) {
        let mut engine = Engine::new();
        let position = Chess::new();

        b.iter(|| engine.root_search(&position, 5, vec![], Instant::now(), 1_000_000))
    }
}

const NULL_MOVE: Move = Move::Normal {
    role: Role::Pawn,
    from: Square::A1,
    capture: None,
    to: Square::A1,
    promotion: None,
};

const POSITIVE_INFINITY: i32 = 100000;
const NEGATIVE_INFINITY: i32 = -100000;

pub struct Engine {
    tt: TranspositionTable,
    book: HashMap<u64, Vec<String>>,
    nodes_searched: u64,
    repetition_table: Vec<u64>,
}

impl Engine {
    pub fn new() -> Engine {
        let mut file = File::open("openings.json").unwrap();
        let mut openings = String::new();

        file.read_to_string(&mut openings).unwrap();

        let book = serde_json::from_str(&openings).unwrap();
        Engine {
            tt: TranspositionTable::new(128),
            book,
            nodes_searched: 0,
            repetition_table: vec![],
        }
    }

    fn order_moves(&self, position: &Chess) -> Vec<Move> {
        let legal_moves = position.legal_moves().to_vec();
        let mut capture_moves = position.capture_moves().to_vec();
        let mut promotion_moves = position.promotion_moves().to_vec();
        let mut other_moves: Vec<Move> = vec![];
        for chess_move in legal_moves {
            if !capture_moves.contains(&chess_move) || !promotion_moves.contains(&chess_move) {
                other_moves.append(&mut vec![chess_move])
            }
        }

        let mut ordered_moves: Vec<Move> = vec![];

        ordered_moves.append(&mut promotion_moves);
        ordered_moves.append(&mut capture_moves);
        ordered_moves.append(&mut other_moves);

        return ordered_moves;
    }

    fn quiesce(
        &mut self,
        position: Chess,
        mut alpha: i32,
        beta: i32,
        depth_from_root: u8,
        start_time: Instant,
        max_time: u64,
    ) -> Option<i32> {
        if (start_time.elapsed().as_millis() as u64) > max_time {
            return None;
        }

        self.nodes_searched += 1;

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
            let score = -self.quiesce(
                new_position,
                -beta,
                -alpha,
                depth_from_root + 1,
                start_time,
                max_time,
            )?;

            if score >= beta {
                return Some(beta);
            }
            if score > alpha {
                alpha = score
            }
        }

        Some(alpha)
    }

    fn threefold_rule(&self, repetition_table: &mut Vec<u64>) -> bool {
        let mut map: HashMap<u64, u8> = HashMap::new();

        for pos in repetition_table {
            if map.contains_key(&pos) {
                map.insert(*pos, map.get(&pos).unwrap() + 1);
            } else {
                map.insert(*pos, 1);
            }
        }
        
        map.values().max().unwrap() > &3
    }

    fn calculate_extension(&self, position: &Chess, chess_move: &Move) -> u8 {
        if position.is_check() {
            return 1;
        }
        if chess_move.is_promotion() {
            return 1;
        }
        0
    }

    fn probe_table(&self, key: &u64, depth_left: u8, alpha: i32, beta: i32) -> Option<i32> {
        let transposition = self.tt.get(key)?;
        let evaluation = transposition.evaluation;
        if transposition.depth_left >= depth_left {
            if transposition.evaluation_type == EvaluationType::Exact {
                return Some(evaluation);
            }
            if (transposition.evaluation_type == EvaluationType::Alpha) && (evaluation <= alpha) {
                return Some(alpha);
            }
            if (transposition.evaluation_type == EvaluationType::Beta) && (evaluation >= beta) {
                return Some(beta);
            }
        }
        None
    }

    fn alpha_beta(
        &mut self,
        position: Chess,
        mut alpha: i32,
        beta: i32,
        depth_left: u8,
        depth_from_root: u8,
        mut position_table: Vec<u64>,
        start_time: Instant,
        max_time: u64,
    ) -> Option<i32> {
        if (start_time.elapsed().as_millis() as u64) > max_time {
            return None;
        }

        self.nodes_searched += 1;

        let zobrist = position
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0;

        position_table.push(zobrist);

        let table_lookup = self.probe_table(&zobrist, depth_left, alpha, beta);
        if table_lookup.is_some() {
            return table_lookup;
        }

        if self.threefold_rule(&mut position_table) {
            return Some(0);
        }

        if depth_left == 0 {
            let evaluation = self.quiesce(
                position,
                alpha,
                beta,
                depth_from_root + 1,
                start_time,
                max_time,
            )?;

            self.tt
                .insert(zobrist, evaluation, depth_left, EvaluationType::Exact);
            return Some(evaluation);
        }

        let moves = self.order_moves(&position);

        for chess_move in moves {
            let extensions = self.calculate_extension(&position, &chess_move);

            let mut new_position = position.clone();
            new_position.play_unchecked(&chess_move);
            let score = -self.alpha_beta(
                new_position,
                -beta,
                -alpha,
                depth_left + extensions - 1,
                depth_from_root + 1,
                if chess_move.is_capture() {
                    vec![]
                } else {
                    position_table.clone()
                },
                start_time,
                max_time,
            )?;
            if score >= beta {
                self.tt
                    .insert(zobrist, beta, depth_left, EvaluationType::Beta);
                return Some(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }

        self.tt
            .insert(zobrist, alpha, depth_left, EvaluationType::Alpha);
        return Some(alpha);
    }

    fn root_search(
        &mut self,
        position: &Chess,
        depth_left: u8,
        mut repetition_table: Vec<u64>,
        start_time: Instant,
        max_time: u64,
    ) -> Option<(Move, i32)> {
        if (start_time.elapsed().as_millis() as u64) > max_time {
            return None;
        }

        self.nodes_searched += 1;

        let zobrist = position
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0;

        repetition_table.push(zobrist);

        let mut alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;

        let ordered_moves = self.order_moves(&position);

        let mut best_move = position.legal_moves()[0].clone();

        for chess_move in ordered_moves {
            let mut new_position = position.clone();
            new_position.play_unchecked(&chess_move);
            let evaluation = -self.alpha_beta(
                new_position,
                -beta,
                -alpha,
                depth_left - 1,
                1,
                if chess_move.is_capture() {
                    vec![]
                } else {
                    repetition_table.clone()
                },
                start_time,
                max_time,
            )?;

            if evaluation >= beta {
                return Some((chess_move, beta));
            }
            if evaluation > alpha {
                alpha = evaluation;
                best_move = chess_move;
            }
        }
        return Some((best_move, alpha));
    }

    fn iterative_deepening(
        &mut self,
        position: Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, i32) {
        let start_time = Instant::now();

        let mut best_move = position.legal_moves()[0].clone();
        let mut best_evaluation = NEGATIVE_INFINITY;

        let mut depth: u8 = 1;

        self.nodes_searched = 0;

        while depth < max_depth {
            info!("searching {} ply deep", depth);
            (best_move, best_evaluation) = match self.root_search(
                &position,
                depth,
                self.repetition_table.clone(),
                start_time,
                max_time,
            ) {
                Some(val) => val,
                None => {
                    info!("search cancelled (time)");
                    break;
                }
            };
            let nps = self.nodes_searched / (start_time.elapsed().as_millis() as u64 + 1) * 1000;
            println!("info score cp {}", best_evaluation);
            println!(
                "info nodes {} nps {} depth {}",
                self.nodes_searched, nps, depth
            );
            if (best_evaluation == POSITIVE_INFINITY) || (best_evaluation == NEGATIVE_INFINITY) {
                break;
            }
            depth += 1;
        }

        let nps = self.nodes_searched / (start_time.elapsed().as_millis() as u64 + 1) * 1000;
        println!("info score cp {}", best_evaluation);
        println!(
            "info nodes {} nps {} depth {}",
            self.nodes_searched, nps, depth
        );
        (best_move, best_evaluation)
    }

    pub fn clear_repetition_table(&mut self) {
        self.repetition_table.clear();
    }

    pub fn find_best_move(&mut self, position: Chess, max_time: u64) -> (Move, Uci, i32) {
        let zobrist = position.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal);
        if self.book.contains_key(&zobrist.0) {
            info!("using book");
            let moves = self.book.get(&zobrist.0).unwrap();
            let move_string = moves.choose(&mut rand::thread_rng()).unwrap();
            let uci = Uci::from_str(move_string).unwrap();
            let chess_move = uci.to_move(&position).unwrap();
            return (chess_move, uci, 0);
        }
        let (best_move, evaluation) = self.iterative_deepening(position, max_time, 40);
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
