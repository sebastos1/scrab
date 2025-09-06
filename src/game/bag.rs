use super::tile::Tile;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Bag {
    tiles: Vec<Tile>,
}
// https://en.wikipedia.org/wiki/Scrabble_letter_distributions
impl Bag {
    pub fn new() -> Self {
        let mut tiles = Vec::new();
        let dist = [
            (Tile::Letter(b'A'), 9),
            (Tile::Letter(b'B'), 2),
            (Tile::Letter(b'C'), 2),
            (Tile::Letter(b'D'), 4),
            (Tile::Letter(b'E'), 12),
            (Tile::Letter(b'F'), 2),
            (Tile::Letter(b'G'), 3),
            (Tile::Letter(b'H'), 2),
            (Tile::Letter(b'I'), 9),
            (Tile::Letter(b'J'), 1),
            (Tile::Letter(b'K'), 1),
            (Tile::Letter(b'L'), 4),
            (Tile::Letter(b'M'), 2),
            (Tile::Letter(b'N'), 6),
            (Tile::Letter(b'O'), 8),
            (Tile::Letter(b'P'), 2),
            (Tile::Letter(b'Q'), 1),
            (Tile::Letter(b'R'), 6),
            (Tile::Letter(b'S'), 4),
            (Tile::Letter(b'T'), 6),
            (Tile::Letter(b'U'), 4),
            (Tile::Letter(b'V'), 2),
            (Tile::Letter(b'W'), 2),
            (Tile::Letter(b'X'), 1),
            (Tile::Letter(b'Y'), 2),
            (Tile::Letter(b'Z'), 1),
            (Tile::Blank(None), 200),
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

    // check the amount of tiles left before swapping!!
    pub fn swap(&mut self, tiles: &mut Vec<Tile>) {
        if tiles.len() > self.tiles_left() {
            return;
        }

        // first get new tiles
        let mut new_tiles = Vec::new();
        for _ in 0..tiles.len() {
            if let Some(tile) = self.draw() {
                new_tiles.push(tile);
            }
        }

        // then put old tiles back and shuf that fle
        self.tiles.extend(tiles.drain(..));
        self.shuffle();
        tiles.extend(new_tiles);
    }

    pub fn tiles_left(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    // ui
    pub fn get_tile_counts(&self) -> Vec<(Tile, usize)> {
        let letters = [
            Tile::Letter(b'A'),
            Tile::Letter(b'B'),
            Tile::Letter(b'C'),
            Tile::Letter(b'D'),
            Tile::Letter(b'E'),
            Tile::Letter(b'F'),
            Tile::Letter(b'G'),
            Tile::Letter(b'H'),
            Tile::Letter(b'I'),
            Tile::Letter(b'J'),
            Tile::Letter(b'K'),
            Tile::Letter(b'L'),
            Tile::Letter(b'M'),
            Tile::Letter(b'N'),
            Tile::Letter(b'O'),
            Tile::Letter(b'P'),
            Tile::Letter(b'Q'),
            Tile::Letter(b'R'),
            Tile::Letter(b'S'),
            Tile::Letter(b'T'),
            Tile::Letter(b'U'),
            Tile::Letter(b'V'),
            Tile::Letter(b'W'),
            Tile::Letter(b'X'),
            Tile::Letter(b'Y'),
            Tile::Letter(b'Z'),
            Tile::Blank(None),
        ];

        letters
            .iter()
            .map(|&tile| {
                let count = self.tiles.iter().filter(|&&t| t == tile).count();
                (tile, count)
            })
            .collect()
    }
}
