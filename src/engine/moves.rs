use std::{collections::HashMap, fmt::Debug};

use crate::{
    engine::gaddag::Gaddag,
    game::{Game, board::Board, tile::Tile},
};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Move {
    word: String,
    row: usize,
    col: usize,
    direction: Direction,
    score: u32,
    tiles_used: Vec<Tile>,
    words_formed: Vec<String>,
}

/*
looking only at the horizontal case for simplification
1. Start from a board and a rack, otherwise empty slate
2. Find all anchor tiles:
    - empty squares to the right or left of a tile, and within board bounds
3. perform horizontal cross checks

(then same vertically)

if we weren't running in parallell, we could just perform and cache the cross checks once when placing

generation (for each anchor): (recursive?)
1. look left, building prefixes by either:
    - pushing an existing tile on the board
    - trying a tile from the rack
        - check if it has a valid gaddag child node
        - check vertical cross checks
        - check board boundary check
2. when the separator is hit, swap to building suffixes, basically doing the same as in step 1 in the other direction.
3. for both, if is_terminal is true, we add that word as valid

(then same vertically)
*/

pub struct MoveGenerator {
    gaddag: Gaddag,
}

pub struct DebugThings {
    pub horizontal_anchors: Vec<(usize, usize)>,
    pub vertical_anchors: Vec<(usize, usize)>,
    pub horizontal_allowed_ext: HashMap<(usize, usize), u32>,
    pub vertical_allowed_ext: HashMap<(usize, usize), u32>,
}

impl MoveGenerator {
    pub fn new(gaddag: Gaddag) -> Self {
        MoveGenerator { gaddag }
    }

    // we start from a board, with an otherwise empty slate
    pub fn generate_moves(&self, game: &Game) -> DebugThings {
        // first find all anchor tiles:
        // empty tiles next to an existing tile, within board bounds, which have ANY valid letters

        // so these guys get:
        /*
                    |
            - X T E N -
                |

            horizontal anchors will be -
            vertical anchors would be |
            vertical allowed ext would be "o" for the T and "a" for the N, or whatever else makes
            a valid word using JUST one letter added

            separating these will let us check only the horizontal_allowed_ext when placing vertical words
        */
        let (horizontal_anchors, horizontal_allowed_ext) = self.find_anchors(&game.board, &Direction::Horizontal);
        let (vertical_anchors, vertical_allowed_ext) = self.find_anchors(&game.board, &Direction::Vertical);

        /*
            both approaches use the total set of anchors

            We keep both the start and ending anchor points:
            _ x R A I N x
            1. The first anchor point ALWAYS gets a tile (which is cleaner), and grows outward in the standard way.
            _ x R A I N x
            ^
            2. The second anchor point STOPS after a chain of existing tiles, since anything beyond that point will already be handled by the anchor point before those (or will be a wall, which is fine):
            _ _ R A I N x
                        ^
            3. This also works in chains, even with gaps (pretend RM is a valid word):
            _ x R A I N x _ x R M x _
            ^         ^   ^     ^
            The first case handles TRAIN, and TRAINSTORM (scary)
                        |   |     |
                        The second case handles RAINS, RAINED, and importantly: RAINSTORM
                            |     |
                            The third case handles ARM, HARM ARMED
                                |
                                The fourth case handles RME (real word, look it up)
        */

        for &(row, col) in &horizontal_anchors {
            self.goorgoon(game, row, col);
        }

        DebugThings {
            horizontal_anchors,
            vertical_anchors,
            horizontal_allowed_ext,
            vertical_allowed_ext,
        }
    }

    pub fn goorgoon(&self, game: &Game, anchor_row: usize, anchor_col: usize) {
        let mut prefix = Vec::new();
        for col in (0..anchor_col).rev() {
            if let Some(tile) = game.board.get_tile(anchor_row, col) {
                prefix.insert(0, tile.to_byte());
            } else {
                break;
            }
        }

        let mut suffix = Vec::new();
        for col in (anchor_col + 1)..game.board.width() {
            if let Some(tile) = game.board.get_tile(anchor_row, col) {
                suffix.push(tile.to_byte());
            } else {
                break;
            }
        }
        let prefix_str: String = prefix.iter().map(|&b| b as char).collect();
        let suffix_str: String = suffix.iter().map(|&b| b as char).collect();

        println!("Anchor ({}, {}) - prefix: '{}', suffix: '{}'", anchor_row, anchor_col, prefix_str, suffix_str);

        for tile in game.rack.tiles() {
            let mut word_bytes = prefix.clone();
            word_bytes.push(tile.to_byte());
            word_bytes.extend(&suffix);

            let word_str: String = word_bytes.iter().map(|&b| b as char).collect();

            if self.gaddag.contains_u8(&word_bytes) {
                println!("  Valid word with {}: {}", tile.to_char(), word_str);
            }
        }
    }

    // finds both anchors and cross checks  from that direction
    pub fn find_anchors(&self, board: &Board, direction: &Direction) -> (Vec<(usize, usize)>, HashMap<(usize, usize), u32>) {
        if board.is_empty() {
            return (vec![(7, 7)], HashMap::new()); // todo make all allowed?
        }
        let mut anchors = Vec::new();

        // bitsets for valid letters
        let mut cross_checks: HashMap<(usize, usize), u32> = HashMap::new();

        // horizontally
        for row in 0..board.height() {
            for col in 0..board.width() {
                if board.get_tile(row, col).is_some() {
                    continue;
                }

                let (has_prev, has_next) = match direction {
                    Direction::Horizontal => (col > 0 && board.get_tile(row, col - 1).is_some(), col < board.width() - 1 && board.get_tile(row, col + 1).is_some()),
                    Direction::Vertical => (row > 0 && board.get_tile(row - 1, col).is_some(), row < board.height() - 1 && board.get_tile(row + 1, col).is_some()),
                };

                if has_prev || has_next {
                    let mut prefix = Vec::new();
                    let mut suffix = Vec::new();
                    anchors.push((row, col));

                    match direction {
                        Direction::Vertical => {
                            // up/down
                            for i in (0..row).rev() {
                                if let Some(tile) = board.get_tile(i, col) {
                                    prefix.insert(0, tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                            for i in (row + 1)..board.height() {
                                if let Some(tile) = board.get_tile(i, col) {
                                    suffix.push(tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                        }
                        Direction::Horizontal => {
                            // left/right
                            for i in (0..col).rev() {
                                if let Some(tile) = board.get_tile(row, i) {
                                    prefix.insert(0, tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                            for i in (col + 1)..board.width() {
                                if let Some(tile) = board.get_tile(row, i) {
                                    suffix.push(tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                        }
                    }

                    let mut valid_letters = 0u32;
                    for c in b'A'..=b'Z' {
                        let mut bytes = prefix.clone();
                        bytes.push(c);
                        bytes.extend(suffix.iter());
                        if self.gaddag.contains_u8(&bytes) {
                            valid_letters |= 1 << (c - b'A');
                        }
                    }
                    if valid_letters != 0 {
                        cross_checks.insert((row, col), valid_letters);
                    }
                }
            }
        }

        (anchors, cross_checks)
    }
}
