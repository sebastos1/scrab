use macroquad::prelude::*;

use crate::engine::moves::DebugThings;

impl super::UI {
    pub fn draw_debug_info(&self, debug: &DebugThings) {
        let start_x = super::MARGIN;
        let start_y = super::MARGIN;
        let mouse_pos = mouse_position();

        for &(row, col) in &debug.horizontal_anchors {
            let cell_x = start_x + col as f32 * super::board::CELL_SIZE;
            let cell_y = start_y + row as f32 * super::board::CELL_SIZE;
            let center_x = cell_x + super::board::CELL_SIZE / 2.0;
            let center_y = cell_y + super::board::CELL_SIZE / 2.0;
            draw_circle(center_x, center_y, 4.0, RED);
        }

        for &(row, col) in &debug.vertical_anchors {
            let cell_x = start_x + col as f32 * super::board::CELL_SIZE;
            let cell_y = start_y + row as f32 * super::board::CELL_SIZE;
            let center_x = cell_x + super::board::CELL_SIZE / 2.0;
            let center_y = cell_y + super::board::CELL_SIZE / 2.0;
            draw_circle(center_x, center_y, 4.0, BLUE);
        }

        if mouse_pos.0 >= start_x && mouse_pos.1 >= start_y {
            let col = ((mouse_pos.0 - start_x) / super::board::CELL_SIZE) as usize;
            let row = ((mouse_pos.1 - start_y) / super::board::CELL_SIZE) as usize;

            if col < 15 && row < 15 {
                if let Some(bits) = debug.horizontal_allowed_ext.get(&(row, col)) {
                    self.draw_valid_letters(mouse_pos.0, mouse_pos.1, *bits, "Horizontal");
                }
                if let Some(bits) = debug.vertical_allowed_ext.get(&(row, col)) {
                    self.draw_valid_letters(mouse_pos.0, mouse_pos.1, *bits, "Vertical");
                }
            }
        }
    }

    fn draw_valid_letters(&self, x: f32, y: f32, bits: u32, label: &str) {
        let mut valid_chars = String::new();
        for i in 0..26 {
            if (bits & (1 << i)) != 0 {
                valid_chars.push((b'A' + i as u8) as char);
            }
        }

        draw_text_ex(
            &format!("{}: {}", label, valid_chars),
            x,
            y,
            TextParams {
                font: self.font.as_ref(),
                font_size: 12,
                color: BLACK,
                ..Default::default()
            },
        );
    }
}
