use std::mem::size_of;

const DESIRED_SIZE_MB: u32 = 512;
const TT_ENTRY_SIZE_IN_BYTES: usize = size_of::<Transposition>();
const DESIRED_SIZE_IN_BYTES: u32 = DESIRED_SIZE_MB * 1024 * 1024;
const NUM_ENTRIES: usize = (DESIRED_SIZE_IN_BYTES as usize) / TT_ENTRY_SIZE_IN_BYTES;

#[derive(Clone, Copy)]
struct Transposition {
    key: u64,
    evaluation: i32,
    depth_left: u8,
}

impl Transposition {
    pub fn new() -> Transposition {
        Transposition {
            key: 0,
            evaluation: 0,
            depth_left: 0,
        }
    }
}

pub struct TranspositionTable {
    transpositions: Vec<Transposition>,
    count: u64,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            transpositions: vec![Transposition::new(); NUM_ENTRIES],
            count: NUM_ENTRIES as u64,
        }
    }

    fn index(&self, key: u64) -> usize {
        (key % self.count) as usize
    }

    pub fn insert(&mut self, key: u64, evaluation: i32, depth_left: u8) {
        let index = self.index(key);
        let entry = Transposition {
            key,
            evaluation,
            depth_left,
        };
        self.transpositions[index] = entry;
    }

    pub fn get(&self, key: u64) -> Option<(i32, u8)> {
        let index = self.index(key);
        let transposition = self.transpositions[index];
        if transposition.key == key {
            Some((transposition.evaluation, transposition.depth_left))
        } else {
            None
        }
    }
}
