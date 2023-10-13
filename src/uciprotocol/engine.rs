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

struct Transposition {
    depth: u8,
    evaluation: i32,
    color: Color,
}

struct TranspositionTable {
    table: HashMap<Chess, Transposition>,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, evaluation: i32, position: Chess, depth: u8, color: Color) -> i32 {
        self.table.insert(
            position,
            Transposition {
                depth,
                evaluation,
                color,
            },
        );
        evaluation
    }
}

pub struct Engine {
    tt: TranspositionTable,
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
            tt: TranspositionTable::new(),
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
        match self.tt.table.get(&position) {
            Some(transposition) => {
                if transposition.depth >= depth_left {
                    let evlauation = transposition.evaluation;
                    if transposition.color == position.turn() {
                        return Some(evlauation);
                    }
                    return Some(-evlauation);
                }
            }
            None => (),
        };

        if depth_left == 0 {
            return Some(self.tt.insert(
                self.quiesce(
                    position.clone(),
                    alpha,
                    beta,
                    depth_from_root + 1,
                    start_time,
                    max_time,
                )?,
                position.clone(),
                depth_from_root,
                position.turn(),
            ));
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
                return Some(self.tt.insert(
                    beta,
                    position.clone(),
                    depth_from_root,
                    position.turn(),
                ));
            }
            if score > alpha {
                alpha = score;
            }
        }

        return Some(
            self.tt
                .insert(alpha, position.clone(), depth_from_root, position.turn()),
        );
    }

    fn root_search(
        &mut self,
        position: Chess,
        max_depth: u8,
        start_time: Instant,
        max_time: u64,
    ) -> Option<(Move, i32)> {
        let mut alpha = NEGATIVE_INFINITY;
        let beta = POSITIVE_INFINITY;

        let ordered_moves = self.order_moves(position.clone());

        let mut best_move = NULL_MOVE;

        for chess_move in ordered_moves {
            let evaluation = self.alpha_beta(
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

        let mut best_move = position.legal_moves()[0].clone();
        let mut best_evaluation = NEGATIVE_INFINITY;

        let mut depth: u8 = 1;

        while ((start_time.elapsed().as_millis() as u64) < max_time) && (depth <= max_depth) {
            info!("searching {} ply deep", depth);
            (best_move, best_evaluation) = match self.root_search(position.clone(), depth, start_time, max_time) {
                Some(val) => val,
                None => {info!("search cancelled (time)"); break}
            };
            depth += 1;
        }

        info!("eval {}", best_evaluation);
        (best_move, best_evaluation)
    }

    pub fn find_best_move(&mut self, position: Chess, max_time: u64) -> (Move, Uci) {
        let zobrist = position.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal);
        if self.book.contains_key(&zobrist.0) {
            info!("using book");
            let moves = self.book.get(&zobrist.0).unwrap();
            let move_string = moves.choose(&mut rand::thread_rng()).unwrap();
            let uci = Uci::from_str(move_string).unwrap();
            let chess_move = uci.to_move(&position).unwrap();
            return (chess_move, uci);
        }
        let (best_move, _) = self.iterative_deepening(position, max_time, 20);
        (
            best_move.clone(),
            best_move.clone().to_uci(CastlingMode::Standard),
        )
    }
}

impl Default for Engine {
    fn default() -> Engine {
        Engine::new()
    }
}
