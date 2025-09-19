use super::{rack::Rack, tile::Tile};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Bag {
    pub tiles: [u8; 27],
}

impl Bag {
    pub fn new() -> Self {
        // todo put this somewhere
        let tiles = [9, 2, 2, 4, 12, 2, 3, 2, 9, 1, 1, 4, 2, 6, 8, 2, 1, 6, 4, 6, 4, 2, 2, 1, 2, 1, 2];
        Bag { tiles }
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.iter().sum::<u8>() == 0
    }

    pub fn draw(&mut self) -> Option<Tile> {
        let total: u8 = self.tiles.iter().sum();
        if total == 0 {
            return None;
        }

        let mut target = rand::rng().random_range(0..total);
        for (idx, &count) in self.tiles.iter().enumerate() {
            if target < count {
                self.tiles[idx] -= 1;
                return Some(if idx == 26 { Tile::blank(None) } else { Tile::letter(b'A' + idx as u8) });
            }
            target -= count;
        }
        unreachable!()
    }

    pub fn draw_tiles(&mut self, count: usize) -> Vec<Tile> {
        (0..count).filter_map(|_| self.draw()).collect()
    }

    pub fn swap(&mut self, rack: &mut Rack, tiles_to_swap: Vec<Tile>) -> bool {
        let total: u8 = self.tiles.iter().sum();
        if total < 7 || tiles_to_swap.is_empty() || tiles_to_swap.len() > 7 {
            return false;
        }

        for tile in &tiles_to_swap {
            rack.remove_tile(*tile);
        }

        let new_tiles = self.draw_tiles(tiles_to_swap.len());

        for tile in tiles_to_swap {
            self.tiles[tile.to_index() as usize] += 1;
        }

        for tile in new_tiles {
            rack.add_tile(tile);
        }

        true
    }

    pub fn get_tile_counts(&self) -> Vec<(Tile, usize)> {
        self.tiles
            .iter()
            .enumerate()
            .map(|(idx, &count)| {
                let tile = if idx == 26 { Tile::blank(None) } else { Tile::letter(b'A' + idx as u8) };
                (tile, count as usize)
            })
            .collect()
    }

    pub fn count(&self, index: usize) -> u8 {
        self.tiles[index]
    }
}
