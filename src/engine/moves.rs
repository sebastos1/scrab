use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

use fst::raw::CompiledAddr;

use crate::{
    engine::{anchors::Anchor, gaddag::Gaddag},
    game::{Game, board::Board, rack::Rack, tile::Tile},
};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub word: String,
    pub pos: (usize, usize),
    pub direction: Direction,
    pub score: u32,
    pub tiles_used: Vec<PlayedTile>,
    pub words_formed: Vec<String>,
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

#[derive(Debug, Clone, Copy)]
enum ExploreDir {
    Left = -1,
    Right = 1,
}

#[derive(Debug, Clone)]
pub enum PlayedTile {
    FromRack(Tile),
    FromBoard, // don't need to know it
}

pub struct DebugThings {
    pub horizontal_anchors: Vec<Anchor>,
    pub vertical_anchors: Vec<Anchor>,
    pub horizontal_allowed_ext: HashMap<Anchor, u32>,
    pub vertical_allowed_ext: HashMap<Anchor, u32>,
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
    pub fn generate_moves(&self, game: &Game) -> (DebugThings, Vec<Move>) {
        let (horizontal_anchors, horizontal_allowed_ext) = self.find_anchors(&game.board, &Direction::Horizontal);
        let (vertical_anchors, vertical_allowed_ext) = self.find_anchors(&game.board, &Direction::Vertical);

        let mut moves = Vec::new();
        for anchor in &horizontal_anchors {
            self.goorgoon(game, anchor, &mut moves);
        }
        for mov in &moves {
            println!("Generated move: {:?}, starting at {:?}", mov.word, mov.pos);
        }

        (
            DebugThings {
                horizontal_anchors,
                vertical_anchors,
                horizontal_allowed_ext,
                vertical_allowed_ext,
            },
            moves,
        )
    }

    pub fn goorgoon(&self, game: &Game, anchor: &Anchor, mut moves: &mut Vec<Move>) {
        // before recursion, get suffix:
        // _ _ x R A I N _ -> RAIN
        let mut suffix = Vec::new();
        for col in (anchor.col + 1)..game.board.width() {
            if let Some(tile) = game.board.get_tile(anchor.row, col) {
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

        println!("string from utf8 suffix: {:?}", String::from_utf8(suffix.clone()));

        self.explore(
            &game.board,
            &game.rack,
            anchor,
            0,
            ExploreDir::Left,
            VecDeque::new(),
            current_node,
            &mut moves,
            String::from_utf8(suffix).unwrap_or_default(),
            (anchor.row, anchor.col),
        );
    }

    fn explore(
        &self,
        board: &Board,
        rack: &Rack,
        anchor: &Anchor,
        offset: i8,
        explore_dir: ExploreDir,
        tiles_placed: VecDeque<PlayedTile>,
        current_node: CompiledAddr,
        moves: &mut Vec<Move>,
        word: String,
        word_start: (usize, usize),
    ) {
        let col = anchor.col as i8 + offset;
        let row = anchor.row as i8; // todo
        if col < 0 || col >= board.width() as i8 {
            return;
        }
        let col = col as usize;
        let row = row as usize;

        if let Some(tile) = board.get_tile(row, col) {
            println!("Board has tile '{}' at ({}, {})", tile.to_char(), row, col);
            if let Some(next_node) = self.gaddag.can_next(current_node, tile.to_char()) {
                let new_word = match explore_dir {
                    ExploreDir::Left => format!("{}{}", tile.to_char(), word),
                    ExploreDir::Right => format!("{}{}", word, tile.to_char()),
                };
                let new_word_start = match explore_dir {
                    ExploreDir::Left => (row, col),
                    ExploreDir::Right => word_start,
                };

                let mut new_tiles_placed = tiles_placed.clone();
                match explore_dir {
                    ExploreDir::Left => new_tiles_placed.push_front(PlayedTile::FromBoard),
                    ExploreDir::Right => new_tiles_placed.push_back(PlayedTile::FromBoard),
                }

                self.explore(
                    board,
                    rack,
                    anchor,
                    offset + explore_dir as i8,
                    explore_dir,
                    new_tiles_placed,
                    next_node,
                    moves,
                    new_word,
                    new_word_start,
                );
            }
            return;
        }

        if self.gaddag.is_terminal(current_node) && tiles_placed.iter().any(|t| matches!(t, PlayedTile::FromRack(_))) {
            println!("tiles placed {:?}", tiles_placed);
            let move_obj = Move {
                word: word.clone(),
                pos: word_start,
                direction: Direction::Horizontal,
                score: 0,
                tiles_used: tiles_placed.iter().cloned().collect(),
                words_formed: vec![],
            };
            moves.push(move_obj);
            println!("\n FOUND MOVE: {}", word);
        }

        for letter in self.gaddag.valid_children_char(current_node) {
            // if we hit the delimiter, we start looking right instead
            if letter == super::gaddag::DELIMITER as char {
                println!("Hit delimiter at ({}, {})", row, col);
                if let Some(delimiter_node) = self.gaddag.can_next(current_node, letter) {
                    self.explore(
                        board,
                        rack,
                        anchor,
                        1,
                        ExploreDir::Right,
                        tiles_placed.clone(),
                        delimiter_node,
                        moves,
                        word.clone(),
                        word_start,
                    );
                }
                continue;
            }

            println!("Can place letter '{}' at ({}, {})", letter, anchor.row, anchor.col);

            // check rack
            if let Some(tile) = rack.has_letter(letter as u8) {
                println!("Rack has letter '{}'", tile.to_char());

                // check crosschecks

                println!("Placing letter '{}'", letter);
                let mut new_rack = rack.clone();
                new_rack.remove_tile(tile);
                println!("Rack after removing: {:?}", new_rack);

                if let Some(next_node) = self.gaddag.can_next(current_node, letter) {
                    let mut new_tiles_placed = tiles_placed.clone();
                    match explore_dir {
                        ExploreDir::Left => new_tiles_placed.push_front(PlayedTile::FromRack(tile)),
                        ExploreDir::Right => new_tiles_placed.push_back(PlayedTile::FromRack(tile)),
                    }

                    let new_word = match explore_dir {
                        ExploreDir::Left => format!("{}{}", letter, word),
                        ExploreDir::Right => format!("{}{}", word, letter),
                    };
                    let new_word_start = match explore_dir {
                        ExploreDir::Left => (row, col),
                        ExploreDir::Right => word_start,
                    };

                    self.explore(
                        board,
                        &new_rack,
                        anchor,
                        offset + explore_dir as i8,
                        explore_dir,
                        new_tiles_placed,
                        next_node,
                        moves,
                        new_word,
                        new_word_start,
                    );
                }
            }
        }
    }
}
