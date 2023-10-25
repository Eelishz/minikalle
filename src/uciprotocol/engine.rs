use log::info;
use rand::seq::SliceRandom;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{uci::Uci, CastlingMode, Chess, Move, Position};
use shakmaty::{Role, Square};
use std::str::FromStr;
use std::{collections::HashMap, time::Instant};
mod evaluator;
mod openings;
mod transpositiontable;
use evaluator::evaluate;
use openings::OPENINGS;
use transpositiontable::{EvaluationType, TranspositionTable};

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
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut engine = Engine::new();

        // Call your alpha-beta function
        let evaluation = engine
            .alpha_beta(
                position,
                NEGATIVE_INFINITY,
                POSITIVE_INFINITY,
                3,
                0,
                vec![],
                Instant::now(),
                1000,
            )
            .unwrap();

        // Assert that the result is as expected
        assert!(evaluation >= 0);
    }

    #[test]
    fn test_root_search() {
        // Create a test position
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut engine = Engine::new();

        // Call your alpha-beta function
        let (_, evaluation) = engine.root_search(&position, 3, Instant::now(), 1000).unwrap();

        // Assert that the result is as expected
        assert!(evaluation >= 0);
    }

    #[test]
    fn test_quiesce() {
        // Create a test position
        let position = Chess::new(); // You may want to set up a specific test position here
        let mut engine = Engine::new();

        // Call your alpha-beta function
        let evaluation = engine
            .quiesce(
                position,
                NEGATIVE_INFINITY,
                POSITIVE_INFINITY,
                0,
                Instant::now(),
                1000,
            )
            .unwrap();

        // Assert that the result is as expected
        assert_eq!(evaluation, 0);
    }

    #[test]
    fn test_mates() {
        // Some easy checkmates
        let mut engine = Engine::new();

        let fen: Fen = "6k1/2R5/8/8/8/3R4/2K5/8 w - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000, 40);

        assert_eq!(uci.to_string(), "d3d8".to_string());

        let fen: Fen = "6k1/2p4p/2p4b/p7/3P1p2/2P2P2/PP2b1KP/4q3 b - - 9 35"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000, 40);

        assert_eq!(uci.to_string(), "e1f1".to_string());

        let fen: Fen = "6k1/2p4p/b1p1q2b/p7/3P1pp1/2P2P2/PP4PP/4B1K1 b - - 1 29"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000, 40);

        assert_eq!(uci.to_string(), "e6e1".to_string());

        let fen: Fen = "4r1k1/ppp2ppp/5n2/6P1/1PP5/2b4P/r7/5K1R b - - 0 33"
            .parse()
            .unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000, 40);

        assert_eq!(uci.to_string(), "e8e1".to_string());
    }

    #[test]
    fn test_captures() {
        let fen: Fen = "7k/8/8/4p3/3Q4/8/8/K7 w - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let mut engine = Engine::new();

        let (_, uci, _) = engine.find_best_move(position, 1_000, 40);

        assert_eq!(uci.to_string(), "d4e5".to_string());

        let fen: Fen = "7k/8/8/4q3/3Q4/8/8/K7 b - - 0 1".parse().unwrap();

        let position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(position, 1_000, 40);

        assert_eq!(uci.to_string(), "e5d4".to_string());
    }

    #[test]
    fn test_move_ordering() {
        let position = Chess::new();
        let engine = Engine::new();

        let result = engine.order_moves(&position);

        assert_eq!(result.len(), position.legal_moves().len());
    }

    #[test]
    fn test_move_repetition() {
        let engine = Engine::new();

        let position = Chess::new();
        let zobrist = position
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0;

        let mut repetition_table = vec![zobrist; 3];

        let result = engine.threefold_rule(&mut repetition_table);

        assert_eq!(result, true);
    }
    #[bench]
    fn bench_search(b: &mut Bencher) {
        let position = Chess::new();

        let alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;
        let mut engine = Engine::new();

        b.iter(|| {
            engine.alpha_beta(
                position.clone(),
                alpha,
                beta,
                3,
                0,
                vec![],
                Instant::now(),
                1_000_000,
            )
        })
    }
}

