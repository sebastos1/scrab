pub mod bag;
pub mod board;
pub mod rack;
pub mod tile;

use crate::{
    engine::{
        Pos,
        moves::{Direction, Move, PlayedTile},
    },
    game::{bag::Bag, board::BOARD_TILES},
};
use board::Board;
use rack::Rack;

pub struct Game {
    pub board: Board,
    pub bag: Bag,
    pub racks: [Rack; 2],
    pub scores: [u16; 2],
    pub current_player: usize,
}

pub fn init() -> Game {
    let board = Board::new();
    let mut bag = Bag::new();
    let racks = [Rack::new(bag.draw_tiles(7)), Rack::new(bag.draw_tiles(7))];
    Game {
        board,
        racks,
        scores: [0, 0],
        current_player: 0,
        bag,
    }
}

impl Game {
    pub fn play_move(&mut self, mv: &Move) {
        let word_start_idx = mv.used_bits.trailing_zeros() as usize;
        let start_pos = match mv.direction {
            Direction::Horizontal => Pos::new(mv.pos.row, word_start_idx),
            Direction::Vertical => Pos::new(word_start_idx, mv.pos.col),
        };

        let mut tile_offset = 0;
        let mut tiles_from_rack = Vec::new();

        for i in 0..BOARD_TILES {
            if mv.used_bits & (1 << i) != 0 {
                if let Some(PlayedTile::FromRack(tile)) = mv.tiles_data[i].1 {
                    let pos = match mv.direction {
                        Direction::Horizontal => Pos::new(start_pos.row, start_pos.col + tile_offset),
                        Direction::Vertical => Pos::new(start_pos.row + tile_offset, start_pos.col),
                    };
                    self.board.place_tile(pos, tile);
                    tiles_from_rack.push(tile);
                }
                tile_offset += 1;
            }
        }

        for tile in tiles_from_rack {
            self.racks[self.current_player].remove_tile(tile);
        }

        while self.racks[self.current_player].tiles().len() < 7 && !self.bag.is_empty() {
            if let Some(new_tile) = self.bag.draw() {
                self.racks[self.current_player].add_tile(new_tile);
            }
        }

        self.scores[self.current_player] += mv.score;
        self.current_player = (self.current_player + 1) % 2;
    }
}
