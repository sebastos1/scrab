use super::anchors::CrossChecks;
use crate::Direction;
use crate::game::board::Multiplier;
use crate::{
    GADDAG, Pos,
    game::{
        board::{BOARD_SIZE, Board},
        rack::Rack,
        tile::Tile,
    },
};
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct Move {
    pub tiles_data: SmallVec<[PlayedTile; 7]>,
    pub pos: Pos,
    pub direction: Direction,
    pub score: u16,
}

impl Move {
    pub fn tile_positions(&self) -> impl Iterator<Item = (Pos, PlayedTile)> + '_ {
        self.tiles_data.iter().enumerate().map(|(i, &tile)| {
            let pos = match self.direction {
                Direction::Horizontal => Pos::new(self.pos.row, self.pos.col + i),
                Direction::Vertical => Pos::new(self.pos.row + i, self.pos.col),
            };
            (pos, tile)
        })
    }

    pub fn get_word_string(&self) -> String {
        self.tiles_data.iter().map(|tile| tile.byte() as char).collect()
    }
}

#[derive(Debug, Clone, Copy)]
enum ExploreDir {
    Back = -1,
    Forward = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayedTile {
    Rack(Tile),
    Board(Tile), // known for scoring
}

impl PlayedTile {
    fn byte(&self) -> u8 {
        match self {
            PlayedTile::Rack(tile) => tile.byte(),
            PlayedTile::Board(tile) => tile.byte(),
        }
    }
}

struct MoveBuffer {
    data: [Option<PlayedTile>; BOARD_SIZE],
    played_tiles_count: u8, // has a tile from rack
}

impl MoveBuffer {
    fn new() -> Self {
        MoveBuffer {
            data: [None; BOARD_SIZE],
            played_tiles_count: 0,
        }
    }

    fn set(&mut self, pos: usize, tile: PlayedTile) {
        self.data[pos] = Some(tile);
        if matches!(tile, PlayedTile::Rack(_)) {
            self.played_tiles_count += 1;
        }
    }

    fn unset(&mut self, pos: usize) {
        if matches!(self.data[pos], Some(PlayedTile::Rack(_))) {
            self.played_tiles_count -= 1;
        }
        self.data[pos] = None;
    }

    fn is_occupied(&self, pos: usize) -> bool {
        self.data[pos].is_some()
    }

    fn has_played_tile(&self) -> bool {
        self.played_tiles_count > 0
    }

    fn calculate_score(&self, board: &Board, direction: Direction, cross_checks: &CrossChecks, anchor_pos: Pos) -> u16 {
        let mut main_score = 0u16;
        let mut word_multiplier = 1u16;
        let mut cross_scores = 0u16;

        for i in 0..BOARD_SIZE {
            if !self.is_occupied(i) {
                continue;
            }

            let pos = match direction {
                Direction::Horizontal => Pos::new(anchor_pos.row, i),
                Direction::Vertical => Pos::new(i, anchor_pos.col),
            };

            if let Some(played_tile) = self.data[i] {
                if let PlayedTile::Board(tile) = played_tile {
                    main_score += tile.points() as u16;
                    continue;
                }

                if let PlayedTile::Rack(tile) = played_tile {
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
                    let cross_score = cross_check.score() as u16;
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
    pub fn run(board: Board, rack: Rack) -> Vec<Move> {
        let generator = MoveGenerator { board, rack };
        generator.generate_moves()
    }

    // we start from a board, with an otherwise empty slate
    pub fn generate_moves(&self) -> Vec<Move> {
        let (h_anchors, h_allowed) = super::anchors::find_anchors(&self.board, &Direction::Horizontal);
        let (v_anchors, v_allowed) = super::anchors::find_anchors(&self.board, &Direction::Vertical);

        let mut moves = Vec::new();
        let mut rack = self.rack.clone();

        for anchor_pos in h_anchors {
            self.check_anchors(Direction::Horizontal, &mut moves, &mut rack, anchor_pos, &v_allowed);
        }
        for anchor_pos in v_anchors {
            self.check_anchors(Direction::Vertical, &mut moves, &mut rack, anchor_pos, &h_allowed);
        }

        // filter moves
        // use rand::Rng;
        // if !moves.is_empty() {
        //     let mut rng = rand::rng();
        //     let random_index = rng.random_range(0..moves.len());
        //     let selected_move = moves[random_index].clone();
        //     moves.clear();
        //     moves.push(selected_move);
        // }

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
            if let Some(tile) = self.board.get_board_tile(next_pos) {
                let board_idx = match direction {
                    Direction::Horizontal => next_pos.col,
                    Direction::Vertical => next_pos.row,
                };
                move_buffer.set(board_idx, PlayedTile::Board(tile));
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
        for i in (0..BOARD_SIZE).rev() {
            if move_buffer.is_occupied(i) {
                let byte = move_buffer.data[i].unwrap().byte();
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
        current_node: fst::raw::CompiledAddr,
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

        if let Some(tile) = self.board.get_board_tile(current_pos) {
            if let Some(next_node) = GADDAG.can_next(current_node, tile.byte()) {
                move_buffer.set(board_idx, PlayedTile::Board(tile));

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
            let tiles_data: SmallVec<[PlayedTile; 7]> = move_buffer.data.iter().filter_map(|&tile| tile).collect();
            let word_start_idx = move_buffer.data.iter().position(|tile| tile.is_some()).unwrap_or(0);
            let word_start_pos = match direction {
                Direction::Horizontal => Pos::new(anchor_pos.row, word_start_idx),
                Direction::Vertical => Pos::new(word_start_idx, anchor_pos.col),
            };
            moves.push(Move {
                tiles_data,
                pos: word_start_pos,
                direction,
                score,
            });
        }

        if placed_tiles_seen && matches!(explore_dir, ExploreDir::Back) {
            return;
        }

        let cross_check = unsafe { *cross_checks.get_unchecked(current_pos.row).get_unchecked(current_pos.col) };
        let cross_check_mask = cross_check.mask();
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
                move_buffer.set(board_idx, PlayedTile::Rack(tile));
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
