use std::collections::{HashSet, VecDeque};

pub struct DeadNonceList {
    set: HashSet<u64>,
    queue: VecDeque<u64>,
    max_len: usize,
}

impl DeadNonceList {
    pub fn new(max_len: usize) -> DeadNonceList {
        DeadNonceList {
            set: HashSet::new(),
            queue: VecDeque::new(),
            max_len: max_len,
        }
    }

    pub fn insert(&mut self, nonce: u64) {
        if self.set.contains(&nonce) {
            return;
        }

        self.set.insert(nonce);
        self.queue.push_back(nonce);
    }

    pub fn contains(&self, nonce: u64) -> bool {
        self.set.contains(&nonce)
    }

    pub fn clean(&mut self) {
        while self.queue.len() > self.max_len {
            let nonce = self.queue.pop_front().unwrap();
            self.set.remove(&nonce);
        }
    }
}