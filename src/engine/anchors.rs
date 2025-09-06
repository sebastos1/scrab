use std::collections::HashSet;

use crate::{
    GADDAG,
    engine::{Pos, moves::Direction},
    game::board::{BOARD_TILES, Board},
};

pub type CrossChecks = [[u32; BOARD_TILES]; BOARD_TILES];

pub trait CrossChecksExt {
    fn all_ones() -> CrossChecks;
}

impl CrossChecksExt for CrossChecks {
    fn all_ones() -> CrossChecks {
        let mut result = CrossChecks::default();
        for row in 0..BOARD_TILES {
            for col in 0..BOARD_TILES {
                result[row][col] = u32::MAX;
            }
        }
        result
    }
}

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
                if board.get_tile(neighbor_pos).is_none() {
                    anchors.insert(neighbor_pos);
                }
            }
        }
    }

    for &pos in &anchors {
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();

        let mut current_pos = pos;
        while let Some(prev_pos) = current_pos.offset(directions[0].0, directions[0].1) {
            if let Some(tile) = board.get_tile(prev_pos) {
                prefix.insert(0, tile.byte());
                current_pos = prev_pos;
            } else {
                break;
            }
        }

        current_pos = pos;
        while let Some(next_pos) = current_pos.offset(directions[1].0, directions[1].1) {
            if let Some(tile) = board.get_tile(next_pos) {
                suffix.push(tile.byte());
                current_pos = next_pos;
            } else {
                break;
            }
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
        cross_checks[pos.row][pos.col] = valid_letters;
    }

    (anchors.into_iter().collect(), cross_checks)
}
