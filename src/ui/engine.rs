use macroquad::prelude::*;

use crate::{
    engine::moves::{Direction, Move, PlayedTile},
    ui::{MARGIN, board::BOARD_SIZE},
    util::Pos,
};

impl super::UI {
    pub fn draw_move_list(&mut self, moves: &[Move]) {
        let moves_x = MARGIN + BOARD_SIZE + 30.0;
        let moves_y = MARGIN;
        let line_height = 25.0;
        let visible_moves = 20;
        let list_height = visible_moves as f32 * line_height;

        let (_scroll_x, scroll_y) = mouse_wheel();
        if scroll_y != 0.0 {
            if scroll_y > 0.0 && self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(3);
            } else if scroll_y < 0.0 && self.scroll_offset + visible_moves < moves.len() {
                self.scroll_offset += 3;
            }
        }

        draw_text_ex(
            &format!(
                "Moves: {} ({}-{})",
                moves.len(),
                self.scroll_offset + 1,
                (self.scroll_offset + visible_moves).min(moves.len())
            ),
            moves_x,
            moves_y,
            TextParams {
                font: self.font.as_ref(),
                font_size: 16,
                color: WHITE,
                ..Default::default()
            },
        );

        let scrollbar_x = moves_x + 280.0;
        let scrollbar_y = moves_y + 30.0;
        draw_rectangle(scrollbar_x, scrollbar_y, 10.0, list_height, Color::new(0.2, 0.2, 0.2, 1.0));

        if moves.len() > visible_moves {
            let thumb_height = (visible_moves as f32 / moves.len() as f32) * list_height;
            let thumb_y = scrollbar_y + (self.scroll_offset as f32 / moves.len() as f32) * list_height;
            draw_rectangle(scrollbar_x, thumb_y, 10.0, thumb_height, Color::new(0.6, 0.6, 0.6, 1.0));
        }

        let mouse_pos = mouse_position();
        self.hovered_move = None;

        let end_idx = (self.scroll_offset + visible_moves).min(moves.len());
        for (display_idx, mv) in moves[self.scroll_offset..end_idx].iter().enumerate() {
            let actual_idx = self.scroll_offset + display_idx;
            let y = moves_y + 40.0 + display_idx as f32 * line_height;

            let hover_rect = (moves_x, y - 12.0, 270.0, line_height);
            let is_hovered = mouse_pos.0 >= hover_rect.0
                && mouse_pos.0 <= hover_rect.0 + hover_rect.2
                && mouse_pos.1 >= hover_rect.1
                && mouse_pos.1 <= hover_rect.1 + hover_rect.3;

            if is_hovered {
                self.hovered_move = Some(actual_idx);
                self.draw_rounded_rect(
                    hover_rect.0 - 3.0,
                    hover_rect.1,
                    hover_rect.2,
                    hover_rect.3,
                    3.0,
                    Color::new(0.3, 0.3, 0.3, 0.6),
                );
            }

            let text_color = if is_hovered { Color::new(1.0, 0.9, 0.4, 1.0) } else { WHITE };

            let direction_arrow = match mv.direction {
                Direction::Horizontal => ">",
                Direction::Vertical => "↓",
            };

            draw_text_ex(
                &format!("{} {} ({}pts)", mv.word, direction_arrow, mv.score),
                moves_x,
                y,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: 13,
                    color: text_color,
                    ..Default::default()
                },
            );
        }

        if let Some(idx) = self.hovered_move {
            if let Some(mv) = moves.get(idx) {
                self.draw_move_preview(mv);
                self.draw_formed_words(mv);
            }
        }
    }

    pub fn draw_move_preview(&self, mv: &Move) {
        for (i, played_tile) in mv.tiles_used.iter().enumerate() {
            let (row, col) = match mv.direction {
                Direction::Horizontal => (mv.pos.row, mv.pos.col + i),
                Direction::Vertical => (mv.pos.row + i, mv.pos.col),
            };

            if row < 15 && col < 15 {
                if let PlayedTile::FromRack(tile) = played_tile {
                    let (tile_x, tile_y) = self.tile_position(Pos::new(row, col));
                    self.draw_placeable_tile(tile_x, tile_y, *tile, true);
                }
            }
        }
    }

    pub fn draw_formed_words(&self, mv: &Move) {
        if !mv.words_formed.is_empty() {
            let debug_x = MARGIN;
            let debug_y = MARGIN + BOARD_SIZE + 120.0;

            draw_text_ex(
                "Words formed:",
                debug_x,
                debug_y,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: 14,
                    color: WHITE,
                    ..Default::default()
                },
            );

            for (i, word) in mv.words_formed.iter().enumerate() {
                draw_text_ex(
                    &format!("• {}", word),
                    debug_x + (i % 4) as f32 * 120.0,
                    debug_y + 20.0 + (i / 4) as f32 * 20.0,
                    TextParams {
                        font: self.font.as_ref(),
                        font_size: 13,
                        color: Color::new(0.0, 1.0, 0.0, 1.0),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
