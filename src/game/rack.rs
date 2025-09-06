use super::tile::Tile;

#[derive(Debug, Clone)]
pub struct Rack {
    // 0-25 A-Z, 26 blank
    tiles: [u8; 27],
}

impl Rack {
    pub fn new(tile_vec: Vec<Tile>) -> Self {
        let mut tiles = [0u8; 27];

        for tile in tile_vec {
            match tile {
                Tile::Letter(c) => {
                    tiles[(c - b'A') as usize] += 1;
                }
                Tile::Blank(_) => {
                    tiles[26] += 1;
                }
            }
        }

        Self { tiles }
    }

    pub fn tiles(&self) -> Vec<Tile> {
        let mut tiles = Vec::new();

        for (i, &count) in self.tiles[..26].iter().enumerate() {
            for _ in 0..count {
                tiles.push(Tile::Letter(b'A' + i as u8));
            }
        }

        for _ in 0..self.tiles[26] {
            tiles.push(Tile::Blank(None));
        }

        tiles
    }

    pub fn has_letter(&self, letter: u8) -> Option<Tile> {
        let idx = (letter - b'A') as usize;

        // letter
        if idx < 26 && self.tiles[idx] > 0 {
            return Some(Tile::Letter(letter));
        }

        // blank
        if self.tiles[26] > 0 {
            return Some(Tile::Blank(None));
        }

        None
    }

    pub fn add_tile(&mut self, tile: Tile) {
        match tile {
            Tile::Letter(c) => {
                self.tiles[(c - b'A') as usize] += 1;
            }
            Tile::Blank(_) => {
                self.tiles[26] += 1;
            }
        }
    }

    pub fn remove_tile(&mut self, tile: Tile) -> bool {
        match tile {
            Tile::Letter(c) => {
                let idx = (c - b'A') as usize;
                if self.tiles[idx] > 0 {
                    self.tiles[idx] -= 1;
                    return true;
                }
            }
            Tile::Blank(_) => {
                if self.tiles[26] > 0 {
                    self.tiles[26] -= 1;
                    return true;
                }
            }
        }
        false
    }
}
