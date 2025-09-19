use super::tile::Tile;
use crate::Pos;
use lazy_static::lazy_static;

lazy_static! {
    static ref BOARD_MULTIPLIERS: [[Option<Multiplier>; BOARD_SIZE]; BOARD_SIZE] = init_multipliers();
}

pub const BOARD_SIZE: usize = 15;
// pub const START_POS: Pos = Pos { row: 7, col: 7 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiplier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

#[derive(Debug, Clone)]
pub struct Board {
    tiles: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Self {
        Self {
            tiles: [[None; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn place_tile(&mut self, pos: Pos, tile: Tile) -> bool {
        let (row, col) = (pos.row, pos.col);
        if row < BOARD_SIZE && col < BOARD_SIZE && self.tiles[row][col].is_none() {
            self.tiles[row][col] = Some(tile);
            true
        } else {
            false
        }
    }

    pub fn get_board_tile(&self, pos: Pos) -> Option<Tile> {
        if pos.row < BOARD_SIZE && pos.col < BOARD_SIZE {
            self.tiles[pos.row][pos.col]
        } else {
            None
        }
    }

    pub fn get_multiplier(&self, pos: Pos) -> Option<Multiplier> {
        if pos.row < BOARD_SIZE && pos.col < BOARD_SIZE {
            return BOARD_MULTIPLIERS[pos.row][pos.col];
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        for row in &self.tiles {
            for &cell in row {
                if cell.is_some() {
                    return false;
                }
            }
        }
        true
    }

    // gets filled tiles
    pub fn tiles(&self) -> Vec<(Pos, Tile)> {
        let mut tiles = Vec::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if let Some(tile) = self.tiles[row][col] {
                    tiles.push((Pos::new(row, col), tile));
                }
            }
        }
        tiles
    }
}

fn init_multipliers() -> [[Option<Multiplier>; BOARD_SIZE]; BOARD_SIZE] {
    let mut multipliers = [[None; BOARD_SIZE]; BOARD_SIZE];

    let triple_word = [(0, 0), (0, 7), (0, 14), (7, 0), (7, 14), (14, 0), (14, 7), (14, 14)];
    for &(row, col) in &triple_word {
        multipliers[row][col] = Some(Multiplier::TripleWord);
    }

    let double_word = [
        (1, 1),
        (2, 2),
        (3, 3),
        (4, 4),
        (1, 13),
        (2, 12),
        (3, 11),
        (4, 10),
        (13, 1),
        (12, 2),
        (11, 3),
        (10, 4),
        (13, 13),
        (12, 12),
        (11, 11),
        (10, 10),
        (7, 7),
    ];
    for &(row, col) in &double_word {
        multipliers[row][col] = Some(Multiplier::DoubleWord);
    }

    let triple_letter = [
        (1, 5),
        (1, 9),
        (5, 1),
        (5, 5),
        (5, 9),
        (5, 13),
        (9, 1),
        (9, 5),
        (9, 9),
        (9, 13),
        (13, 5),
        (13, 9),
    ];
    for &(row, col) in &triple_letter {
        multipliers[row][col] = Some(Multiplier::TripleLetter);
    }

    let double_letter = [
        (0, 3),
        (0, 11),
        (2, 6),
        (2, 8),
        (3, 0),
        (3, 7),
        (3, 14),
        (6, 2),
        (6, 6),
        (6, 8),
        (6, 12),
        (7, 3),
        (7, 11),
        (8, 2),
        (8, 6),
        (8, 8),
        (8, 12),
        (11, 0),
        (11, 7),
        (11, 14),
        (12, 6),
        (12, 8),
        (14, 3),
        (14, 11),
    ];
    for &(row, col) in &double_letter {
        multipliers[row][col] = Some(Multiplier::DoubleLetter);
    }

    multipliers
}
