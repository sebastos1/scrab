use bincode::{Decode, Encode};

use crate::ai::network::Network;
use crate::engine::moves::Move;
use crate::engine::moves::MoveGenerator;
use crate::game::Game;
use crate::game::rack::Rack;
use crate::game::tile::Tile;
use std::path::Path;

pub fn save_training_data(data: &[TrainingPosition], path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = bincode::encode_to_vec(data, bincode::config::standard())?;
    std::fs::write(path, encoded)?;
    Ok(())
}

pub fn load_training_data(path: &Path) -> Result<Vec<TrainingPosition>, Box<dyn std::error::Error>> {
    let data = std::fs::read(path)?;
    let (examples, _len): (Vec<TrainingPosition>, usize) = bincode::decode_from_slice(&data, bincode::config::standard())?;
    Ok(examples)
}

use std::sync::Arc;
use std::thread;
pub fn setup_training_data(network: Network, num_games: usize, save_path: &str) {
    let num_threads = 8;
    let games_per_thread = num_games / num_threads;
    let network = Arc::new(network);

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let network_clone = Arc::clone(&network);
        let games_to_run = if thread_id == num_threads - 1 {
            games_per_thread + (num_games % num_threads)
        } else {
            games_per_thread
        };

        let handle = thread::spawn(move || {
            println!("Thread {} starting {} games", thread_id, games_to_run);
            collect_training_data(&network_clone, games_to_run)
        });

        handles.push(handle);
    }

    let mut all_training_data = Vec::new();
    for handle in handles {
        let thread_data = handle.join().unwrap();
        all_training_data.extend(thread_data);
    }

    println!("Collected {} total training examples", all_training_data.len());
    save_training_data(&all_training_data, Path::new(save_path)).unwrap();
}

pub fn collect_training_data(network: &Network, num_games: usize) -> Vec<TrainingPosition> {
    let mut training_data = Vec::new();

    for i in 0..num_games {
        let timer = std::time::Instant::now();
        println!("Playing game {}/{}", i, num_games);

        let mut game_positions = Vec::new();
        let mut game = Game::init();

        // collection positions
        while !game.is_over() {
            game_positions.push((game.clone(), game.current_player));

            let moves = MoveGenerator::run(game.board.clone(), game.racks[game.current_player].clone());

            match get_best_move(network, &game, &moves) {
                Action::Move(mv) => game.play_move(&mv),
                Action::Swap(tiles) => {
                    game.exchange(tiles);
                }
                Action::Pass => game.pass_turn(),
            }
        }

        // each position is bundled with the game outcome
        let (winner, _) = game.end_game();
        for (position, player) in game_positions {
            let target = match winner {
                Some(w) if w == player => 1.0, // this player won
                Some(_) => -1.0,               // this player lost
                None => 0.0,                   // draw
            };
            training_data.push(TrainingPosition {
                position,
                target_value: target,
            });
        }

        let elapsed = timer.elapsed();
        println!("Game finished in {:.2?}", elapsed);
    }

    training_data
}

pub fn get_best_move(network: &Network, game: &Game, moves: &[Move]) -> Action {
    let mut all_games = Vec::new();
    let mut actions = Vec::new();

    for mv in moves {
        all_games.push(game.simulate_move(mv));
        actions.push(Action::Move(mv.clone()));
    }

    // passing
    // all_games.push(game.clone());
    // actions.push(Action::Pass);

    // let rack_tiles: Vec<_> = game.racks[game.current_player].tiles.into_iter().collect();
    // for num_tiles in 1..=rack_tiles.len().min(7) {
    //     let tiles_to_exchange = rack_tiles[..num_tiles].to_vec();
    //     if let Some(exchange_game) = simulate_swap(game, tiles_to_exchange.clone()) {
    //         all_games.push(exchange_game);
    //         actions.push(Action::Swap(tiles_to_exchange));
    //     }
    // }

    if actions.is_empty() {
        return Action::Pass;
    }

    if let Ok(evaluations) = network.evaluate_batch(&all_games) {
        let best_idx = evaluations
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index)
            .unwrap_or(actions.len() - 1);

        actions[best_idx].clone()
    } else {
        Action::Pass
    }
}
