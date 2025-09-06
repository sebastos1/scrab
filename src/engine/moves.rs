use crate::{
    GADDAG,
    engine::{Pos, anchors::CrossChecks},
    game::{
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
    pub score: u16,
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
        let mut word = Vec::with_capacity(BOARD_TILES);
        let mut tiles = Vec::with_capacity(BOARD_TILES);

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
        let mut word = Vec::with_capacity(BOARD_TILES);
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
    used_bits: u16,
    played_tiles_count: u8, // has a tile from rack
}

impl MoveBuffer {
    fn new() -> Self {
        MoveBuffer {
            data: [(0, PlayedTile::FromBoard); BOARD_TILES],
            used_bits: 0,
            played_tiles_count: 0,
        }
    }

    fn set(&mut self, pos: usize, letter: u8, tile: PlayedTile) {
        self.data[pos] = (letter, tile);
        self.used_bits |= 1 << pos;
        if matches!(tile, PlayedTile::FromRack(_)) {
            self.played_tiles_count += 1;
        }
    }

    fn unset(&mut self, pos: usize) {
        if matches!(self.data[pos].1, PlayedTile::FromRack(_)) {
            self.played_tiles_count -= 1;
        }
        self.used_bits &= !(1 << pos);
    }

    fn has_played_tile(&self) -> bool {
        self.played_tiles_count > 0
    }
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

// per-turn fields in this guy
pub struct MoveGenerator {
    pub board: Board,
    pub rack: Rack,
}

impl MoveGenerator {
    pub fn new(board: Board, rack: Rack) -> Self {
        Self { board, rack }
    }

    // we start from a board, with an otherwise empty slate
    pub fn generate_moves(&self) -> Vec<Move> {
        let (h_anchors, h_allowed) = super::anchors::find_anchors(&self.board, &Direction::Horizontal);
        let (v_anchors, v_allowed) = super::anchors::find_anchors(&self.board, &Direction::Vertical);

        let mut moves = Vec::new();
        let mut rack = self.rack.clone();
        for anchor_pos in h_anchors {
            self.goorgoon(&mut moves, &mut rack, Direction::Horizontal, anchor_pos, &v_allowed);
        }
        for anchor_pos in v_anchors {
            self.goorgoon(&mut moves, &mut rack, Direction::Vertical, anchor_pos, &h_allowed);
        }

        moves
    }

    pub fn goorgoon(&self, mut moves: &mut Vec<Move>, mut rack: &mut Rack, direction: Direction, anchor_pos: Pos, cross_checks: &CrossChecks) {
        // before recursion, get suffix:
        // _ _ x R A I N _ -> RAIN
        let mut move_buffer = MoveBuffer::new();
        let mut current_pos = anchor_pos;

        let suffix_dir = match direction {
            Direction::Horizontal => (0, 1),
            Direction::Vertical => (1, 0),
        };

        while let Some(next_pos) = current_pos.offset(suffix_dir.0, suffix_dir.1) {
            if let Some(tile) = self.board.get_tile(next_pos) {
                let board_idx = match direction {
                    Direction::Horizontal => next_pos.col,
                    Direction::Vertical => next_pos.row,
                };
                move_buffer.set(board_idx, tile.byte(), PlayedTile::FromBoard);
                current_pos = next_pos;
            } else {
                break;
            }
        }

        let suffix_offset = match direction {
            Direction::Horizontal => current_pos.col - anchor_pos.col + 1,
            Direction::Vertical => current_pos.row - anchor_pos.row + 1,
        };

        // we start from the suffix node, which will always be valid. hopefully.
        let mut current_node = GADDAG.0.as_fst().root().addr();
        for i in (0..BOARD_TILES).rev() {
            if move_buffer.used_bits & (1 << i) != 0 {
                let byte = move_buffer.data[i].0;
                let node = &GADDAG.0.as_fst().node(current_node);
                if let Some(transition_idx) = node.find_input(byte) {
                    current_node = node.transition_addr(transition_idx);
                } else {
                    return;
                }
            }
        }

        self.explore(
            &mut moves,
            &mut move_buffer,
            &mut rack,
            direction,
            anchor_pos,
            &cross_checks,
            suffix_offset,
            0,
            ExploreDir::Left,
            current_node,
        );
    }

    fn explore(
        &self,
        moves: &mut Vec<Move>,
        move_buffer: &mut MoveBuffer,
        rack: &mut Rack,
        direction: Direction,
        anchor_pos: Pos,
        cross_checks: &CrossChecks, // use the opposite direction's cross checks
        suffix_offset: usize,
        offset: i8,
        explore_dir: ExploreDir,
        current_node: CompiledAddr,
    ) {
        let offset_dir = match direction {
            Direction::Horizontal => (0, 1),
            Direction::Vertical => (1, 0),
        };

        let current_pos = match anchor_pos.offset(offset_dir.0 * offset as isize, offset_dir.1 * offset as isize) {
            Some(pos) => pos,
            None => return,
        };

        let board_idx = match direction {
            Direction::Horizontal => current_pos.col,
            Direction::Vertical => current_pos.row,
        };

        if let Some(tile) = self.board.get_tile(current_pos) {
            let byte = tile.byte();
            if let Some(next_node) = GADDAG.can_next(current_node, byte) {
                move_buffer.set(board_idx, byte, PlayedTile::FromBoard);

                self.explore(
                    moves,
                    move_buffer,
                    rack,
                    direction,
                    anchor_pos,
                    cross_checks,
                    suffix_offset,
                    offset + explore_dir as i8,
                    explore_dir,
                    next_node,
                );

                move_buffer.unset(board_idx);
            }
            return;
        }

        if GADDAG.is_terminal(current_node) && move_buffer.has_played_tile() {
            moves.push(Move {
                tiles_used: move_buffer.used_bits,
                tiles_data: move_buffer.data,
                pos: Pos {
                    row: anchor_pos.row,
                    col: anchor_pos.col,
                },
                direction,
                score: 0,
            });
        }

        let cross_check_mask = unsafe { *cross_checks.get_unchecked(current_pos.row).get_unchecked(current_pos.col) };
        GADDAG.for_each_child(current_node, |letter| {
            // if we hit the delimiter, we start looking right instead
            if letter == super::gaddag::DELIMITER {
                if let Some(delimiter_node) = GADDAG.can_next(current_node, letter) {
                    self.explore(
                        moves,
                        move_buffer,
                        rack,
                        direction,
                        anchor_pos,
                        cross_checks,
                        suffix_offset,
                        suffix_offset as i8, // start at 1 to the right
                        ExploreDir::Right,
                        delimiter_node,
                    );
                }
                return true;
            }

            // check the cross checks
            if cross_check_mask & 1 << (letter - b'A') == 0 {
                return true;
            }

            // check rack
            if let Some(tile) = rack.has_letter(letter) {
                rack.remove_tile(tile);

                let played_tile = match tile {
                    Tile::Blank(_) => Tile::Blank(Some(letter)),
                    _ => tile,
                };

                move_buffer.set(board_idx, letter, PlayedTile::FromRack(played_tile));

                if let Some(next_node) = GADDAG.can_next(current_node, letter) {
                    self.explore(
                        moves,
                        move_buffer,
                        rack,
                        direction,
                        anchor_pos,
                        cross_checks,
                        suffix_offset,
                        offset + explore_dir as i8,
                        explore_dir,
                        next_node,
                    );
                }
                move_buffer.unset(board_idx);
                rack.add_tile(tile);
            }
            true
        });
    }
}
