use crate::{
    Direction, GADDAG, Pos,
    game::board::{BOARD_SIZE, Board},
};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct CrossCheck(u32);
impl CrossCheck {
    pub const fn new() -> Self {
        Self(0x03FFFFFF) // all letters valid, 0 score
    }

    #[inline]
    pub fn mask(self) -> u32 {
        self.0 & 0x03FFFFFF
    }

    #[inline]
    pub fn score(self) -> u8 {
        (self.0 >> 26) as u8
    }

    #[inline]
    pub fn pack(mask: u32, score: u8) -> Self {
        Self(mask | ((score as u32) << 26))
    }
}

// bitmask 0-25 for A-Z and the final bits are used to store the score. swag
// this gives the score 6 bits, so a max score of 63, which is plenty
pub type CrossChecks = [[CrossCheck; BOARD_SIZE]; BOARD_SIZE];

pub fn empty_cross_checks() -> CrossChecks {
    [[CrossCheck::new(); BOARD_SIZE]; BOARD_SIZE]
}

// finds both anchors and cross checks  from that direction
pub fn find_anchors(board: &Board, direction: &Direction) -> (Vec<Pos>, CrossChecks) {
    if board.is_empty() {
        return (vec![Pos::new(7, 7)], empty_cross_checks());
    }

    let directions = match direction {
        Direction::Horizontal => [(0, -1), (0, 1)], // left, right
        Direction::Vertical => [(-1, 0), (1, 0)],   // up, down
    };

    // get all unique anchors
    let mut anchors = HashSet::new();
    for (pos, _) in board.tiles() {
        for &(dir_row, dir_col) in &directions {
            if let Some(neighbor_pos) = pos.offset(dir_row, dir_col) {
                if board.get_board_tile(neighbor_pos).is_empty() {
                    anchors.insert(neighbor_pos);
                }
            }
        }
    }

    let mut cross_checks = empty_cross_checks(); // bitsets for valid letters

    for &pos in &anchors {
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut cross_score = 0u8;

        let mut current_pos = pos;
        while let Some(prev_pos) = current_pos.offset(directions[0].0, directions[0].1) {
            if let Some(tile) = board.get_tile(prev_pos) {
                prefix.insert(0, tile.byte());
                cross_score = cross_score.saturating_add(tile.points());
                current_pos = prev_pos;
            } else {
                break;
            }
        }

        current_pos = pos;
        while let Some(next_pos) = current_pos.offset(directions[1].0, directions[1].1) {
            if let Some(tile) = board.get_tile(next_pos) {
                suffix.push(tile.byte());
                cross_score = cross_score.saturating_add(tile.points());
                current_pos = next_pos;
            } else {
                break;
            }
        }

        if prefix.is_empty() && suffix.is_empty() {
            CrossCheck::new();
            continue;
        }

        let mut valid_letters = 0u32;
        for c in b'A'..=b'Z' {
            let mut bytes = prefix.clone();
            bytes.push(c);
            bytes.extend(suffix.iter());
            if GADDAG.contains(&bytes) {
                valid_letters |= 1 << (c - b'A');
            }
        }
        cross_checks[pos.row][pos.col] = CrossCheck::pack(valid_letters, cross_score.min(63));
    }

    (anchors.into_iter().collect(), cross_checks)
}
