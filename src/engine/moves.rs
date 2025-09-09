use crate::engine::anchors::CrossChecksExt;
use crate::game::board::Multiplier;
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
    pub used_bits: u16, // bitmask of used tiles
    pub tiles_data: [(u8, Option<PlayedTile>); BOARD_TILES],
    pub pos: Pos,
    pub direction: Direction,
    pub score: u16,
}

impl Move {
    pub fn rack_tiles_count(&self) -> usize {
        let mut count = 0;
        let mut bits = self.used_bits;
        while bits != 0 {
            let i = bits.trailing_zeros() as usize;
            if matches!(self.tiles_data[i].1, Some(PlayedTile::FromRack(_))) {
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
            if self.used_bits & (1 << i) != 0 {
                word.push(self.tiles_data[i].0);
                if let Some(tile) = self.tiles_data[i].1 {
                    tiles.push(tile);
                }
            }
        }

        let word_start_idx = self.used_bits.trailing_zeros() as usize;
        let word_start = match self.direction {
            Direction::Horizontal => Pos::new(self.pos.row, word_start_idx),
            Direction::Vertical => Pos::new(word_start_idx, self.pos.col),
        };

        (String::from_utf8_lossy(&word).to_string(), word_start, tiles)
    }

    pub fn get_word_string(&self) -> String {
        let mut word = Vec::with_capacity(BOARD_TILES);
        for i in 0..BOARD_TILES {
            if self.used_bits & (1 << i) != 0 {
                word.push(self.tiles_data[i].0);
            }
        }
        String::from_utf8_lossy(&word).to_string()
    }
}

#[derive(Debug, Clone, Copy)]
enum ExploreDir {
    Back = -1,
    Forward = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayedTile {
    FromRack(Tile),
    FromBoard(Tile), // known for scoring
}

struct MoveBuffer {
    data: [(u8, Option<PlayedTile>); BOARD_TILES],
    used_bits: u16,
    played_tiles_count: u8, // has a tile from rack
}

impl MoveBuffer {
    fn new() -> Self {
        MoveBuffer {
            data: [(0, None); BOARD_TILES],
            used_bits: 0,
            played_tiles_count: 0,
        }
    }

    fn set(&mut self, pos: usize, letter: u8, tile: PlayedTile) {
        self.data[pos] = (letter, Some(tile));
        self.used_bits |= 1 << pos;
        if matches!(tile, PlayedTile::FromRack(_)) {
            self.played_tiles_count += 1;
        }
    }

    fn unset(&mut self, pos: usize) {
        if matches!(self.data[pos].1, Some(PlayedTile::FromRack(_))) {
            self.played_tiles_count -= 1;
        }
        self.data[pos] = (0, None);
        self.used_bits &= !(1 << pos);
    }

    fn has_played_tile(&self) -> bool {
        self.played_tiles_count > 0
    }

    fn calculate_score(&self, board: &Board, direction: Direction, cross_checks: &CrossChecks, anchor_pos: Pos) -> u16 {
        let mut main_score = 0u16;
        let mut word_multiplier = 1u16;
        let mut cross_scores = 0u16;

        for i in 0..BOARD_TILES {
            if self.used_bits & (1 << i) == 0 {
                continue;
            }

            let pos = match direction {
                Direction::Horizontal => Pos::new(anchor_pos.row, i),
                Direction::Vertical => Pos::new(i, anchor_pos.col),
            };

            if let Some(played_tile) = self.data[i].1 {
                if let PlayedTile::FromBoard(tile) = played_tile {
                    main_score += tile.points() as u16;
                    continue;
                }

                if let PlayedTile::FromRack(tile) = played_tile {
                    let letter_score = tile.points() as u16;

                    main_score += match board.get_multiplier(pos) {
                        Some(Multiplier::DoubleLetter) => letter_score * 2,
                        Some(Multiplier::TripleLetter) => letter_score * 3,
                        Some(Multiplier::DoubleWord) => {
                            word_multiplier *= 2;
                            letter_score
                        }
                        Some(Multiplier::TripleWord) => {
                            word_multiplier *= 3;
                            letter_score
                        }
                        None => letter_score,
                    };

                    let cross_check = unsafe { *cross_checks.get_unchecked(pos.row).get_unchecked(pos.col) };
                    let cross_score = CrossChecks::get_score(cross_check) as u16;
                    if cross_score > 0 {
                        cross_scores += match board.get_multiplier(pos) {
                            Some(Multiplier::DoubleLetter) => cross_score + 2 * letter_score,
                            Some(Multiplier::TripleLetter) => cross_score + 3 * letter_score,
                            Some(Multiplier::DoubleWord) => 2 * (cross_score + letter_score),
                            Some(Multiplier::TripleWord) => 3 * (cross_score + letter_score),
                            None => cross_score + letter_score,
                        };
                    }
                }
            }
        }

        let bingo = if self.played_tiles_count == 7 { 50 } else { 0 };
        main_score * word_multiplier + cross_scores + bingo
    }
}

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
        let start = std::time::Instant::now();
        let (h_anchors, h_allowed) = super::anchors::find_anchors(&self.board, &Direction::Horizontal);
        let (v_anchors, v_allowed) = super::anchors::find_anchors(&self.board, &Direction::Vertical);
        let duration = start.elapsed();

