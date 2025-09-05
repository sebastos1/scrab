use crate::{
    engine::{anchors::CrossChecks, gaddag::Gaddag},
    game::{Game, board::Board, rack::Rack, tile::Tile},
    util::Pos,
};
use fst::raw::CompiledAddr;
use std::{collections::VecDeque, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub word: String,
    pub pos: Pos,
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

/*
    This is probably not reflecting the current implementation

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
    pub fn generate_moves(&self, game: &Game) -> Vec<Move> {
        let (horizontal_anchors, horizontal_allowed_ext) = self.find_anchors(&game.board, &Direction::Horizontal);
        let (vertical_anchors, vertical_allowed_ext) = self.find_anchors(&game.board, &Direction::Vertical);

        let mut moves = Vec::new();
        for anchor in &horizontal_anchors {
            self.goorgoon(game, anchor, &mut moves, &vertical_allowed_ext, Direction::Horizontal);
        }
        for anchor in &vertical_anchors {
            self.goorgoon(game, anchor, &mut moves, &horizontal_allowed_ext, Direction::Vertical);
        }

        moves
    }

    pub fn goorgoon(&self, game: &Game, anchor: &Pos, mut moves: &mut Vec<Move>, cross_checks: &CrossChecks, direction: Direction) {
        // before recursion, get suffix:
        // _ _ x R A I N _ -> RAIN
        let mut suffix = Vec::new();
        let mut current_pos = *anchor;

        let suffix_dir = match direction {
            Direction::Horizontal => (0, 1),
            Direction::Vertical => (1, 0),
        };

        while let Some(next_pos) = current_pos.offset(suffix_dir.0, suffix_dir.1) {
            if let Some(tile) = game.board.get_tile(next_pos) {
                suffix.push(tile.to_byte());
                current_pos = next_pos;
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
                return;
            }
        }

        // in-place swag
        let mut word = suffix;
        let mut played_tiles = VecDeque::new();
        let mut rack = game.rack.clone();

        self.explore(
            &game.board,
            &mut rack,
            anchor,
            0,
            ExploreDir::Left,
            &mut played_tiles,
            current_node,
            &mut moves,
            &mut word,
            *anchor,
            &cross_checks,
            direction,
        );
    }

    fn explore(
        &self,
        board: &Board,
        rack: &mut Rack,
        anchor: &Pos,
        offset: i8,
        explore_dir: ExploreDir,
        played_tiles: &mut VecDeque<PlayedTile>,
        current_node: CompiledAddr,
        moves: &mut Vec<Move>,
        word: &mut Vec<u8>,
        word_start: Pos,
        cross_checks: &CrossChecks, // use the opposite direction's cross checks
        direction: Direction,
    ) {
        let offset_dir = match direction {
            Direction::Horizontal => (0, 1),
            Direction::Vertical => (1, 0),
        };

        let current_pos = match anchor.offset(offset_dir.0 * offset as isize, offset_dir.1 * offset as isize) {
            Some(pos) => pos,
            None => return,
        };

        if let Some(tile) = board.get_tile(current_pos) {
            if let Some(next_node) = self.gaddag.can_next(current_node, tile.to_char()) {
                let new_word_start;
                match explore_dir {
                    ExploreDir::Left => {
                        word.insert(0, tile.to_byte());
                        played_tiles.push_front(PlayedTile::FromBoard);
                        new_word_start = current_pos;
                    }
                    ExploreDir::Right => {
                        word.push(tile.to_byte());
                        played_tiles.push_back(PlayedTile::FromBoard);
                        new_word_start = word_start;
                    }
                }

                self.explore(
                    board,
                    rack,
                    anchor,
                    offset + explore_dir as i8,
                    explore_dir,
                    played_tiles,
                    next_node,
                    moves,
                    word,
                    new_word_start,
                    cross_checks,
                    direction,
                );

                match explore_dir {
                    ExploreDir::Left => {
                        word.remove(0);
                        played_tiles.pop_front();
                    }
                    ExploreDir::Right => {
                        word.pop();
                        played_tiles.pop_back();
                    }
                }
            }
            return;
        }

        if self.gaddag.is_terminal(current_node) && played_tiles.iter().any(|t| matches!(t, PlayedTile::FromRack(_))) {
            let move_obj = Move {
                word: String::from_utf8_lossy(word).to_string(),
                pos: word_start,
                direction,
                score: 0,
                tiles_used: played_tiles.iter().cloned().collect(),
                words_formed: vec![],
            };
            moves.push(move_obj);
        }

        for letter in self.gaddag.valid_children_char(current_node) {
            // if we hit the delimiter, we start looking right instead
            if letter == super::gaddag::DELIMITER as char {
                if let Some(delimiter_node) = self.gaddag.can_next(current_node, letter) {
                    self.explore(
                        board,
                        rack,
                        anchor,
                        1,
                        ExploreDir::Right,
                        played_tiles,
                        delimiter_node,
                        moves,
                        word,
                        word_start,
                        cross_checks,
                        direction,
                    );
                }
                continue;
            }

            // check rack
            if let Some(tile) = rack.has_letter(letter as u8) {
                // check crosschecks
                let cross_check_mask = cross_checks[current_pos.row][current_pos.col];
                let letter_bit = 1 << (letter as u8 - b'A');
                if cross_check_mask & letter_bit == 0 {
                    continue;
                }

                rack.remove_tile(tile);

                let letter_byte = letter as u8;
                let new_word_start;
                match explore_dir {
                    ExploreDir::Left => {
                        word.insert(0, letter_byte);
                        played_tiles.push_front(PlayedTile::FromRack(tile));
                        new_word_start = current_pos;
                    }
                    ExploreDir::Right => {
                        word.push(letter_byte);
                        played_tiles.push_back(PlayedTile::FromRack(tile));
                        new_word_start = word_start;
                    }
                }

                if let Some(next_node) = self.gaddag.can_next(current_node, letter) {
                    self.explore(
                        board,
                        rack,
                        anchor,
                        offset + explore_dir as i8,
                        explore_dir,
                        played_tiles,
                        next_node,
                        moves,
                        word,
                        new_word_start,
                        cross_checks,
                        direction,
                    );
                }

                match explore_dir {
                    ExploreDir::Left => {
                        word.remove(0);
                        played_tiles.pop_front();
                    }
                    ExploreDir::Right => {
                        word.pop();
                        played_tiles.pop_back();
                    }
                }
                rack.add_tile(tile);
            }
        }
    }
}
