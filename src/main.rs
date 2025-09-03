mod engine;
pub mod game;
pub mod ui;

use crate::ui::get_window_config;
use engine::gaddag::Gaddag;
use macroquad::prelude::*;
use ui::*;

#[macroquad::main(get_window_config)]
async fn main() {
    let gaddag = Gaddag::from_wordlist("wordlist2.txt");

    println!("{:#?}", gaddag.0.as_fst().size());

    let move_generator = engine::moves::MoveGenerator::new(gaddag);
    let game = game::init();
    let debug = move_generator.generate_moves(&game);

    let mut board_updated = true;
    let ui = UI::new().await;
    loop {
        clear_background(ui::BACKGROUND_COLOR);

        ui.draw_board(&game.board);
        ui.draw_rack(&game.rack);

        ui.draw_debug_info(&debug);

        if board_updated {
            // let start = std::time::Instant::now();
            // let elapsed = start.elapsed();
            board_updated = false;
        }

        if is_key_pressed(KeyCode::R) {
            board_updated = true;
        }

        next_frame().await
    }
}
