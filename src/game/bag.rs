use super::{rack::Rack, tile::Tile};
use rand::seq::SliceRandom;

#[derive(Debug, Clone, bincode::Decode, bincode::Encode)]
pub struct Bag {
    pub tiles: Vec<Tile>,
}

impl Bag {
    pub fn new() -> Self {
        let mut tiles = Vec::new();
        let dist = [
            (Tile::letter(b'A'), 9),
            (Tile::letter(b'B'), 2),
            (Tile::letter(b'C'), 2),
            (Tile::letter(b'D'), 4),
            (Tile::letter(b'E'), 12),
            (Tile::letter(b'F'), 2),
            (Tile::letter(b'G'), 3),
            (Tile::letter(b'H'), 2),
            (Tile::letter(b'I'), 9),
            (Tile::letter(b'J'), 1),
            (Tile::letter(b'K'), 1),
            (Tile::letter(b'L'), 4),
            (Tile::letter(b'M'), 2),
            (Tile::letter(b'N'), 6),
            (Tile::letter(b'O'), 8),
            (Tile::letter(b'P'), 2),
            (Tile::letter(b'Q'), 1),
            (Tile::letter(b'R'), 6),
            (Tile::letter(b'S'), 4),
            (Tile::letter(b'T'), 6),
            (Tile::letter(b'U'), 4),
            (Tile::letter(b'V'), 2),
            (Tile::letter(b'W'), 2),
            (Tile::letter(b'X'), 1),
            (Tile::letter(b'Y'), 2),
            (Tile::letter(b'Z'), 1),
            (Tile::blank(None), 2),
        ];

        for (tile, count) in dist.iter() {
            for _ in 0..*count {
                tiles.push(*tile);
            }
        }

        let mut bag = Bag { tiles };
        bag.shuffle();
        bag
    }

    fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.tiles.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Tile> {
        self.tiles.pop()
    }

    pub fn draw_tiles(&mut self, count: usize) -> Vec<Tile> {
        let mut drawn = Vec::new();
        for _ in 0..count {
            if let Some(tile) = self.draw() {
                drawn.push(tile);
            } else {
                break;
            }
        }
        drawn
    }

    pub fn swap(&mut self, rack: &mut Rack, tiles_to_swap: Vec<Tile>) -> bool {
        // there has to be 7 tiles yo
        if self.tiles.len() < 7 || tiles_to_swap.is_empty() || tiles_to_swap.len() > 7 {
            return false;
        }

        for tile_to_swap in &tiles_to_swap {
            rack.remove_tile(*tile_to_swap);
        }

        let new_tiles = self.draw_tiles(tiles_to_swap.len());
        self.tiles.extend(tiles_to_swap);
        self.shuffle();

        for tile in new_tiles {
            rack.add_tile(tile);
        }

        true
    }

    pub fn get_tile_counts(&self) -> Vec<(Tile, usize)> {
        let mut tiles_to_count = Vec::with_capacity(27);

        for letter in b'A'..=b'Z' {
            tiles_to_count.push(Tile::letter(letter));
        }
        tiles_to_count.push(Tile::blank(None));

        tiles_to_count
            .into_iter()
            .map(|tile| {
                let count = self.tiles.iter().filter(|&&t| t == tile).count();
                (tile, count)
            })
            .collect()
    }
}
