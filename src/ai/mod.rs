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

pub mod network;

use crate::engine::Pos;
use crate::game::{Game, board::BOARD_TILES};
use candle_core::{Device, Result, Tensor};

const LETTER_TYPES: usize = 27; // 26 letters + 1 blank
const RACK_SIZE: usize = LETTER_TYPES; // counts of each letter in rack
const BAG_SIZE: usize = LETTER_TYPES; // counts of each letter in bag

// channels: board(27) + rack(27) + bag(27) + score_diff(1) + scoreless_ratio(1) = 83
const CHANNELS: usize = LETTER_TYPES + RACK_SIZE + BAG_SIZE + 1 + 1;

pub fn games_to_batch_tensor(device: &Device, games: &[Game]) -> Result<Tensor> {
    let batch_size = games.len();

    // check dimensions based on one game
    let first_tensor = game_to_tensor(&device, &games[0])?;
    let tensor_shape = first_tensor.shape();

    let batch_shape = [
        batch_size,
        tensor_shape.dims()[1], // channels
        tensor_shape.dims()[2], // height
        tensor_shape.dims()[3], // width
    ];

    let mut batch_data = Vec::with_capacity(batch_size * CHANNELS * BOARD_TILES * BOARD_TILES);
    for game in games {
        let tensor = game_to_tensor(&device, game)?;
        let tensor_data = tensor.flatten_all()?.to_vec1::<f32>()?;
        batch_data.extend(tensor_data);
    }
    Tensor::from_vec(batch_data, &batch_shape, &device)
}

pub fn game_to_tensor(device: &Device, game: &Game) -> Result<Tensor> {
    // one dimensional for my sweet network
    let mut tensor_data = vec![0f32; BOARD_TILES * BOARD_TILES * CHANNELS];
    for row in 0..BOARD_TILES {
        for col in 0..BOARD_TILES {
            let base_idx = (row * BOARD_TILES + col) * CHANNELS;

            // one hot board state
            let pos = Pos::new(row, col);
            if let Some(tile) = game.board.get_tile(pos) {
                let tile_idx = tile.to_index() as usize;
                if tile_idx < LETTER_TYPES {
                    tensor_data[base_idx + tile_idx] = 1.0;
                }
            }

            // current player rack
            let rack = &game.racks[game.current_player];
            for tile in rack.tiles() {
                let rack_idx = base_idx + LETTER_TYPES + tile.to_index() as usize;
                tensor_data[rack_idx] += 1.0 / 7.0; // max rack size
            }

            // remaining bag tiles
            for tile in &game.bag.tiles {
                let bag_idx = base_idx + LETTER_TYPES * 2 + tile.to_index() as usize;
                tensor_data[bag_idx] += 1.0 / 12.0; // max tile count
            }

            // score diff
            let my_score = game.scores[game.current_player] as f32;
            let opp_score = game.scores[1 - game.current_player] as f32;
            let score_diff = ((my_score - opp_score) / 100.0).tanh();
            tensor_data[base_idx + LETTER_TYPES * 3] = score_diff;

            // scoreless turns
            let scoreless_ratio = game.zeroed_turns as f32 / 6.0;
            tensor_data[base_idx + LETTER_TYPES * 3 + 1] = scoreless_ratio;
        }
    }

    // [1, channels, height, width]
    Tensor::from_vec(tensor_data, &[1, CHANNELS, BOARD_TILES, BOARD_TILES], &device)
}
