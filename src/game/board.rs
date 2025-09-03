use super::tile::Tile;

const BOARD_SIZE: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multiplier {
    Normal,
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

#[derive(Debug, Clone)]
pub struct Board {
    grid: [[Option<Tile>; BOARD_SIZE]; BOARD_SIZE],
    multipliers: [[Multiplier; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Self {
        let mut multipliers = [[Multiplier::Normal; BOARD_SIZE]; BOARD_SIZE];
        let triple_word = [(0, 0), (0, 7), (0, 14), (7, 0), (7, 14), (14, 0), (14, 7), (14, 14)];
        for &(row, col) in &triple_word {
            multipliers[row][col] = Multiplier::TripleWord;
        }

        let double_word = [(1, 1), (2, 2), (3, 3), (4, 4), (1, 13), (2, 12), (3, 11), (4, 10), (13, 1), (12, 2), (11, 3), (10, 4), (13, 13), (12, 12), (11, 11), (10, 10)];
        for &(row, col) in &double_word {
            multipliers[row][col] = Multiplier::DoubleWord;
        }

        let triple_letter = [(1, 5), (1, 9), (5, 1), (5, 5), (5, 9), (5, 13), (9, 1), (9, 5), (9, 9), (9, 13), (13, 5), (13, 9)];
        for &(row, col) in &triple_letter {
            multipliers[row][col] = Multiplier::TripleLetter;
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
            multipliers[row][col] = Multiplier::DoubleLetter;
        }

        Self { grid: [[None; BOARD_SIZE]; BOARD_SIZE], multipliers }
    }

    pub fn height(&self) -> usize {
        BOARD_SIZE
    }

    pub fn width(&self) -> usize {
        BOARD_SIZE
    }

    pub fn place_tile(&mut self, row: usize, col: usize, tile: Tile) -> bool {
        if row < BOARD_SIZE && col < BOARD_SIZE && self.grid[row][col].is_none() {
            self.grid[row][col] = Some(tile);
            true
        } else {
            false
        }
    }

    pub fn get_tile(&self, row: usize, col: usize) -> Option<Tile> {
        if row < BOARD_SIZE && col < BOARD_SIZE { self.grid[row][col] } else { None }
    }

    pub fn get_multiplier(&self, row: usize, col: usize) -> Multiplier {
        if row < BOARD_SIZE && col < BOARD_SIZE { self.multipliers[row][col] } else { Multiplier::Normal }
    }

    pub fn is_empty(&self) -> bool {
        for row in &self.grid {
            for &cell in row {
                if cell.is_some() {
                    return false;
                }
            }
        }
        true
    }
}
