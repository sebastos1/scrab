pub mod bag;
pub mod board;
pub mod rack;
pub mod tile;

use self::{bag::Bag, board::Board, rack::Rack, tile::Tile};
use crate::engine::moves::{Move, PlayedTile};

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct Game {
    pub board: Board,
    pub bag: Bag,
    pub racks: [Rack; 2],
    pub scores: [u16; 2],
    pub current_player: usize,
    pub zeroed_turns: u8,
}

impl Game {
    pub fn init() -> Self {
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

    pub fn is_over(&self) -> bool {
        self.zeroed_turns >= 6 || (self.bag.tiles.is_empty() && (self.racks[0].is_empty() || self.racks[1].is_empty()))
    }

    fn next_turn(&mut self) {
        self.current_player = (self.current_player + 1) % 2;
        if self.is_over() {
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

    fn place_move(&mut self, mv: &Move) {
        let mut tiles_from_rack = Vec::new();

        for (pos, played_tile) in mv.tile_positions() {
            if let PlayedTile::Rack(tile) = played_tile {
                self.board.place_tile(pos, tile);
                tiles_from_rack.push(tile);
            }
        }

        // Remove tiles from rack
        for tile in tiles_from_rack {
            self.racks[self.current_player].remove_tile(tile);
        }

        // Update score
        if mv.score != 0 {
            self.zeroed_turns = 0;
            self.scores[self.current_player] += mv.score;
        } else {
            self.zeroed_turns += 1;
        }
    }

    pub fn play_move(&mut self, mv: &Move) {
        self.place_move(mv);
        while self.racks[self.current_player].tiles().len() < 7 && !self.bag.tiles.is_empty() {
            if let Some(new_tile) = self.bag.draw() {
                self.racks[self.current_player].add_tile(new_tile);
            }
        }

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

    // simulation helpers
    // gives a copy of the current game state with the move applied
    pub fn simulate_move(&self, mv: &Move) -> Game {
        let mut simulated = self.clone();
        simulated.place_move(mv);
        simulated
    }

    pub fn simulate_swap(&self, tiles: Vec<Tile>) -> Option<Game> {
        let mut simulated = self.clone();
        if simulated.exchange(tiles) { Some(simulated) } else { None }
    }
}
