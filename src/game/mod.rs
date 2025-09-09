pub mod bag;
pub mod board;
pub mod rack;
pub mod tile;

use crate::{
    engine::{
        Pos,
        moves::{Direction, Move, PlayedTile},
    },
    game::{bag::Bag, board::BOARD_TILES, tile::Tile},
};
use board::Board;
use rack::Rack;

pub struct Game {
    pub board: Board,
    pub bag: Bag,
    pub racks: [Rack; 2],
    pub scores: [u16; 2],
    pub current_player: usize,
    pub zeroed_turns: u8,
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
        zeroed_turns: 0,
    }
}

impl Game {
    pub fn next_turn(&mut self) {
        self.current_player = (self.current_player + 1) % 2;
        if self.zeroed_turns >= 6 || (self.bag.tiles.is_empty() && (self.racks[0].is_empty() || self.racks[1].is_empty())) {
            let (winner, scores) = self.end_game();
            println!("Game over! Winner: {:?}, Scores: {:?}", winner, scores);
        }
    }

    pub fn end_game(&mut self) -> (Option<usize>, [u16; 2]) {
        let p1_rack_points: u16 = self.racks[0].tiles().iter().map(|t| t.points() as u16).sum();
        let p2_rack_points: u16 = self.racks[1].tiles().iter().map(|t| t.points() as u16).sum();

        // player 1 went out
        if self.racks[0].is_empty() && self.bag.tiles.is_empty() {
            self.scores[0] += 2 * p2_rack_points;

        // player 2 went out
        } else if self.racks[1].is_empty() && self.bag.tiles.is_empty() {
            self.scores[1] += 2 * p1_rack_points;

        // nobody went out
        } else {
            self.scores[0] -= p1_rack_points;
            self.scores[1] -= p2_rack_points;
        }

        (
            if self.scores[0] > self.scores[1] {
                Some(0)
            } else if self.scores[1] > self.scores[0] {
                Some(1)
            } else {
                None
            },
            self.scores,
        )
    }

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

        while self.racks[self.current_player].tiles().len() < 7 && !self.bag.tiles.is_empty() {
            if let Some(new_tile) = self.bag.draw() {
                self.racks[self.current_player].add_tile(new_tile);
            }
        }

        if mv.score != 0 {
            self.zeroed_turns = 0;
            self.scores[self.current_player] += mv.score;
        } else {
            self.zeroed_turns += 1; // technically possible (blank next to a blank)
        }

        println!("player {} played move: {:?}, scoring {}", self.current_player + 1, mv, mv.score);

        self.next_turn();
    }

    pub fn pass_turn(&mut self) {
        self.zeroed_turns += 1;
        self.next_turn();
    }

    pub fn exchange(&mut self, tiles: Vec<Tile>) -> bool {
        if !self.bag.swap(&mut self.racks[self.current_player], tiles) {
            return false;
        }
        self.zeroed_turns += 1;
        self.next_turn();
        true
    }
}
