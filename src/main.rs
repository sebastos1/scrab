mod ai;
mod engine;
mod game;
mod ui;

use crate::{
    ai::network::Network,
    engine::moves::MoveGenerator,
    game::{Game, tile::Tile},
    ui::get_window_config,
};
use engine::gaddag::Gaddag;
use lazy_static::lazy_static;
use macroquad::prelude::*;
use ui::*;

lazy_static! {
    static ref GADDAG: Gaddag = Gaddag::from_wordlist("wordlists/CSW24.txt");
}

#[macroquad::main(get_window_config)]
async fn main() {
    println!("Gaddag entry count: {:#?}", &GADDAG.0.as_fst().size());

    let mut game = game::init();
    let network = Network::init().unwrap();

    let mut ui = UI::new().await;
    let mut moves = Vec::new();
    let mut board_updated = true;
    let mut selected_rack_tiles: Vec<usize> = Vec::new();
    loop {
        clear_background(ui::BACKGROUND_COLOR);
        ui.draw_board(&game.board);
        ui.draw_rack(&game.racks[game.current_player], &mut selected_rack_tiles);
        ui.draw_bag(&game.bag);
        ui.draw_players(&game.scores, game.current_player);

        if board_updated {
            let move_generator = MoveGenerator::new(game.board.clone(), game.racks[game.current_player].clone());
            moves = move_generator.generate_moves();
            board_updated = false;
            selected_rack_tiles.clear();

            // simulate moves and score
            let mut evals = Vec::new();
            let sim_games: Vec<Game> = moves.iter().map(|mv| game.simulate_move(mv)).collect();

            let evaluations = network.evaluate_batch(&sim_games).unwrap();

            for (i, mv) in moves.iter().enumerate() {
                evals.push((mv.get_word_string(), evaluations[i]));
            }
            for eval in evals {
                println!("Move: {} scores {:.3}", eval.0, eval.1);
            }
        }

        if let Some(move_idx) = ui.draw_move_list(&moves) {
            if let Some(selected_move) = moves.get(move_idx) {
                game.play_move(selected_move);
                board_updated = true;
            }
        }

        if is_key_pressed(KeyCode::R) {
            game = game::init();
            board_updated = true;
        }

        if is_key_pressed(KeyCode::P) {
            game.pass_turn();
            board_updated = true;
        }

        if is_key_pressed(KeyCode::E) {
            let tiles_to_exchange: Vec<Tile> = selected_rack_tiles.iter().map(|&i| game.racks[game.current_player].tiles()[i]).collect();
            if game.exchange(tiles_to_exchange) {
                board_updated = true;
            };
        }

        ui.draw_hint();

        next_frame().await
    }
}
