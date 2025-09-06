pub mod board;
pub mod rack;
pub mod tile;

use board::Board;
use rack::Rack;
use tile::Tile;

use crate::engine::Pos;

pub struct Game {
    pub board: board::Board,
    pub rack: rack::Rack,
}

pub fn init() -> Game {
    let mut board = Board::new();

    let rack = ['T', 'S', 'R', 'S', 'E', '*', 'G'].iter().map(|&c| Tile::from_char(c).unwrap()).collect();
    let rack = Rack::new(rack);

    let tiles = [(7, 4, 'S'), (7, 6, 'R'), (7, 7, 'A'), (7, 8, 'I'), (7, 9, 'N'), (6, 10, 'A')];
    for (row, col, tile) in tiles {
        board.place_tile(Pos { row, col }, Tile::from_char(tile).unwrap());
    }

    println!("Rack tiles: {:?}", rack.tiles());

    Game { board, rack }
}
