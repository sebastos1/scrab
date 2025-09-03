pub mod board;
pub mod rack;
pub mod tile;

use board::Board;
use rack::Rack;
use tile::Tile;

pub struct Game {
    pub board: board::Board,
    pub rack: rack::Rack,
}

pub fn init() -> Game {
    let mut board = Board::new();
    let rack = Rack::new(vec![
        Tile::T,
        Tile::S, // haii
        Tile::Blank,
    ]);

    board.place_tile(7, 6, Tile::R);
    board.place_tile(7, 7, Tile::A);
    board.place_tile(7, 8, Tile::I);
    board.place_tile(7, 9, Tile::N);

    println!("Rack tiles: {:?}", rack.tiles());

    Game { board, rack }
}
