use crate::evaluation::evaluate;
use crate::openings::OPENINGS;
use crate::transpositiontable::{EvaluationType, TranspositionTable};
use rand::seq::SliceRandom;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{uci::Uci, CastlingMode, Chess, Move, Position};
use shakmaty::{MoveList, Role, Square};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::SystemTime;

pub const POS_INF: i16 = 25_000;
pub const NEG_INF: i16 = -25_000;
const INITIAL_WINDOW_SIZE: i16 = 40;
const R: u8 = 3;

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
    use_book: bool,
}

impl Engine {
    pub fn new() -> Engine {
        let book = serde_json::from_str(&OPENINGS).unwrap();
        Engine {
            tt: TranspositionTable::new(16),
            book,
            use_book: true,
        }
    }

    pub fn new_game(&mut self) {}

    pub fn set_hash(&mut self, value: usize) {
        self.tt = TranspositionTable::new(value);
    }

    pub fn set_book(&mut self, value: bool) {
        self.use_book = value;
    }

    fn iterative_deepening(
        &mut self,
        position: &Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, i16) {
        let start_time = SystemTime::now();

        // Initial guess for aspiration window

        let (mut evaluation, mut best_move, mut nodes_searched) = search(
            position,
            NEG_INF,
            POS_INF,
            1,
            0,
            &mut self.tt,
            0,
            max_time,
            &start_time,
        )
        .unwrap();

        let mut a_window = INITIAL_WINDOW_SIZE;
        let mut b_window = INITIAL_WINDOW_SIZE;
        let mut alpha = evaluation.saturating_sub(a_window);
        let mut beta = evaluation.saturating_add(b_window);

        let nps = nodes_searched / (start_time.elapsed().unwrap().as_millis() as u64 + 1) * 1000;

        println!("info nodes {0} nps {nps} depth 1", nodes_searched);
        match evaluation {
            POS_INF => {
                println!("info score mate 1");
                return (best_move, evaluation);
            }
            NEG_INF => {
                println!("info score mate -1");
                return (best_move, evaluation);
            }
            _ => println!("info score cp {}", evaluation),
        }

        let mut depth: u8 = 2;

        while depth < max_depth {
            let search = search(
                position,
                alpha,
                beta,
                depth,
                0,
                &mut self.tt,
                0,
                max_time,
                &start_time,
            );

            let new_evaluation;
            let new_best_move;

            match search {
                Some(s) => {
                    new_evaluation = s.0;
                    new_best_move = s.clone().1;
                    nodes_searched += s.2;
                }
                None => break,
            }

            if new_evaluation <= alpha {
                a_window *= 2;
                alpha -= a_window;
                continue;
            } else if new_evaluation >= beta {
                b_window *= 2;
                beta += b_window;
                continue;
            } else {
                a_window = INITIAL_WINDOW_SIZE;
                b_window = INITIAL_WINDOW_SIZE;
                evaluation = new_evaluation;
                best_move = new_best_move;
            }

            let nps =
                nodes_searched / (start_time.elapsed().unwrap().as_millis() as u64 + 1) * 1000;

            println!("info nodes {0} nps {nps} depth {depth}", nodes_searched);
            match evaluation {
                POS_INF => {
                    println!("info score mate {depth}");
                    break;
                }
                NEG_INF => {
                    println!("info score mate -{depth}");
                    break;
                }
                _ => println!("info score cp {}", evaluation),
            }

            depth += 1;
        }

        return (best_move, evaluation);
    }

    pub fn find_best_move(
        &mut self,
        position: &Chess,
        max_time: u64,
        max_depth: u8,
    ) -> (Move, Uci, i16) {
        let zobrist = position.zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal);
        if self.book.contains_key(&zobrist.0) && self.use_book {
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
        while j > 0 && scores[j] > scores[j - 1] {
            unsafe {
                scores.swap_unchecked(j, j - 1);
                moves.swap_unchecked(j, j - 1);
            }
            j -= 1;
        }
    }
}

#[inline]
fn order_moves(position: &Chess, tt: &TranspositionTable, zobrist: u64) -> MoveList {
    // MVV-LVA (most valuable capture, least valuable attacker)
    // Hash move
    let mut legal_moves = position.legal_moves();

    let hash_move = match tt.get(&zobrist) {
        Some(transposition) => transposition.best_move,
        None => NULL_MOVE,
    };

    let mut scores = [0; 256]; // 256 should be large enough

    for (i, m) in legal_moves.iter().enumerate() {
        if m == &hash_move {
            scores[i] = POS_INF;
        } else if m.is_capture() {
            let attacker = match m.role() {
                Role::Pawn => 100,
                Role::Knight => 300,
                Role::Bishop => 300,
                Role::Rook => 500,
                Role::Queen => 900,
                Role::King => 2000,
            };
            let victim = match m.capture().unwrap() {
                Role::Pawn => 100,
                Role::Knight => 300,
                Role::Bishop => 300,
                Role::Rook => 500,
                Role::Queen => 900,
                Role::King => 2000,
            };

            scores[i] = victim - attacker;
        }
    }

    let score_slice = &mut scores[0..legal_moves.len()];
    sort_moves(score_slice, &mut legal_moves);

    return legal_moves;
}

