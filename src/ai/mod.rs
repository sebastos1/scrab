/*
input:
- board: 15*15*(26+1) one-hot board state (after a play)
- bag: counts [int; 27]
- tanh((my_score - opponent_score)/100) (-1 to 1)
- tiles remaining on rack: counts [int; 27]
- scoreless_turns / 6.0 (0 to 1)

so basically, we pass:
- move: board state after move, updated scores, rack with used tiles removed (100s-1000s)
- exchange: board as is, with exchanged tiles removed (127)
- pass: board as is, no tiles removed (1)
then we pick the best out of these! beautiful

output:
- scalar for how good this action is
*/

pub mod data;
pub mod network;
pub mod training;

use crate::game::Game;
use crate::{BOARD_SIZE, Pos};
use candle_core::{Device, Result, Tensor};

use network::{BOARD_CHANNELS, FEATURES};

pub fn games_to_tensors(device: &Device, games: &[Game]) -> Result<(Tensor, Tensor)> {
    let batch_size = games.len();

    let mut board_data = Vec::with_capacity(batch_size * BOARD_CHANNELS * BOARD_SIZE * BOARD_SIZE);
    let mut global_data = Vec::with_capacity(batch_size * FEATURES);

    for game in games {
        let (board_tensor, global_tensor) = game_to_tensors(device, game)?;
        let board_flat = board_tensor.flatten_all()?.to_vec1::<f32>()?;
        let global_flat = global_tensor.flatten_all()?.to_vec1::<f32>()?;
        board_data.extend(board_flat);
        global_data.extend(global_flat);
    }

    let board_batch = Tensor::from_vec(board_data, &[batch_size, BOARD_CHANNELS, BOARD_SIZE, BOARD_SIZE], device)?;
    let global_batch = Tensor::from_vec(global_data, &[batch_size, FEATURES], device)?;

    Ok((board_batch, global_batch))
}

// TODO OPPONENT TILES, DRY UP
pub fn game_to_tensors(device: &Device, game: &Game) -> Result<(Tensor, Tensor)> {
    let mut board_data = vec![0f32; BOARD_SIZE * BOARD_SIZE];
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if let Some(tile) = game.board.get_board_tile(Pos::new(row, col)) {
                board_data[row * BOARD_SIZE + col] = (tile.to_index() as f32 + 1.0) / 28.0;
            }
        }
    }

    // global data
    let mut global_data = vec![0f32; FEATURES];

    // rack -> global
    let rack = &game.racks[game.current_player];
    for tile in rack.tiles() {
        global_data[tile.to_index() as usize] += 1.0 / 7.0;
    }

    for (tile, count) in game.bag.get_tile_counts() {
        global_data[27 + tile.to_index() as usize] = count as f32 / 12.0;
    }

    // score gap -> global
    let my_score = game.scores[game.current_player] as f32;
    let opp_score = game.scores[1 - game.current_player] as f32;
    global_data[54] = ((my_score - opp_score) / 100.0).tanh();

    // scoreless turns -> global
    global_data[55] = game.zeroed_turns as f32 / 6.0;

    let board_tensor = Tensor::from_vec(board_data, &[1, BOARD_CHANNELS, BOARD_SIZE, BOARD_SIZE], device)?;
    let global_tensor = Tensor::from_vec(global_data, &[1, FEATURES], device)?;

    Ok((board_tensor, global_tensor))
}
