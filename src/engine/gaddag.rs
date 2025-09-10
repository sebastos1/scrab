use fst::Set;
use fst::raw::CompiledAddr;
use lazy_static::lazy_static;
use std::collections::BTreeSet;

lazy_static! {
    pub static ref GADDAG: Gaddag = {
        if let Ok(gaddag) = Gaddag::load("wordlists/CSW24.fst") {
            gaddag
        } else {
            let gaddag = Gaddag::from_wordlist("wordlists/CSW24.txt");
            gaddag.save("wordlists/CSW24.fst").unwrap();
            gaddag
        }
    };
}

// from https://github.com/amedeedaboville/fst-gaddag
#[derive(Debug)]
pub struct Gaddag(pub Set<Vec<u8>>);
pub const DELIMITER: u8 = b'+';

impl Gaddag {
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        std::fs::write(path, self.0.as_fst().as_bytes())
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Gaddag(Set::new(std::fs::read(path)?)?))
    }

    pub fn from_wordlist(path: &str) -> Self {
        use std::io::BufRead;
        let file = std::fs::File::open(path).unwrap();
        let reader = std::io::BufReader::new(file);
        let words: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
        let mut entries: BTreeSet<Vec<u8>> = BTreeSet::new();
        for word in words {
            let bytes = word.as_bytes();

            // full reversed word
            entries.insert(bytes.to_vec().iter().rev().cloned().collect()); // nialpxe

            // lpxe+ain
            let len = bytes.len();
            for i in 1..len {
                let mut entry = Vec::with_capacity(len + 1);

                entry.extend(bytes[..i].iter().rev()); // lpxe
                entry.push(DELIMITER); // +
                entry.extend(&bytes[i..]); // ain
                // => lpxe+ain

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

    pub fn node_at(&self, node_addr: CompiledAddr) -> fst::raw::Node {
        self.0.as_fst().node(node_addr)
    }

    ///Attempts to follow the node in the GADDAG, and returns the next node.
    pub fn can_next(&self, node_addr: CompiledAddr, next: u8) -> Option<CompiledAddr> {
        let current_node = self.node_at(node_addr);
        if let Some(i) = current_node.find_input(next) {
            Some(current_node.transition(i).addr)
        } else {
            None
        }
    }

    pub fn is_terminal(&self, node_addr: CompiledAddr) -> bool {
        self.node_at(node_addr).is_final()
    }

    // holy speed
    pub fn for_each_child<F>(&self, node_addr: CompiledAddr, mut f: F)
    where
        F: FnMut(u8) -> bool,
    {
        let node = self.node_at(node_addr);
        for i in 0..node.len() {
            let transition = node.transition(i);
            if !f(transition.inp) {
                break;
            }
        }
    }
}