const POSITIVE_INFINITY: i16 = 30000;
const NEGATIVE_INFINITY: i16 = -30000;

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
    repetition_table: Vec<u64>,
}

impl Engine {
    pub fn new() -> Engine {
        let book = serde_json::from_str(&OPENINGS).unwrap();
        Engine {
            tt: TranspositionTable::new(256),
            book,
            nodes_searched: 0,
            repetition_table: vec![],
        }
    }

    fn order_moves(&self, position: &Chess) -> Vec<Move> {
        //MVV-LVA (most valuable capture, least valuable attacker)
        //Hash move
        let legal_moves = position.legal_moves().to_vec();

        let hash_move = match self.tt.get(
            &position
                .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
                .0,
        ) {
            Some(transposition) => transposition.best_move,
            None => NULL_MOVE,
        };

        let mut scores = vec![0; legal_moves.len()];

        for (i, chess_move) in legal_moves.iter().enumerate() {
            if legal_moves.contains(&hash_move) || (chess_move == &hash_move) {
                scores[i] = -99999;
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

        let mut sorted_moves: Vec<(&Move, i32)> = legal_moves.iter().zip(scores).collect();
        sorted_moves.sort_by(|a, b| a.1.cmp(&b.1));

        return sorted_moves.iter().map(|x| x.0.clone()).collect();
    }

    fn quiesce(
        &mut self,
        position: Chess,
        mut alpha: i16,
        beta: i16,
        depth_from_root: u8,
        start_time: Instant,
        max_time: u64,
    ) -> Option<i16> {
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
            let evaluation = -self.quiesce(
                new_position,
                -beta,
                -alpha,
                depth_from_root + 1,
                start_time,
                max_time,
            )?;

            if evaluation >= beta {
                return Some(beta);
            }
            if evaluation > alpha {
                alpha = evaluation;
            }
        }

        Some(alpha)
    }

    pub fn threefold_rule(&self, repetition_table: &mut Vec<u64>) -> bool {
        repetition_table.sort_unstable();
        let mut i = 1;

        for j in 1..repetition_table.len() {
            if repetition_table[j] == repetition_table[j - 1] {
                i += 1;
            } else {
                i = 1;
            }
        }

        i >= 3
    }

    fn calculate_extension(&self, chess_move: &Move) -> u8 {
        if chess_move.is_capture() {
            return 1;
        } else if chess_move.is_promotion() {
            return 1;
        }
        return 0;
    }

    fn alpha_beta(
        &mut self,
        position: Chess,
        mut alpha: i16,
        beta: i16,
        depth_left: u8,
        depth_from_root: u8,
        mut position_table: Vec<u64>,
        start_time: Instant,
        max_time: u64,
    ) -> Option<i16> {
        if (start_time.elapsed().as_millis() as u64) > max_time {
            return None;
        }

        self.nodes_searched += 1;

        let zobrist = position
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0;

        position_table.push(zobrist);

        if self.threefold_rule(&mut position_table) {
            return Some(0);
        }

        let table_lookup = self.tt.probe_table(&zobrist, depth_left, alpha, beta);
        if table_lookup.is_some() {
            let mut new_position = position.clone();
            let chess_move = table_lookup.clone().unwrap().0;
            new_position.play_unchecked(&chess_move);
            return Some(table_lookup.unwrap().1);
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

            self.tt.insert(
                zobrist,
                NULL_MOVE,
                evaluation,
                depth_left,
                EvaluationType::Exact,
            );
            return Some(evaluation);
        }

        let moves = self.order_moves(&position);

        if moves.len() == 0 {
            return Some(0);
        }

        let mut best_move = NULL_MOVE;

        for chess_move in moves {
            if chess_move.is_capture() {
                position_table.clear();
            }

            let extension = self.calculate_extension(&chess_move);

            let mut new_position = position.clone();
            new_position.play_unchecked(&chess_move);
            let evaluation = -self.alpha_beta(
                new_position,
                -beta,
                -alpha,
                depth_left + extension - 1,
                depth_from_root + 1,
                position_table.clone(),
                start_time,
                max_time,
            )?;
            if evaluation >= beta {
                self.tt.insert(
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

        self.tt
            .insert(zobrist, best_move, alpha, depth_left, EvaluationType::Alpha);
        return Some(alpha);
    }

    pub fn root_search(
        &mut self,
        position: &Chess,
        depth_left: u8,
        start_time: Instant,
        max_time: u64,
    ) -> Option<(Move, i16)> {
        self.nodes_searched += 1;

        let zobrist = position
            .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
            .0;

        let mut position_table = self.repetition_table.clone();

        let mut alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;

        let moves = self.order_moves(&position);

        let mut best_move = moves[0].clone();

        if depth_left == 0 {
            let mut best_evaluation = NEGATIVE_INFINITY;
            for chess_move in moves {
                let mut new_position = position.clone();
                new_position.play_unchecked(&chess_move);
                let evaluation = -evaluate(&new_position);

                if evaluation > best_evaluation {
                    best_move = chess_move.clone();
                    best_evaluation = evaluation;
                }
            }
            return Some((best_move, best_evaluation));
        }

        position_table.push(zobrist);

        for chess_move in moves {
            if chess_move.is_capture() {
                position_table.clear();
            }

            let extension = self.calculate_extension(&chess_move);

            let mut new_position = position.clone();
            new_position.play_unchecked(&chess_move);
            let evaluation = -self
                .alpha_beta(
                    new_position,
                    -beta,
                    -alpha,
                    depth_left + extension - 1,
                    1,
                    position_table.clone(),
                    start_time,
                    max_time,
                )?;

            if evaluation >= beta {
                self.tt.insert(
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

        self.tt.insert(
            zobrist,
            best_move.clone(),
            alpha,
            depth_left,
            EvaluationType::Alpha,
        );
        return Some((best_move, alpha));
    }

    fn iterative_deepening(
        &mut self,
        position: Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, i16) {
        let start_time = Instant::now();

        let mut best_move = position.legal_moves()[0].clone();
        let mut best_evaluation = NEGATIVE_INFINITY;

        let mut depth: u8 = 0;

        self.nodes_searched = 0;

        while depth < max_depth {
            info!("searching {} ply deep", depth);
            let search = self.root_search(&position, depth, start_time, max_time);

            if search.is_none() {
                break;
            }

            best_move = search.clone().unwrap().0;
            best_evaluation = search.unwrap().1;

            let nps = self.nodes_searched / (start_time.elapsed().as_millis() as u64 + 1) * 1000;
            println!(
                "info nodes {0} nps {nps} depth {depth}",
                self.nodes_searched
            );
            println!("info score cp {}", best_evaluation);
            match best_evaluation {
                POSITIVE_INFINITY => {
                    println!("info score mate {}", depth + 1);
                    break;
                }
                NEGATIVE_INFINITY => {
                    println!("info score mate -{}", depth + 1);
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
        (best_move, best_evaluation)
    }

    pub fn clear_repetition_table(&mut self) {
        self.repetition_table.clear();
    }

    pub fn find_best_move(
        &mut self,
        position: Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, Uci, i16) {
        let zobrist = position.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal);
        if self.book.contains_key(&zobrist.0) {
            info!("using book");
            let moves = self.book.get(&zobrist.0).unwrap();
            let move_string = moves.choose(&mut rand::thread_rng()).unwrap();
            let uci = Uci::from_str(move_string).unwrap();
            let chess_move = uci.to_move(&position).unwrap();
            return (chess_move, uci, 0);
        }
        let (best_move, evaluation) = self.iterative_deepening(position, max_time, max_depth);
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
