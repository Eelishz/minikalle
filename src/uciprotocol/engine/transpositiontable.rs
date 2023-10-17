use std::mem::size_of;

#[derive(Clone, Copy, PartialEq)]
pub enum EvaluationType {
    Exact,
    Alpha,
    Beta,
}

#[derive(Clone, Copy)]
pub struct Transposition {
    pub key: u64,
    pub evaluation: i32,
    pub depth_left: u8,
    pub evaluation_type: EvaluationType,
}

impl Transposition {
    pub fn new() -> Transposition {
        Transposition {
            key: 0,
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
        evaluation: i32,
        depth_left: u8,
        evaluation_type: EvaluationType,
    ) {
        let index = self.index(&key);
        let entry = Transposition {
            key,
            evaluation,
            depth_left,
            evaluation_type,
        };
        self.transpositions[index] = entry;
    }

    pub fn get(&self, key: &u64) -> Option<Transposition> {
        let index = self.index(key);
        let transposition = self.transpositions[index];
        if &transposition.key == key {
            Some(transposition)
        } else {
            None
        }
    }
}
