use fst::Set;
use fst::raw::CompiledAddr;
use std::collections::BTreeSet;

// GOAT
// https://github.com/amedeedaboville/fst-gaddag

/*
e+xplain
xe+plain
pxe+lain
lpxe+ain
alpxe+in
ialpxe+n
nialpxe
*/

#[derive(Debug)]
pub struct Gaddag(pub Set<Vec<u8>>);
pub const DELIMITER: u8 = b'+';

impl Gaddag {
    pub fn from_wordlist(path: &str) -> Self {
        use std::fs::File;
        use std::io::{BufReader, prelude::*};

        let mut entries: BTreeSet<Vec<u8>> = BTreeSet::new();

        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let words: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        for word in words {
            let bytes = word.as_bytes();

            // nialpxe
            entries.insert(bytes.to_vec().iter().rev().cloned().collect());

            // lpxe+ain
            let len = bytes.len();
            for i in 1..len {
                let mut entry = Vec::with_capacity(len + 1);

                // lpxe
                entry.extend(bytes[..i].iter().rev());

                // +
                entry.push(DELIMITER);

                // ain
                entry.extend(&bytes[i..]);
                entries.insert(entry);
            }
        }

        Gaddag(Set::from_iter(entries).unwrap())
    }

    ///Returns true if the given word is in the dictionary.
    ///Searches for `^input.rev()$`.
    pub fn contains(&self, word: &[u8]) -> bool {
        let search_vec: Vec<u8> = word.iter().rev().cloned().collect();
        self.0.contains(search_vec)
    }

    ///Attempts to follow the node in the GADDAG, and returns the next node.
    pub fn can_next(&self, node_addr: CompiledAddr, next: u8) -> Option<CompiledAddr> {
        let current_node = self.0.as_fst().node(node_addr);
        if let Some(i) = current_node.find_input(next) {
            Some(current_node.transition(i).addr)
        } else {
            None
        }
    }

    pub fn is_terminal(&self, node_addr: CompiledAddr) -> bool {
        self.0.as_fst().node(node_addr).is_final()
    }

    // holy speed
    pub fn for_each_child<F>(&self, node_addr: CompiledAddr, mut f: F)
    where
        F: FnMut(u8) -> bool,
    {
        let node = self.0.as_fst().node(node_addr);
        for i in 0..node.len() {
            let transition = node.transition(i);
            if !f(transition.inp) {
                break;
            }
        }
    }
}
