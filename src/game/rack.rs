use super::tile::Tile;

pub const RACK_TILES: usize = 7;

#[derive(Debug, Clone)]
pub struct Rack {
    pub tiles: [Tile; RACK_TILES],
    pub count: u8,
    pub mask: u32,
}

impl Rack {
    pub fn new(tile_vec: Vec<Tile>) -> Self {
        let count = tile_vec.len().min(RACK_TILES) as u8;

        let mut tiles = [Tile::empty(); RACK_TILES];
        let mut mask = 0u32;
        for (i, tile) in tile_vec.into_iter().take(RACK_TILES).enumerate() {
            tiles[i] = tile;
            if !tile.is_blank() {
                mask |= 1 << (tile.byte() - b'A');
            }
        }

        Self { tiles, count, mask }
    }

    pub fn tiles(&self) -> &[Tile] {
        unsafe {
            // count is always <= RACK_TILES
            self.tiles.get_unchecked(..self.count as usize)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn take_tile(&mut self, letter: u8) -> Option<Tile> {
        if letter >= b'A' && letter <= b'Z' {
            let bit = 1u32 << (letter - b'A');
            if (self.mask & bit) != 0 {
                for i in 0..self.count as usize {
                    if self.tiles[i].byte() == letter && !self.tiles[i].is_blank() {
                        let tile = self.tiles[i];
                        self.remove_at(i);
                        return Some(tile);
                    }
                }
            }
        }

        for i in 0..self.count as usize {
            if self.tiles[i].is_blank() && self.tiles[i].byte() == b'*' {
                self.remove_at(i);
                return Some(Tile::blank(Some(letter)));
            }
        }

        None
    }

    pub fn add_tile(&mut self, tile: Tile) {
        if self.count < RACK_TILES as u8 {
            let rack_tile = if tile.is_blank() {
                Tile::blank(None)
            } else {
                self.mask |= 1 << (tile.byte() - b'A');
                tile
            };
            self.tiles[self.count as usize] = rack_tile;
            self.count += 1;
        }
    }

    pub fn remove_tile(&mut self, tile: Tile) -> bool {
        let count = self.count as usize;
        if tile.is_blank() {
            for i in 0..count {
                if self.tiles[i].is_blank() {
                    self.remove_at(i);
                    return true;
                }
            }
        } else {
            let letter = tile.byte();
            if letter >= b'A' && letter <= b'Z' {
                let bit = 1u32 << (letter - b'A');
                if (self.mask & bit) == 0 {
                    return false;
                }
            }

            for i in 0..count {
                if self.tiles[i] == tile {
                    self.remove_at(i);
                    return true;
                }
            }
        }
        false
    }

    fn remove_at(&mut self, index: usize) {
        let last_idx = (self.count - 1) as usize;
        let removed = self.tiles[index];
        if !removed.is_blank() && removed.byte() >= b'A' && removed.byte() <= b'Z' {
            let letter = removed.byte();
            let mut found_another = false;
            for i in 0..self.count as usize {
                if i != index && self.tiles[i].byte() == letter && !self.tiles[i].is_blank() {
                    found_another = true;
                    break;
                }
            }
            if !found_another {
                self.mask &= !(1 << (letter - b'A'));
            }
        }

        if index != last_idx {
            self.tiles[index] = self.tiles[last_idx];
        }
        self.tiles[last_idx] = Tile::empty();
        self.count -= 1;
    }
}
