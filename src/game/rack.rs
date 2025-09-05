use std::collections::HashMap;

use super::tile::Tile;

#[derive(Debug, Clone)]
pub struct Rack {
    tiles: Vec<Tile>,
}

impl Rack {
    pub fn new(tiles: Vec<Tile>) -> Self {
        Self { tiles }
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub fn has_letter(&self, letter: u8) -> Option<Tile> {
        for &tile in &self.tiles {
            if tile.to_byte() == letter || tile == Tile::Blank {
                return Some(tile);
            }
        }
        None
    }

    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    pub fn remove_tile(&mut self, tile: Tile) -> bool {
        if let Some(pos) = self.tiles.iter().position(|&t| t == tile) {
            self.tiles.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn remove_tiles(&mut self, used_tiles: &[Tile]) -> bool {
        let mut temp_rack = self.tiles.clone();

        for &tile in used_tiles {
            if let Some(pos) = temp_rack.iter().position(|&t| t == tile) {
                temp_rack.remove(pos);
            } else {
                return false;
            }
        }

        self.tiles = temp_rack;
        true
    }

    pub fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        for tile in self.tiles() {
            if *tile != Tile::Blank {
                let ch = tile.to_byte();
                bits |= 1 << (ch - b'A');
            }
        }
        bits
    }

    pub fn to_counts(&self) -> HashMap<u8, i32> {
        let mut counts = HashMap::new();
        for tile in self.tiles() {
            let ch = tile.to_byte();
            *counts.entry(ch).or_insert(0) += 1;
        }
        counts
    }
}
