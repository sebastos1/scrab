mod engine;
pub mod game;
pub mod ui;

use crate::{engine::moves::MoveGenerator, ui::get_window_config};
use engine::gaddag::Gaddag;
use lazy_static::lazy_static;
use macroquad::prelude::*;
use ui::*;

lazy_static! {
    static ref GADDAG: Gaddag = Gaddag::from_wordlist("wordlist.txt");
}

#[macroquad::main(get_window_config)]
async fn main() {
    println!("Gaddag entry count: {:#?}", &GADDAG.0.as_fst().size());

    let mut game = game::init();

    let mut ui = UI::new().await;
    let mut moves = Vec::new();
    let mut board_updated = true;
    loop {
        clear_background(ui::BACKGROUND_COLOR);
        ui.draw_board(&game.board);
        ui.draw_rack(&game.racks[game.current_player]);
        ui.draw_bag(&game.bag);

        if board_updated {
            let move_generator = MoveGenerator::new(game.board.clone(), game.racks[game.current_player].clone());
            let start = std::time::Instant::now();
            moves = move_generator.generate_moves();
            let elapsed = start.elapsed();
            println!("{} moves in {:.2?}", moves.len(), elapsed);
            board_updated = false;
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

        next_frame().await
    }
}
