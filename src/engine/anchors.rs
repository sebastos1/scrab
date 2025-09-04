use std::collections::HashMap;

use crate::{engine::moves::Direction, game::board::Board};

impl super::moves::MoveGenerator {
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
    pub fn find_anchors(&self, board: &Board, direction: &Direction) -> (Vec<(usize, usize)>, HashMap<(usize, usize), u32>) {
        if board.is_empty() {
            return (vec![(7, 7)], HashMap::new()); // todo make all allowed?
        }
        let mut anchors = Vec::new();

        // bitsets for valid letters
        let mut cross_checks: HashMap<(usize, usize), u32> = HashMap::new();

        // horizontally
        for row in 0..board.height() {
            for col in 0..board.width() {
                if board.get_tile(row, col).is_some() {
                    continue;
                }

                let (has_prev, has_next) = match direction {
                    Direction::Horizontal => (col > 0 && board.get_tile(row, col - 1).is_some(), col < board.width() - 1 && board.get_tile(row, col + 1).is_some()),
                    Direction::Vertical => (row > 0 && board.get_tile(row - 1, col).is_some(), row < board.height() - 1 && board.get_tile(row + 1, col).is_some()),
                };

                if has_prev || has_next {
                    let mut prefix = Vec::new();
                    let mut suffix = Vec::new();
                    anchors.push((row, col));

                    match direction {
                        Direction::Vertical => {
                            // up/down
                            for i in (0..row).rev() {
                                if let Some(tile) = board.get_tile(i, col) {
                                    prefix.insert(0, tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                            for i in (row + 1)..board.height() {
                                if let Some(tile) = board.get_tile(i, col) {
                                    suffix.push(tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                        }
                        Direction::Horizontal => {
                            // left/right
                            for i in (0..col).rev() {
                                if let Some(tile) = board.get_tile(row, i) {
                                    prefix.insert(0, tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                            for i in (col + 1)..board.width() {
                                if let Some(tile) = board.get_tile(row, i) {
                                    suffix.push(tile.to_byte());
                                } else {
                                    break;
                                }
                            }
                        }
                    }

                    let mut valid_letters = 0u32;
                    for c in b'A'..=b'Z' {
                        let mut bytes = prefix.clone();
                        bytes.push(c);
                        bytes.extend(suffix.iter());
                        if self.gaddag.contains_u8(&bytes) {
                            valid_letters |= 1 << (c - b'A');
                        }
                    }
                    if valid_letters != 0 {
                        cross_checks.insert((row, col), valid_letters);
                    }
                }
            }
        }

        (anchors, cross_checks)
    }
}
