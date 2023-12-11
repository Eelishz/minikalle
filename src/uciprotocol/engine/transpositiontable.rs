use std::mem::size_of;
use shakmaty::{Move, Square, Role};

const NULL_MOVE: Move = Move::Normal {
    role: Role::Pawn,
    from: Square::A1,
    capture: None,
    to: Square::A1,
    promotion: None,
};

#[derive(Clone, Copy, PartialEq)]
pub enum EvaluationType {
    Exact,
    Alpha,
    Beta,
}

#[derive(Clone)]
pub struct Transposition {
    pub key: u64,
    pub best_move: Move,
    pub evaluation: i16,
    pub depth_left: u8,
    pub evaluation_type: EvaluationType,
}

impl Transposition {
    pub fn new() -> Transposition {
        Transposition {
            key: 0,
            best_move: NULL_MOVE,
            evaluation: 0,
            depth_left: 0,
            evaluation_type: EvaluationType::Exact,
        }
    }
}

pub struct TranspositionTable {
    transpositions: Vec<Transposition>,
    count: u64,
}

impl TranspositionTable {
    pub fn new(desired_size_in_mb: u32) -> TranspositionTable {
        let tt_entry_size_in_bytes: usize = size_of::<Transposition>();
        let desired_size_in_bytes: u32 = desired_size_in_mb * 1024 * 1024;
        let num_entries: usize = (desired_size_in_bytes as usize) / tt_entry_size_in_bytes;

        TranspositionTable {
            transpositions: vec![Transposition::new(); num_entries],
            count: num_entries as u64,
        }
    }

    fn index(&self, key: &u64) -> usize {
        (key % self.count) as usize
    }

    pub fn insert(
        &mut self,
        key: u64,
        best_move: Move,
        evaluation: i16,
        depth_left: u8,
        evaluation_type: EvaluationType,
    ) {
        let index = self.index(&key);
        let entry = Transposition {
            key,
            best_move,
            evaluation,
            depth_left,
            evaluation_type,
        };
        self.transpositions[index] = entry;
    }

    pub fn get(&self, key: &u64) -> Option<Transposition> {
        let index = self.index(key);
        let transposition = &self.transpositions[index];
        if &transposition.key == key {
            Some(transposition.clone())
        } else {
            None
        }
    }

    pub fn probe_table(&self, key: &u64, depth_left: u8, alpha: i16, beta: i16) -> Option<(Move, i16)> {
        let transposition = self.get(key)?;
        let best_move = transposition.best_move;
        let evaluation = transposition.evaluation;
        if transposition.depth_left >= depth_left {
            if transposition.evaluation_type == EvaluationType::Exact {
                return Some((best_move, evaluation));
            }
            if (transposition.evaluation_type == EvaluationType::Alpha) && (evaluation <= alpha) {
                return Some((best_move, alpha));
            }
            if (transposition.evaluation_type == EvaluationType::Beta) && (evaluation >= beta) {
                return Some((best_move, beta));
            }
        }
        None
    }
}