        let mut moves = Vec::new();
        let mut rack = self.rack.clone();

        let anchor_count = h_anchors.len() + v_anchors.len();

        for anchor_pos in h_anchors {
            self.check_anchors(Direction::Horizontal, &mut moves, &mut rack, anchor_pos, &v_allowed);
        }
        for anchor_pos in v_anchors {
            self.check_anchors(Direction::Vertical, &mut moves, &mut rack, anchor_pos, &h_allowed);
        }
        let duration3 = start.elapsed();

        println!(
            "{} anchors in {:.2?}, {} moves in  {:.2?}",
            anchor_count,
            duration,
            moves.len(),
            duration3
        );

        // filter moves ?

        moves
    }

    pub fn check_anchors(&self, direction: Direction, mut moves: &mut Vec<Move>, mut rack: &mut Rack, anchor_pos: Pos, cross_checks: &CrossChecks) {
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
                move_buffer.set(board_idx, tile.byte(), PlayedTile::FromBoard(tile));
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
            ExploreDir::Back,
            current_node,
            false,
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
        placed_tiles_seen: bool,
    ) {
        let offset_dir = match direction {
            Direction::Horizontal => (0, 1),
            Direction::Vertical => (1, 0),
        };

        let current_pos = match anchor_pos.offset(offset_dir.0 * offset as isize, offset_dir.1 * offset as isize) {
            Some(pos) => pos,
            None => {
                // flip direction
                if matches!(explore_dir, ExploreDir::Back) {
                    let new_node = if let Some(delimiter_node) = GADDAG.can_next(current_node, super::gaddag::DELIMITER) {
                        delimiter_node
                    } else {
                        return;
                    };

                    self.explore(
                        moves,
                        move_buffer,
                        rack,
                        direction,
                        anchor_pos,
                        cross_checks,
                        suffix_offset,
                        suffix_offset as i8, // start at 1 to the right
                        ExploreDir::Forward,
                        new_node,
                        placed_tiles_seen,
                    );
                }
                return;
            }
        };

        let board_idx = match direction {
            Direction::Horizontal => current_pos.col,
            Direction::Vertical => current_pos.row,
        };

        if let Some(tile) = self.board.get_tile(current_pos) {
            if let Some(next_node) = GADDAG.can_next(current_node, tile.byte()) {
                move_buffer.set(board_idx, tile.byte(), PlayedTile::FromBoard(tile));

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
                    true, // placed tiles seen
                );

                move_buffer.unset(board_idx);
            }
            return;
        }

        if GADDAG.is_terminal(current_node) && move_buffer.has_played_tile() {
            let score = move_buffer.calculate_score(&self.board, direction, cross_checks, anchor_pos);
            moves.push(Move {
                used_bits: move_buffer.used_bits,
                tiles_data: move_buffer.data,
                pos: Pos {
                    row: anchor_pos.row,
                    col: anchor_pos.col,
                },
                direction,
                score,
            });
        }

        if placed_tiles_seen && matches!(explore_dir, ExploreDir::Back) {
            return;
        }

        let cross_check = unsafe { *cross_checks.get_unchecked(current_pos.row).get_unchecked(current_pos.col) };
        let cross_check_mask = CrossChecks::get_mask(cross_check);
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
                        ExploreDir::Forward,
                        delimiter_node,
                        placed_tiles_seen,
                    );
                }
                return true;
            }

            // check the cross checks
            if cross_check_mask & 1 << (letter - b'A') == 0 {
                return true;
            }

            // check rack
            if let Some(tile) = rack.take_tile(letter) {
                move_buffer.set(board_idx, letter, PlayedTile::FromRack(tile));
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
                        placed_tiles_seen,
                    );
                }
                move_buffer.unset(board_idx);
                rack.add_tile(tile);
            }
            true
        });
    }
}
