use fst::automaton::{Automaton, Str, Subsequence};
pub use fst::raw::{CompiledAddr, Node};
use fst::{IntoStreamer, Set};
use std::collections::BTreeSet;
use std::iter;

// GOAT
// https://github.com/amedeedaboville/fst-gaddag

#[derive(Debug)]
pub struct Gaddag(pub Set<Vec<u8>>);
pub const DELIMITER: u8 = b'+';

/*
e+xplain
xe+plain
pxe+lain
lpxe+ain
alpxe+in
ialpxe+n
nialpxe
*/

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
    pub fn contains(&self, word: &str) -> bool {
        let search_vec: Vec<u8> = (*word.chars().rev().collect::<String>().as_bytes()).to_vec();
        self.0.contains(search_vec)
    }

    pub fn contains_u8(&self, word: &[u8]) -> bool {
        let search_vec: Vec<u8> = word.iter().rev().cloned().collect();
        self.0.contains(search_vec)
    }

    ///Returns all the words that start with a given prefix.
    ///Searches for `^input.rev(),.*`
    pub fn starts_with(&self, input: &str) -> Vec<String> {
        let search_val: String = input.chars().rev().chain(iter::once(DELIMITER as char)).collect();
        let matcher = Str::new(&search_val).starts_with();
        self.search_fst(matcher)
    }

    ///Returns all the words that end with a given suffix.
    ///Searches for `^input.rev()[^,]*` . That is, the reversed input plus
    ///any sequence that doesn't include the separator.
    pub fn ends_with(&self, input: &str) -> Vec<String> {
        //looks up input.rev(), then filters down to things that do not have a comma
        let search_val: String = input.chars().rev().collect();

        let delimiter_str = char::from(DELIMITER).to_string();
        let matcher = Str::new(&search_val)
            .starts_with()
            .intersection(Subsequence::new(&delimiter_str).complement());

        let stream = self.0.search(matcher).into_stream();
        stream.into_strs().unwrap().iter().map(|w| Self::demangle_item(w)).collect()
    }

    ///Returns all the words that contain the input anywhere in them.
    ///Searches for `^input.rev().*`
    pub fn substring(&self, input: &str) -> Vec<String> {
        let search_val: String = input.chars().rev().collect();
        let matcher = Str::new(&search_val).starts_with();
        self.search_fst(matcher)
    }

    ///Turns the GADDAG row for a word back into that word.
    ///For example GINT+BOA will demangle to BOATING.
    fn demangle_item(item: &str) -> String {
        if let Some(idx) = item.find(DELIMITER as char) {
            item[0..idx].chars().rev().chain(item[(idx + 1)..].chars()).collect()
        } else {
            item.chars().rev().collect()
        }
    }

    ///Applies a fst matcher to the Gaddag, and returns all the words that
    ///match.
    fn search_fst(&self, matcher: impl Automaton) -> Vec<String> {
        self.0
            .search(matcher)
            .into_stream()
            .into_strs()
            .unwrap()
            .iter()
            .map(|w| Self::demangle_item(w))
            .collect()
    }

    ///Returns the node address for a prefix in the dictionary.
    ///This means the input doesn't have to be a full word, but has to be a prefix
    ///of a word in the dictionary. Will return None if the word doesn't exist in the
    ///dictionary.
    pub fn node_for_prefix(&self, prefix: &str) -> Option<CompiledAddr> {
        let mut current_node: Node = self.0.as_fst().root();
        for byte in prefix.bytes() {
            if let Some(transition_idx) = current_node.find_input(byte) {
                let next_node = self.0.as_fst().node(current_node.transition_addr(transition_idx));
                current_node = next_node;
            } else {
                return None;
            }
        }
        Some(current_node.addr())
    }

    ///Attempts to follow the node in the GADDAG, and returns the next node.
    pub fn can_next(&self, node_addr: CompiledAddr, next: char) -> Option<CompiledAddr> {
        let current_node = self.0.as_fst().node(node_addr);
        for byte in next.to_string().bytes() {
            if let Some(i) = current_node.find_input(byte) {
                return Some(current_node.transition(i).addr);
            }
        }
        None
    }

    pub fn is_terminal(&self, node_addr: CompiledAddr) -> bool {
        self.0.as_fst().node(node_addr).is_final()
    }

    pub fn valid_children_char(&self, node_addr: CompiledAddr) -> Vec<char> {
        let mut valid_chars = Vec::new();
        let node = self.0.as_fst().node(node_addr);
        for i in 0..node.len() {
            let transition = node.transition(i);
            let child_letter = transition.inp as char;
            valid_chars.push(child_letter);
        }
        valid_chars
    }
}
