pub mod anchors;
pub mod gaddag;
pub mod moves;

use crate::game::board::BOARD_TILES;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

impl Pos {
    pub fn new(row: usize, col: usize) -> Self {
        Pos { row, col }
    }

    pub fn offset(&self, d_row: isize, d_col: isize) -> Option<Pos> {
        let new_row = self.row as isize + d_row;
        let new_col = self.col as isize + d_col;
        if new_row >= 0 && new_col >= 0 && new_row < BOARD_TILES as isize && new_col < BOARD_TILES as isize {
            Some(Pos {
                row: new_row as usize,
                col: new_col as usize,
            })
        } else {
            None
        }
    }
}
