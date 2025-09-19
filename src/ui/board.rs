use super::MARGIN;
use macroquad::prelude::*;
use scrab::Pos;
use scrab::game::{
    bag::Bag,
    board::{BOARD_SIZE, Board, Multiplier},
    rack::Rack,
    tile::Tile,
};

pub const BOARD_SIZE_PX: f32 = 600.0;
const BOARD_PADDING: f32 = 10.0;
pub const CELL_SIZE: f32 = BOARD_SIZE_PX / BOARD_SIZE as f32;
pub const TILE_SIZE: f32 = CELL_SIZE * 0.9;
const CORNER_RADIUS: f32 = 6.0;

// const BOARD_COLOR: Color = WHITE;
// const BLANK_TILE_COLOR: Color = Color::from_hex(0xc3c5d0);
const BOARD_COLOR: Color = Color::from_hex(0x252626);
const BLANK_TILE_COLOR: Color = Color::from_hex(0x747575);
const TW_COLOR: Color = Color::from_hex(0xf06292);
const DW_COLOR: Color = Color::from_hex(0xf8bbd9);
const TL_COLOR: Color = Color::from_hex(0x5c6bc0);
const DL_COLOR: Color = Color::from_hex(0x90caf9);
const PLACEABLE_TILE_BG: Color = Color::new(0.996, 0.855, 0.624, 1.0); // #feda9f
const PLACEABLE_TILE_BORDER: Color = Color::new(0.929, 0.784, 0.537, 1.0); // #edc889
const START_TILE_COLOR: Color = Color::new(1.0, 0.84, 0.0, 1.0); // gold
const HIGHLIGHTED_TILE_BG: Color = Color::new(1.0, 0.8, 0.8, 1.0);

impl super::UI {
    // returns the top left corner of a tile
    pub fn tile_position(&self, pos: Pos) -> (f32, f32) {
        let x = MARGIN + pos.col as f32 * CELL_SIZE;
        let y = MARGIN + pos.row as f32 * CELL_SIZE;
        (x, y)

        // end of impl super::UI
    }

    // magic rounded rect
    pub fn draw_rounded_tile(&self, x: f32, y: f32, size: f32, r: f32, color: Color) {
        draw_rectangle(x + r, y, size - 2.0 * r, size, color);
        draw_rectangle(x, y + r, size, size - 2.0 * r, color);
        draw_circle(x + r, y + r, r, color);
        draw_circle(x + size - r, y + r, r, color);
        draw_circle(x + r, y + size - r, r, color);
        draw_circle(x + size - r, y + size - r, r, color);
    }

    fn draw_centered_text(&self, text: &str, x: f32, y: f32, w: f32, font_size: f32, color: Color) {
        let dims = measure_text(text, self.font.as_ref(), font_size as u16, 1.0);
        draw_text_ex(
            text,
            x + (w - dims.width) / 2.0,
            y + (w + dims.height) / 2.0,
            TextParams {
                font: self.font.as_ref(),
                font_size: font_size as u16,
                color,
                ..Default::default()
            },
        );
    }

    pub fn draw_board_tile(&self, pos: Pos, board: &Board) {
        let (bg_color, text, text_color) = if let Some(multi) = board.get_multiplier(pos) {
            match multi {
                Multiplier::TripleWord => (TW_COLOR, "3W", DW_COLOR),
                Multiplier::DoubleWord => {
                    if pos.row == 7 && pos.col == 7 {
                        (START_TILE_COLOR, "^_^", TW_COLOR)
                    } else {
                        (DW_COLOR, "2W", TW_COLOR)
                    }
                }
                Multiplier::TripleLetter => (TL_COLOR, "3L", DL_COLOR),
                Multiplier::DoubleLetter => (DL_COLOR, "2L", TL_COLOR),
            }
        } else {
            (BLANK_TILE_COLOR, "", WHITE)
        };

        let (x, y) = self.tile_position(pos);
        let offset = (CELL_SIZE - TILE_SIZE) / 2.0;
        let x = x + offset;
        let y = y + offset;
        self.draw_rounded_tile(x, y, TILE_SIZE, CORNER_RADIUS, bg_color);

        if !text.is_empty() {
            let font_size = TILE_SIZE * 0.5;
            self.draw_centered_text(text, x, y, TILE_SIZE, font_size, text_color);
        }
    }

