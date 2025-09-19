use macroquad::prelude::*;
use scrab::{
    Game, Tile,
    ai::{network::Network, training::train},
};

mod ui;
use ui::*;

// #[macroquad::main(get_window_config)]
fn main() {
    // let record = scrab::ai::data::test();

    let mut network = Network::load("models/model.safetensors").unwrap();
    train(&mut network, "data/positions.bin").unwrap();

    // let mut game = Game::init();
    // let mut moves = Vec::new();

    // let mut ui = UI::new().await;
    // let mut board_updated = true;
    // let mut selected_rack_tiles: Vec<usize> = Vec::new();

    // let mut replay_index = 0;

    // loop {
    //     if board_updated {
    //         let timer = std::time::Instant::now();
    //         moves = scrab::MoveGenerator::run(game.board.clone(), game.racks[game.current_player].clone());
    //         let elapsed = timer.elapsed();
    //         println!("Generated {} moves in {:.2?}", moves.len(), elapsed);
    //         selected_rack_tiles.clear();
    //         board_updated = false;
    //     }

    //     clear_background(ui::BACKGROUND_COLOR);
    //     ui.draw_board(&game.board);
    //     ui.draw_rack(&game.racks[game.current_player], &mut selected_rack_tiles);
    //     ui.draw_bag(&game.bag);
    //     ui.draw_players(&game.scores, game.current_player);
    //     ui.draw_hint();

    //     if let Some(move_idx) = ui.draw_move_list(&moves) {
    //         if let Some(selected_move) = moves.get(move_idx) {
    //             if !game.is_over() {
    //                 game.play_move(selected_move);
    //                 board_updated = true;
    //             }
    //         }
    //     }

    //     if is_key_pressed(KeyCode::R) {
    //         game = Game::init();
    //         board_updated = true;
    //     }

    //     if is_key_pressed(KeyCode::P) {
    //         game.pass_turn();
    //         board_updated = true;
    //     }

    //     if is_key_pressed(KeyCode::E) {
    //         let tiles_to_exchange: Vec<Tile> = selected_rack_tiles.iter().map(|&i| game.racks[game.current_player].tiles()[i]).collect();
    //         game.exchange(tiles_to_exchange);
    //         board_updated = true;
    //     }

    //     // if is_key_pressed(KeyCode::D) {
    //     //     if replay_index < record.moves.len() {
    //     //         let game_move = &record.moves[replay_index];

    //     //         match &game_move.action {
    //     //             scrab::ai::data::Action::Move(mv) => {
    //     //                 game.play_move(mv);
    //     //                 board_updated = true;
    //     //             }
    //     //             scrab::ai::data::Action::Pass => {
    //     //                 game.pass_turn();
    //     //                 board_updated = true;
    //     //             }
    //     //             scrab::ai::data::Action::Swap(tiles) => {
    //     //                 game.exchange(tiles.clone());
    //     //                 board_updated = true;
    //     //             }
    //     //         }

    //     //         replay_index += 1;
    //     //         println!("Applied move {}", replay_index);
    //     //     }
    //     // }

    //     next_frame().await
    // }
}
