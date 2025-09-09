use macroquad::prelude::*;

use crate::{
    engine::{
        Pos,
        moves::{Direction, Move, PlayedTile},
    },
    game::board::BOARD_TILES,
    ui::{
        MARGIN,
        board::{BOARD_SIZE, CELL_SIZE},
    },
};

const PLAYER_HEADER_HEIGHT: f32 = 40.0;
pub const SIDEBAR_X: f32 = MARGIN * 2. + BOARD_SIZE;
pub const SIDEBAR_WIDTH: f32 = 280.0;
pub const MOVE_LIST_HEIGHT: f32 = BOARD_SIZE - PLAYER_HEADER_HEIGHT - MARGIN;
pub const MOVE_LIST_LINE_HEIGHT: f32 = 25.0;
const SCROLLBAR_WIDTH: f32 = 10.0;
const VISIBLE_MOVES: usize = (MOVE_LIST_HEIGHT / MOVE_LIST_LINE_HEIGHT) as usize;
const MOVES_HEADER_HEIGHT: f32 = 30.0;

const SCROLLBAR_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const SCROLLBAR_THUMB_COLOR: Color = Color::new(0.6, 0.6, 0.6, 1.0);
pub const TEXT_HOVER_COLOR: Color = Color::new(1.0, 0.9, 0.4, 1.0);
const MOVE_HOVER_COLOR: Color = Color::new(0.3, 0.3, 0.3, 0.6);

impl super::UI {
    pub fn draw_players(&self, scores: &[u16; 2], current_player: usize) {
        let player_header_y = MARGIN;
        for (i, &score) in scores.iter().enumerate() {
            draw_text_ex(
                &format!("Player {} - {}", i + 1, score),
                SIDEBAR_X + i as f32 * 140.0,
                player_header_y + 18.0,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: 14,
                    color: if i == current_player { TEXT_HOVER_COLOR } else { WHITE },
                    ..Default::default()
                },
            );
        }
    }

    pub fn draw_move_list(&mut self, moves: &[Move]) -> Option<usize> {
        let mut moves: Vec<_> = moves.iter().enumerate().collect();
        moves.sort_by(|a, b| b.1.score.cmp(&a.1.score));

        let (_, scroll) = mouse_wheel();
        if scroll != 0.0 {
            if scroll > 0.0 && self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            } else if scroll < 0.0 && self.scroll_offset + VISIBLE_MOVES < moves.len() {
                self.scroll_offset += 1;
            }
        }

        let move_list_y = MARGIN + PLAYER_HEADER_HEIGHT;

        draw_text_ex(
            &format!("{} moves:", moves.len(),),
            SIDEBAR_X,
            move_list_y,
            TextParams {
                font: self.font.as_ref(),
                font_size: 20,
                ..Default::default()
            },
        );

        let scrollbar_x = SIDEBAR_X + SIDEBAR_WIDTH - SCROLLBAR_WIDTH;
        let scrollbar_y = move_list_y + MOVES_HEADER_HEIGHT;
        draw_rectangle(scrollbar_x, scrollbar_y, SCROLLBAR_WIDTH, MOVE_LIST_HEIGHT, SCROLLBAR_COLOR);

        if moves.len() > VISIBLE_MOVES {
            draw_rectangle(
                scrollbar_x,
                scrollbar_y + (self.scroll_offset as f32 / moves.len() as f32) * MOVE_LIST_HEIGHT,
                SCROLLBAR_WIDTH,
                (VISIBLE_MOVES as f32 / moves.len() as f32) * MOVE_LIST_HEIGHT,
                SCROLLBAR_THUMB_COLOR,
            );
        }

        let mouse_pos = mouse_position();
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
        self.hovered_move = None;
        let mut clicked_move = None;

        let end_idx = (self.scroll_offset + VISIBLE_MOVES).min(moves.len());
        for (display_idx, (original_idx, mv)) in moves[self.scroll_offset..end_idx].iter().enumerate() {
            let actual_idx = self.scroll_offset + display_idx;
            let text_y = MARGIN + MOVES_HEADER_HEIGHT + PLAYER_HEADER_HEIGHT + 10.0 + display_idx as f32 * MOVE_LIST_LINE_HEIGHT;
            let hover_rect = (SIDEBAR_X, text_y - 10.0, SIDEBAR_WIDTH - 20.0, MOVE_LIST_LINE_HEIGHT);
            let is_hovered = mouse_pos.0 >= hover_rect.0
                && mouse_pos.0 <= hover_rect.0 + hover_rect.2
                && mouse_pos.1 >= hover_rect.1
                && mouse_pos.1 <= hover_rect.1 + hover_rect.3;

            if is_hovered {
                self.hovered_move = Some(actual_idx);
                draw_rectangle(hover_rect.0, hover_rect.1, hover_rect.2, hover_rect.3, MOVE_HOVER_COLOR);
                if mouse_clicked {
                    clicked_move = Some(*original_idx);
                    self.scroll_offset = 0;
                }
            }

            let text_color = if is_hovered { TEXT_HOVER_COLOR } else { WHITE };
            let direction_arrow = match mv.direction {
                Direction::Horizontal => ">",
                Direction::Vertical => "↓",
            };

            draw_text_ex(
                &format!("{} {} ({}pts)", mv.get_word_string(), direction_arrow, mv.score),
                SIDEBAR_X,
                text_y + 5.,
                TextParams {
                    font: self.font.as_ref(),
                    font_size: 13,
                    color: text_color,
                    ..Default::default()
                },
            );
        }

        if let Some(idx) = self.hovered_move {
            if let Some((_, mv)) = moves.get(idx) {
                self.draw_move_preview(mv);
            }
        }
        clicked_move
    }

    pub fn draw_move_preview(&self, mv: &Move) {
        let word_start_idx = mv.used_bits.trailing_zeros() as usize;
        let start_pos = match mv.direction {
            Direction::Horizontal => Pos::new(mv.pos.row, word_start_idx),
            Direction::Vertical => Pos::new(word_start_idx, mv.pos.col),
        };

        let mut tile_offset = 0;
        for i in 0..BOARD_TILES {
            if mv.used_bits & (1 << i) != 0 {
                let (row, col) = match mv.direction {
                    Direction::Horizontal => (start_pos.row, start_pos.col + tile_offset),
                    Direction::Vertical => (start_pos.row + tile_offset, start_pos.col),
                };

                if let Some(PlayedTile::FromRack(tile)) = mv.tiles_data[i].1 {
                    let (tile_x, tile_y) = self.tile_position(Pos::new(row, col));
                    self.draw_letter_tile(tile_x, tile_y, CELL_SIZE, tile, true);
                }

                tile_offset += 1;
            }
        }
    }
}
