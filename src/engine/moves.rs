use crate::{
    engine::Pos,
    engine::{anchors::CrossChecks, gaddag::Gaddag},
    game::{
        Game,
        board::{BOARD_TILES, Board},
        rack::Rack,
        tile::Tile,
    },
};
use fst::raw::CompiledAddr;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Move {
    pub tiles_used: u16, // bitmask of used tiles
    pub tiles_data: [(u8, PlayedTile); BOARD_TILES],
    pub pos: Pos,
    pub direction: Direction,
    pub score: u32,
}

impl Move {
    pub fn rack_tiles_count(&self) -> usize {
        let mut count = 0;
        let mut bits = self.tiles_used;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            if matches!(self.tiles_data[i].1, PlayedTile::FromRack(_)) {
                count += 1;
            }
            bits &= bits - 1;
        }
        count
    }

    pub fn to_display(&self) -> (String, Pos, Vec<PlayedTile>) {
        let mut word = Vec::with_capacity(15);
        let mut tiles = Vec::with_capacity(15);

        for i in 0..BOARD_TILES {
            if self.tiles_used & (1 << i) != 0 {
                word.push(self.tiles_data[i].0);
                tiles.push(self.tiles_data[i].1);
            }
        }

        let word_start_idx = self.tiles_used.trailing_zeros() as usize;
        let word_start = match self.direction {
            Direction::Horizontal => Pos::new(self.pos.row, word_start_idx),
            Direction::Vertical => Pos::new(word_start_idx, self.pos.col),
        };

        (String::from_utf8_lossy(&word).to_string(), word_start, tiles)
    }

    pub fn get_word_string(&self) -> String {
        let mut word = Vec::with_capacity(15);
        for i in 0..BOARD_TILES {
            if self.tiles_used & (1 << i) != 0 {
                word.push(self.tiles_data[i].0);
            }
        }
        String::from_utf8_lossy(&word).to_string()
    }
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

#[derive(Debug, Clone, Copy)]
enum ExploreDir {
    Left = -1,
    Right = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayedTile {
    FromRack(Tile),
    FromBoard, // don't need to know it
}

// this might be overkill lol
struct MoveBuffer {
    data: [(u8, PlayedTile); BOARD_TILES],
    used: u16,
}

impl MoveBuffer {
    fn new() -> Self {
        MoveBuffer {
            data: [(0, PlayedTile::FromBoard); BOARD_TILES],
            used: 0,
        }
    }

    fn set(&mut self, pos: usize, letter: u8, tile: PlayedTile) {
        self.data[pos] = (letter, tile);
        self.used |= 1 << pos;
    }

    fn unset(&mut self, pos: usize) {
        self.used &= !(1 << pos);
    }

    fn has_played_tile(&self) -> bool {
        let mut bits = self.used;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            if matches!(self.data[i].1, PlayedTile::FromRack(_)) {
                return true;
            }
            bits &= bits - 1;
        }
        false
    }
}

pub struct MoveGenerator {
    pub gaddag: Gaddag,
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
        let start = std::time::Instant::now();
        let (horizontal_anchors, horizontal_allowed_ext) = self.find_anchors(&game.board, &Direction::Horizontal);
        let elapsed = start.elapsed();
        let (vertical_anchors, vertical_allowed_ext) = self.find_anchors(&game.board, &Direction::Vertical);
        let elapsed2 = start.elapsed() - elapsed;

        let mut moves = Vec::new();
        for anchor in &horizontal_anchors {
            self.goorgoon(game, anchor, &mut moves, &vertical_allowed_ext, Direction::Horizontal);
        }
        let elapsed3 = start.elapsed() - elapsed - elapsed2;
        for anchor in &vertical_anchors {
            self.goorgoon(game, anchor, &mut moves, &horizontal_allowed_ext, Direction::Vertical);
        }
        let elapsed4 = start.elapsed() - elapsed - elapsed2 - elapsed3;