    pub fn draw_letter_tile(&self, x: f32, y: f32, size: f32, tile: Tile, highlight: bool) {
        self.draw_rounded_tile(x, y, size, CORNER_RADIUS, PLACEABLE_TILE_BORDER);

        let bg_color = if highlight { HIGHLIGHTED_TILE_BG } else { PLACEABLE_TILE_BG };
        let padding = size * 0.05;
        self.draw_rounded_tile(x + padding, y + padding, size - 2.0 * padding, CORNER_RADIUS, bg_color);

        let letter = tile.to_char().to_string();
        let font_size = size * 0.6;
        self.draw_centered_text(&letter, x, y, size, font_size, BLACK);

        let points = tile.points().to_string();
        if points != "0" {
            let small_font_size = size * 0.25;
            let points_x = x + size - size * 0.25;
            let points_y = y + size - size * 0.1;
            draw_text_ex(
                &points,
                points_x,
                points_y,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: small_font_size as u16,
                    color: BLACK,
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_board(&self, board: &Board) {
        let start_x = MARGIN;
        let start_y = MARGIN;

        self.draw_rounded_tile(
            start_x - BOARD_PADDING,
            start_y - BOARD_PADDING,
            BOARD_SIZE_PX + BOARD_PADDING * 2.,
            CORNER_RADIUS * 2.0,
            BOARD_COLOR,
        );

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                let pos = Pos::new(row, col);
                let (tile_x, tile_y) = self.tile_position(pos);
                if let Some(tile) = board.get_board_tile(pos) {
                    self.draw_letter_tile(tile_x, tile_y, CELL_SIZE, tile, false);
                } else {
                    self.draw_board_tile(pos, board);
                }
            }
        }
    }

    pub fn draw_rack(&self, rack: &Rack, selected_indices: &mut Vec<usize>) {
        for (i, &tile) in rack.tiles().iter().enumerate() {
            let x = MARGIN + i as f32 * (CELL_SIZE + 5.0);
            let y = BOARD_SIZE_PX + MARGIN * 2.0;

            let is_selected = selected_indices.contains(&i);
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                if mouse_x >= x && mouse_x <= x + CELL_SIZE && mouse_y >= y && mouse_y <= y + CELL_SIZE {
                    if is_selected {
                        if let Some(pos) = selected_indices.iter().position(|&idx| idx == i) {
                            selected_indices.remove(pos);
                        }
                    } else {
                        selected_indices.push(i);
                    }
                }
            }

            self.draw_letter_tile(x, y, CELL_SIZE, tile, is_selected);
        }
    }

    pub fn draw_hint(&self) {
        let mut hint_x = MARGIN;
        let keybinds = [("E", "Exchange"), ("P", "Pass"), ("R", "Restart")];
        for (key, action) in keybinds.iter() {
            let text = format!("[{}] {}  ", key, action);
            draw_text_ex(
                &text,
                hint_x,
                BOARD_SIZE_PX + MARGIN * 3.0 + CELL_SIZE + 20.0,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: 20,
                    color: WHITE,
                    ..Default::default()
                },
            );
            let dims = measure_text(&text, self.font.as_ref(), 20, 1.0);
            hint_x += dims.width + 16.0;
        }
    }

    pub fn draw_bag(&self, bag: &Bag) {
        let bag_x = MARGIN + BOARD_SIZE_PX + MARGIN;
        let bag_y = MARGIN + BOARD_SIZE_PX + MARGIN;
        let mini_tile_size = 20.0;
        let spacing = 25.0;
        let grid_cols = 6;
        let total_tiles: u8 = bag.tiles.iter().sum();

        draw_text_ex(
            &format!("Tiles left: {}", total_tiles),
            bag_x,
            bag_y,
            TextParams {
                font: self.font.as_ref(),
                font_size: 14,
                color: WHITE,
                ..Default::default()
            },
        );

        let tile_counts = bag.get_tile_counts();
        let mut row = 0;
        let mut col = 0;

        for (tile, count) in tile_counts.iter() {
            if *count == 0 {
                continue;
            }

            let x = bag_x + col as f32 * (mini_tile_size + spacing);
            let y = bag_y + 20.0 + row as f32 * (mini_tile_size + 8.0);

            self.draw_letter_tile(x, y, mini_tile_size, *tile, false);

            draw_text_ex(
                &count.to_string(),
                x + mini_tile_size + 3.0,
                y + mini_tile_size - 2.0,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: 12,
                    color: WHITE,
                    ..Default::default()
                },
            );

            col += 1;
            if col >= grid_cols {
                col = 0;
                row += 1;
            }
        }
    }
}
