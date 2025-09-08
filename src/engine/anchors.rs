use std::collections::HashSet;

use crate::{
    GADDAG,
    engine::{Pos, moves::Direction},
    game::board::{BOARD_TILES, Board},
};

// bitmask 0-25 for A-Z and the final bits are used to store the score. swag
// this gives the score 6 bits, so a max score of 63, which is plenty in scrabble
pub type CrossChecks = [[u32; BOARD_TILES]; BOARD_TILES];

pub trait CrossChecksExt {
    fn all_ones() -> CrossChecks;
    fn get_mask(value: u32) -> u32;
    fn get_score(value: u32) -> u8;
    fn pack(mask: u32, score: u8) -> u32;
}

impl CrossChecksExt for CrossChecks {
    fn all_ones() -> CrossChecks {
        let mut result = CrossChecks::default();
        for row in 0..BOARD_TILES {
            for col in 0..BOARD_TILES {
                result[row][col] = 0x03FFFFFF; // all letters valid, 0 score
            }
        }
        result
    }

    #[inline]
    fn get_mask(value: u32) -> u32 {
        value & 0x03FFFFFF // bottom 26 bits
    }

    #[inline]
    fn get_score(value: u32) -> u8 {
        (value >> 26) as u8 // top 6 bits
    }

    #[inline]
    fn pack(mask: u32, score: u8) -> u32 {
        mask | ((score as u32) << 26)
    }
}

// finds both anchors and cross checks  from that direction
pub fn find_anchors(board: &Board, direction: &Direction) -> (Vec<Pos>, CrossChecks) {
    if board.is_empty() {
        return (vec![Pos::new(7, 7)], CrossChecks::all_ones());
    }

    let mut anchors = HashSet::new();
    let mut cross_checks = CrossChecks::all_ones(); // bitsets for valid letters

    let directions = match direction {
        Direction::Horizontal => [(0, -1), (0, 1)], // left, right
        Direction::Vertical => [(-1, 0), (1, 0)],   // up, down
    };

    // get all unique anchors
    for (pos, _) in board.tiles() {
        for &(dir_row, dir_col) in &directions {
            if let Some(neighbor_pos) = pos.offset(dir_row, dir_col) {
                if board.get_board_tile(neighbor_pos).is_empty() {
                    anchors.insert(neighbor_pos);
                }
            }
        }
    }

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
            cross_checks[pos.row][pos.col] = 0x03FFFFFF; // everything valid, 0 score
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
        cross_checks[pos.row][pos.col] = CrossChecks::pack(valid_letters, cross_score.min(63));
    }

    (anchors.into_iter().collect(), cross_checks)
}