        println!(
            "Times: anchors H: {:.2?}, V: {:.2?}, gen H: {:.2?}, gen V: {:.2?}",
            elapsed, elapsed2, elapsed3, elapsed4
        );
        moves
    }

    pub fn goorgoon(&self, game: &Game, anchor: &Pos, mut moves: &mut Vec<Move>, cross_checks: &CrossChecks, direction: Direction) {
        // before recursion, get suffix:
        // _ _ x R A I N _ -> RAIN
        let mut move_buffer = MoveBuffer::new();
        let mut current_pos = *anchor;

        let suffix_dir = match direction {
            Direction::Horizontal => (0, 1),
            Direction::Vertical => (1, 0),
        };

        while let Some(next_pos) = current_pos.offset(suffix_dir.0, suffix_dir.1) {
            if let Some(tile) = game.board.get_tile(next_pos) {
                let board_idx = match direction {
                    Direction::Horizontal => next_pos.col,
                    Direction::Vertical => next_pos.row,
                };
                move_buffer.set(board_idx, tile.to_byte(), PlayedTile::FromBoard);
                current_pos = next_pos;
            } else {
                break;
            }
        }

        // we start from the suffix node, which will always be valid. hopefully.
        let mut current_node = self.gaddag.0.as_fst().root().addr();
        for i in (0..BOARD_TILES).rev() {
            if move_buffer.used & (1 << i) != 0 {
                let byte = move_buffer.data[i].0;
                let node = self.gaddag.0.as_fst().node(current_node);
                if let Some(transition_idx) = node.find_input(byte) {
                    current_node = node.transition_addr(transition_idx);
                } else {
                    return;
                }
            }
        }

        let mut rack = game.rack.clone();

        self.explore(
            &game.board,
            &mut rack,
            anchor,
            0,
            ExploreDir::Left,
            &mut move_buffer,
            current_node,
            &mut moves,
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
        move_buffer: &mut MoveBuffer,
        current_node: CompiledAddr,
        moves: &mut Vec<Move>,
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

        let board_idx = match direction {
            Direction::Horizontal => current_pos.col,
            Direction::Vertical => current_pos.row,
        };

        if let Some(tile) = board.get_tile(current_pos) {
            if let Some(next_node) = self.gaddag.can_next(current_node, tile.to_char()) {
                move_buffer.set(board_idx, tile.to_byte(), PlayedTile::FromBoard);

                self.explore(
                    board,
                    rack,
                    anchor,
                    offset + explore_dir as i8,
                    explore_dir,
                    move_buffer,
                    next_node,
                    moves,
                    cross_checks,
                    direction,
                );

                move_buffer.unset(board_idx);
            }
            return;
        }

        if self.gaddag.is_terminal(current_node) && move_buffer.has_played_tile() {
            moves.push(Move {
                tiles_used: move_buffer.used,
                tiles_data: move_buffer.data,
                pos: Pos {
                    row: anchor.row,
                    col: anchor.col,
                },
                direction,
                score: 0,
            });
        }

        let cross_check_mask = cross_checks[current_pos.row][current_pos.col]; // crosschecks for this tile
        for letter in self.gaddag.valid_children_char(current_node) {
            // if we hit the delimiter, we start looking right instead
            if letter == super::gaddag::DELIMITER as char {
                if let Some(delimiter_node) = self.gaddag.can_next(current_node, letter) {
                    self.explore(
                        board,
                        rack,
                        anchor,
                        1, // start at 1 to the right
                        ExploreDir::Right,
                        move_buffer,
                        delimiter_node,
                        moves,
                        cross_checks,
                        direction,
                    );
                }
                continue;
            }

            // check the cross checks
            let letter_byte = letter as u8;
            if cross_check_mask & 1 << (letter_byte - b'A') == 0 {
                continue;
            }

            // check rack
            if let Some(tile) = rack.has_letter(letter_byte) {
                rack.remove_tile(tile);

                let played_tile = match tile {
                    Tile::Blank(_) => Tile::Blank(Some(letter_byte)),
                    _ => tile,
                };

                move_buffer.set(board_idx, letter_byte, PlayedTile::FromRack(played_tile));

                if let Some(next_node) = self.gaddag.can_next(current_node, letter) {
                    self.explore(
                        board,
                        rack,
                        anchor,
                        offset + explore_dir as i8,
                        explore_dir,
                        move_buffer,
                        next_node,
                        moves,
                        cross_checks,
                        direction,
                    );
                }

                move_buffer.unset(board_idx);
                rack.add_tile(tile);
            }
        }
    }
}