fn quiescence(
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

    let zobrist = position
        .zobrist_hash::<Zobrist64>(shakmaty::EnPassantMode::Legal)
        .0;

    let stand_pat = evaluate(&position);

    if stand_pat >= beta {
        return Some((beta, nodes_searched));
    }

    if alpha < stand_pat {
        alpha = stand_pat;
    }

    let moves = order_moves(position, tt, zobrist);

    for m in moves {
        let mut new_position = position.clone();
        new_position.play_unchecked(&m);

        if !m.is_capture() || !m.is_promotion() || !new_position.is_checkmate() {
            continue;
        }

        let (evaluation, new_searched) = quiescence(
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
fn is_passed_pawn(_position: &Chess, _m: &Move) -> bool {
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
    if m.is_promotion() {
        return 1;
    }
    if position.is_check() {
        return 1;
    }
    if is_passed_pawn(position, m) {
        return 1;
    }

    return 0;
}

fn search(
    position: &Chess,
    mut alpha: i16,
    beta: i16,
    depth_left: u8,
    depth_from_root: u8,
    tt: &mut TranspositionTable,
    mut nodes_searched: u64,
    max_time: u64,
    start_time: &SystemTime,
) -> Option<(i16, Move, u64)> {
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
        return Some((
            table_lookup.clone().unwrap().1,
            table_lookup.unwrap().0,
            nodes_searched,
        ));
    }

    if depth_left == 0 {
        let (evaluation, new_seached) = quiescence(
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
        return Some((evaluation, NULL_MOVE, nodes_searched));
    }

    let moves = order_moves(&position, &tt, zobrist);

    let mut capture_moves = false;
    for m in &moves {
        if m.is_capture() {
            capture_moves = true;
            break;
        }
    }

    if !position.is_check() && !capture_moves && depth_left <= 2 {
        let futility_margin = 200;
        let evaluation = evaluate(position);

        if evaluation + futility_margin <= alpha {
            return Some((alpha, NULL_MOVE, nodes_searched));
        }
    }

    let null_move_possible = !position.is_check();

    if null_move_possible && depth_left >= 3 {
        let new_position = position.clone();
        let new_position = new_position.swap_turn().unwrap();

        let (evaluation, _, new_searched) = search(
            &new_position,
            -beta,
            1 - beta,
            (depth_left as i8 - R as i8 - 1).max(0) as u8,
            depth_from_root + 1,
            tt,
            nodes_searched,
            max_time,
            start_time,
        )?;

        let evaluation = -evaluation;
        nodes_searched = new_searched;

        if evaluation >= beta {
            return Some((beta, NULL_MOVE, nodes_searched));
        }
    }

    if moves.len() == 0 {
        return Some((evaluate(position), NULL_MOVE, nodes_searched));
    }

    let mut best_move = NULL_MOVE;

    // main bit

    for (i, m) in moves.iter().enumerate() {
        // Make move.
        // Move is unmade automatically when `new_position` is dropped.
        let mut new_position = position.clone();
        new_position.play_unchecked(m);

        let extension = calculate_extension(m, position, depth_left);

        let lmr_depth = if depth_left < 3
            && new_position.is_check()
            && extension == 0
            && !m.is_capture()
            && i > 4
        {
            (depth_left - 1).max(1) - 1
        } else {
            depth_left - 1
        };
        let (evaluation, _, new_searched) = search(
            &new_position,
            -beta,
            -alpha,
            lmr_depth + extension,
            depth_from_root + 1,
            tt,
            nodes_searched,
            max_time,
            start_time,
        )?;
        nodes_searched = new_searched;
        let evaluation = -evaluation;
        if evaluation >= beta {
            tt.insert(zobrist, m.clone(), beta, depth_left, EvaluationType::Beta);
            return Some((beta, m.clone(), nodes_searched));
        }
        if evaluation > alpha {
            alpha = evaluation;
            best_move = m.clone();
        }
    }

    tt.insert(
        zobrist,
        best_move.clone(),
        alpha,
        depth_left,
        EvaluationType::Alpha,
    );
    return Some((alpha, best_move, nodes_searched));
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
        let (evaluation, _, _) = search(
            &position,
            NEG_INF,
            POS_INF,
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
        let (evaluation, _, _) = search(
            &mut position,
            NEG_INF,
            POS_INF,
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
    fn test_mates() {
        // Some easy checkmates
        let mut engine = Engine::new();

        let fen: Fen = "6k1/2R5/8/8/8/3R4/2K5/8 w - - 0 1".parse().unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 5);

        assert_eq!(uci.to_string(), "d3d8".to_string());

        let fen: Fen = "6k1/2p4p/2p4b/p7/3P1p2/2P2P2/PP2b1KP/4q3 b - - 9 35"
            .parse()
            .unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 5);

        assert_eq!(uci.to_string(), "e1f1".to_string());

        let fen: Fen = "6k1/2p4p/b1p1q2b/p7/3P1pp1/2P2P2/PP4PP/4B1K1 b - - 1 29"
            .parse()
            .unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 5);

        assert_eq!(uci.to_string(), "e6e1".to_string());

        let fen: Fen = "4r1k1/ppp2ppp/5n2/6P1/1PP5/2b4P/r7/5K1R b - - 0 33"
            .parse()
            .unwrap();

        let mut position: Chess = fen.into_position(shakmaty::CastlingMode::Standard).unwrap();

        let (_, uci, _) = engine.find_best_move(&mut position, 1_000, 5);

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

        let result = order_moves(&position, &tt, 0);

        assert_eq!(result.len(), position.legal_moves().len());
    }

    #[bench]
    fn bench_search(b: &mut Bencher) {
        let position = Chess::new();
        let mut tt = TranspositionTable::new(64);

        let alpha = NEG_INF;
        let beta = POS_INF;

        b.iter(|| {
            search(
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
