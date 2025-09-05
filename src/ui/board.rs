use crate::game::board::BOARD_TILES;
use crate::game::board::Board;
use crate::game::board::Multiplier;
use crate::game::rack::Rack;
use crate::game::tile::Tile;
use crate::util::Pos;
use macroquad::prelude::*;

pub const BOARD_SIZE: f32 = 600.0;
pub const CELL_SIZE: f32 = BOARD_SIZE / 15.0;
pub const TILE_SIZE: f32 = CELL_SIZE * 0.9;
const CORNER_RADIUS: f32 = 6.0;

const BOARD_COLOR: Color = WHITE;
const TW_COLOR: Color = Color::from_hex(0xbe4f50);
const TL_COLOR: Color = Color::from_hex(0x0f6798);
const DW_COLOR: Color = Color::from_hex(0x6aa3c4);
const DL_COLOR: Color = Color::from_hex(0xe3a3a5);
const BLANK_TILE_COLOR: Color = Color::new(0.76, 0.77, 0.82, 1.0); // #c3c5d0
const PLACEABLE_TILE_BG: Color = Color::new(0.996, 0.855, 0.624, 1.0); // #feda9f
const PLACEABLE_TILE_BORDER: Color = Color::new(0.929, 0.784, 0.537, 1.0); // #edc889
const START_TILE_COLOR: Color = Color::new(1.0, 0.84, 0.0, 1.0); // gold
const BOARD_PADDING: f32 = 10.0;

impl super::UI {
    // returns the top left corner of a tile
    pub fn tile_position(&self, pos: Pos) -> (f32, f32) {
        let x = super::MARGIN + pos.col as f32 * CELL_SIZE;
        let y = super::MARGIN + pos.row as f32 * CELL_SIZE;
        (x, y)
    }

    pub fn tile_center(&self, pos: Pos) -> (f32, f32) {
        let (x, y) = self.tile_position(pos);
        (x + CELL_SIZE / 2.0, y + CELL_SIZE / 2.0)
    }

    pub fn draw_board(&self, board: &Board) {
        let start_x = super::MARGIN;
        let start_y = super::MARGIN;

        self.draw_rounded_rect(
            start_x - BOARD_PADDING,
            start_y - BOARD_PADDING,
            BOARD_SIZE + BOARD_PADDING * 2.,
            BOARD_SIZE + BOARD_PADDING * 2.,
            CORNER_RADIUS * 2.0,
            BOARD_COLOR,
        );

        for row in 0..BOARD_TILES {
            for col in 0..BOARD_TILES {
                let (cell_x, cell_y) = self.tile_position(Pos::new(row, col));

                // for spacing between tiles
                let tile_offset = (CELL_SIZE - TILE_SIZE) / 2.0;
                let tile_x = cell_x + tile_offset;
                let tile_y = cell_y + tile_offset;

                if let Some(tile) = board.get_tile(Pos::new(row, col)) {
                    self.draw_placeable_tile(tile_x, tile_y, tile, false);
                } else {
                    self.draw_multiplier_tile(tile_x, tile_y, board, row, col);
                }
            }
        }
    }

    pub fn draw_rack(&self, rack: &Rack) {
        let rack_y = super::MARGIN + BOARD_SIZE + 30.0;
        let rack_start_x = super::MARGIN + (BOARD_SIZE - 7.0 * (CELL_SIZE + 5.0)) / 2.0;

        for (i, &tile) in rack.tiles().iter().enumerate() {
            let x = rack_start_x + i as f32 * (CELL_SIZE + 5.0);

            let tile_offset = (CELL_SIZE - TILE_SIZE) / 2.0;
            let tile_x = x + tile_offset;
            let tile_y = rack_y + tile_offset;

            self.draw_placeable_tile(tile_x, tile_y, tile, false);
        }
    }

    pub fn draw_placeable_tile(&self, x: f32, y: f32, tile: Tile, highlight: bool) {
        self.draw_rounded_rect(
            x - TILE_SIZE * 0.05,
            y - TILE_SIZE * 0.05,
            TILE_SIZE * 1.1,
            TILE_SIZE * 1.1,
            CORNER_RADIUS,
            PLACEABLE_TILE_BORDER,
        );

        let bg_color = if highlight {
            Color::new(1.0, 0.8, 0.8, 1.0) // red tint
        } else {
            PLACEABLE_TILE_BG
        };

        self.draw_rounded_rect(x, y, TILE_SIZE, TILE_SIZE, CORNER_RADIUS, bg_color);

        self.draw_tile_content(x, y, tile, BLACK);
    }

    fn draw_multiplier_tile(&self, x: f32, y: f32, board: &Board, row: usize, col: usize) {
        let (color, text) = match board.get_multiplier(Pos::new(row, col)) {
            Multiplier::TripleWord => (TW_COLOR, "TW"),
            Multiplier::DoubleWord => (DW_COLOR, "DW"),
            Multiplier::TripleLetter => (TL_COLOR, "TL"),
            Multiplier::DoubleLetter => (DL_COLOR, "DL"),
            Multiplier::Normal => {
                if row == 7 && col == 7 {
                    (START_TILE_COLOR, "★")
                } else {
                    (BLANK_TILE_COLOR, "")
                }
            }
        };

        self.draw_rounded_rect(x, y, TILE_SIZE, TILE_SIZE, CORNER_RADIUS, color);

        if !text.is_empty() {
            let font_size = TILE_SIZE * 0.3;
            let text_dims = measure_text(text, self.font.as_ref(), font_size as u16, 1.0);
            let text_x = x + (TILE_SIZE - text_dims.width) / 2.0;
            let text_y = y + (TILE_SIZE + text_dims.height) / 2.0;

            draw_text_ex(
                text,
                text_x,
                text_y,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: font_size as u16,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }
    }

    fn draw_tile_content(&self, x: f32, y: f32, tile: Tile, text_color: Color) {
        let letter = match tile {
            Tile::Blank => "",
            _ => {
                let letter_char = format!("{:?}", tile);
                Box::leak(letter_char.into_boxed_str())
            }
        };

        let font_size = TILE_SIZE * 0.6;
        let text_dims = measure_text(letter, self.font.as_ref(), font_size as u16, 1.0);
        let text_x = x + (TILE_SIZE - text_dims.width) / 2.0;
        let text_y = y + (TILE_SIZE + text_dims.height) / 2.0;

        draw_text_ex(
            letter,
            text_x,
            text_y,
            TextParams {
                font: self.font.as_ref(),
                font_size: font_size as u16,
                color: text_color,
                ..Default::default()
            },
        );

        let points = tile.points().to_string();

        if points == "0" {
            return;
        }

        let small_font_size = TILE_SIZE * 0.25;
        let points_x = x + TILE_SIZE - TILE_SIZE * 0.25;
        let points_y = y + TILE_SIZE - TILE_SIZE * 0.1;

        draw_text_ex(
            &points,
            points_x,
            points_y,
            TextParams {
                font: self.font.as_ref(),
                font_size: small_font_size as u16,
                color: text_color,
                ..Default::default()
            },
        );
    }

    // magic rounded rect
    pub fn draw_rounded_rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
        draw_rectangle(x + r, y, w - 2.0 * r, h, color);
        draw_rectangle(x, y + r, w, h - 2.0 * r, color);
        draw_circle(x + r, y + r, r, color);
        draw_circle(x + w - r, y + r, r, color);
        draw_circle(x + r, y + h - r, r, color);
        draw_circle(x + w - r, y + h - r, r, color);
    }
}
