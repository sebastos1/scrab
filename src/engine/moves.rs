use std::{collections::HashMap, fmt::Debug};

use fst::raw::CompiledAddr;

use crate::{
    engine::gaddag::Gaddag,
    game::{Game, board::Board, rack::Rack, tile::Tile},
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
    pub gaddag: Gaddag,
}

pub struct DebugThings {
    pub horizontal_anchors: Vec<(usize, usize)>,
    pub vertical_anchors: Vec<(usize, usize)>,
    pub horizontal_allowed_ext: HashMap<(usize, usize), u32>,
    pub vertical_allowed_ext: HashMap<(usize, usize), u32>,
}

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

impl MoveGenerator {
    pub fn new(gaddag: Gaddag) -> Self {
        MoveGenerator { gaddag }
    }

    // we start from a board, with an otherwise empty slate
    pub fn generate_moves(&self, game: &Game) -> DebugThings {
        let (horizontal_anchors, horizontal_allowed_ext) = self.find_anchors(&game.board, &Direction::Horizontal);
        let (vertical_anchors, vertical_allowed_ext) = self.find_anchors(&game.board, &Direction::Vertical);

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
        // before recursion, get suffix:
        // _ _ x R A I N _ -> RAIN
        let mut suffix = Vec::new();
        for col in (anchor_col + 1)..game.board.width() {
            if let Some(tile) = game.board.get_tile(anchor_row, col) {
                suffix.push(tile.to_byte());
            } else {
                break;
            }
        }

        // we start from the suffix node, which will always be valid. hopefully.
        let mut current_node = self.gaddag.0.as_fst().root().addr();

        for &byte in suffix.iter().rev() {
            // N, I, A, R for "RAIN"
            let node = self.gaddag.0.as_fst().node(current_node);
            if let Some(transition_idx) = node.find_input(byte) {
                current_node = node.transition_addr(transition_idx);
            } else {
                println!("Could not traverse suffix - invalid word on board");
                return;
            }
        }

        self.explore_left(&game.board, &game.rack, anchor_row, anchor_col, Vec::new(), current_node);
    }

    fn explore_left(&self, board: &Board, rack: &Rack, row: usize, col: usize, tiles_placed: Vec<Tile>, current_node: CompiledAddr) {
        if let Some(tile) = board.get_tile(row, col) {}

        for letter in self.gaddag.valid_children_char(current_node) {
            if letter == super::gaddag::DELIMITER as char {
                // if we hit the delimiter, we start looking right instead
                println!("Hit delimiter at ({}, {}), would switch to exploring right", row, col);
            } else {
                println!("Can place letter '{}' at ({}, {})", letter, row, col);
            }
        }
        // // we look left. if there's already a tile, we must use it, no checks needed

        // // additionally we set a flag to stop further exploration left when we hit the end of the prefix
    }
}
