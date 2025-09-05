mod board;
mod engine;
use macroquad::prelude::*;

const MARGIN: f32 = 50.0;
const WINDOW_WIDTH: f32 = board::BOARD_SIZE + MARGIN * 2.0 + 300.0;
const WINDOW_HEIGHT: f32 = board::BOARD_SIZE + MARGIN * 2.0 + 180.0;
pub const BACKGROUND_COLOR: Color = Color::from_hex(0x071830);

pub struct UI {
    font: Option<Font>,
    scroll_offset: usize,
    hovered_move: Option<usize>,
}

pub fn get_window_config() -> Conf {
    Conf {
        window_title: "scrab".to_string(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

impl UI {
    pub async fn new() -> Self {
        Self {
            font: load_ttf_font_from_bytes(include_bytes!("../../outfit.ttf")).ok(),
            scroll_offset: 0,
            hovered_move: None,
        }
    }
}
