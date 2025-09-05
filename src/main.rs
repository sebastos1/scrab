mod engine;
pub mod game;
pub mod ui;
pub mod util;

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

    let mut debug_things = crate::engine::moves::DebugThings {
        horizontal_anchors: Vec::new(),
        vertical_anchors: Vec::new(),
        horizontal_allowed_ext: std::collections::HashMap::new(),
        vertical_allowed_ext: std::collections::HashMap::new(),
    };
    let mut moves = Vec::new();
    let mut board_updated = true;
    let mut ui = UI::new().await;
    loop {
        clear_background(ui::BACKGROUND_COLOR);

        ui.draw_board(&game.board);
        ui.draw_rack(&game.rack);

        if board_updated {
            let start = std::time::Instant::now();
            (debug_things, moves) = move_generator.generate_moves(&game);
            let elapsed = start.elapsed();
            println!("Move generation took: {:.2?}", elapsed);
            board_updated = false;
        }

        ui.draw_debug_info(&debug_things);
        ui.draw_move_list(&moves);

        if is_key_pressed(KeyCode::R) {
            board_updated = true;
        }

        next_frame().await
    }
}
