use log::info;
use rand::seq::SliceRandom;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{uci::Uci, CastlingMode, Chess, Color, Move, MoveList, Position, Role, Square};
use std::io::prelude::*;
use std::str::FromStr;
use std::{collections::HashMap, fs::File, time::Instant};
// use rayon::prelude::*;

mod evaluator;

const NULL_MOVE: Move = Move::Normal {
    role: Role::Pawn,
    from: Square::A1,
    capture: None,
    to: Square::A1,
    promotion: None,
};

const POSITIVE_INFINITY: i32 = 999999999;
const NEGATIVE_INFINITY: i32 = -999999999;


pub struct Engine {
    tt: HashMap<Chess, (i32, u8, Color)>,
    book: HashMap<u64, Vec<String>>,
    evaluator: evaluator::Evaluator,
}

impl Engine {
    pub fn new() -> Engine {
        let mut file = File::open("openings.json").unwrap();
        let mut openings = String::new();

        file.read_to_string(&mut openings).unwrap();

        let book = serde_json::from_str(&openings).unwrap();
        Engine {
            tt: HashMap::new(),
            book,
            evaluator: evaluator::Evaluator::new(),
        }
    }

    fn order_moves(&self, position: Chess) -> MoveList {
        position.legal_moves()
    }

    fn quiesce(
        &self,
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
        let stand_pat = self.evaluator.evaluate(position.clone(), depth_from_root);
        if stand_pat >= beta {
            return Some(beta);
        }
        if alpha < stand_pat {
            alpha = stand_pat;
        }

        for chess_move in position.capture_moves().clone() {
            let score = -self.quiesce(
                position.clone().play(&chess_move).unwrap(),
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

    fn alpha_beta(
        &mut self,
        position: Chess,
        mut alpha: i32,
        beta: i32,
        depth_left: u8,
        depth_from_root: u8,
        start_time: Instant,
        max_time: u64,
    ) -> Option<i32> {
        if (start_time.elapsed().as_millis() as u64) > max_time {
            return None;
        }

        let turn = position.turn();

        match self.tt.get(&position) {
            Some((evaluation, depth, _turn)) => {
                if depth >= &depth_left {
                    if _turn == &turn {
                        return Some(*evaluation);
                    }
                    return Some(-evaluation);
                }
            }
            None => (),
        };

        if (depth_left == 0) || position.is_game_over() {
                let evaluation = self.quiesce(
                    position.clone(),
                    alpha,
                    beta,
                    depth_from_root + 1,
                    start_time,
                    max_time,
                )?;
                self.tt.insert(position.clone(), (evaluation, depth_left, turn));
                return Some(evaluation);
        }

        let moves = self.order_moves(position.clone());

        for chess_move in moves {
            let score = -self.alpha_beta(
                position.clone().play(&chess_move).unwrap(),
                -beta,
                -alpha,
                depth_left - 1,
                depth_from_root + 1,
                start_time,
                max_time,
            )?;
            if score >= beta {
                self.tt.insert(position.clone(), (beta, depth_left, turn));
                return Some(beta);
            }
            if score > alpha {
                alpha = score;
            }
        }

        self.tt.insert(position.clone(), (alpha, depth_left, turn));
        return Some(alpha);
    }

    fn root_search(
        &mut self,
        position: Chess,
        max_depth: u8,
        start_time: Instant,
        max_time: u64,
    ) -> Option<(Move, i32)> {
        if (start_time.elapsed().as_millis() as u64) > max_time {
            return None;
        }

        let mut alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;

        let ordered_moves = self.order_moves(position.clone());

        let mut best_move = position.legal_moves()[0].clone();

        for chess_move in ordered_moves {
            let evaluation = -self.alpha_beta(
                position.clone().play(&chess_move).unwrap(),
                -beta,
                -alpha,
                max_depth - 1,
                1,
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

        let mut best_move = NULL_MOVE;
        let mut best_evaluation = NEGATIVE_INFINITY;

        let mut depth: u8 = 1;

        while depth <= max_depth {
            info!("searching {} ply deep", depth);
            (best_move, best_evaluation) =
                match self.root_search(position.clone(), depth, start_time, max_time) {
                    Some(val) => val,
                    None => {
                        info!("search cancelled (time)");
                        break;
                    }
                };
            depth += 1;
        }

        //info!("hash table size: {}", self.tt);
        (best_move, best_evaluation)
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
