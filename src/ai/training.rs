use crate::ai::data::PositionsReader;
use crate::ai::network::Network;
use candle_core::Tensor;
use candle_nn::{AdamW, Optimizer, ParamsAdamW, loss};
use rand::{Rng, prelude::SliceRandom};

const BATCH_SIZE: usize = 512;
const EPOCHS: usize = 10;

fn train_batch(
    network: &mut Network,
    optimizer: &mut AdamW,
    board_batch: &Tensor,
    global_batch: &Tensor,
    targets: &[f32],
) -> Result<(), Box<dyn std::error::Error>> {
    let target_tensor = Tensor::from_vec(targets.to_vec(), &[targets.len()], &network.device)?;
    let predictions = network.forward(board_batch, global_batch, true)?;
    let loss = loss::mse(&predictions, &target_tensor)?;
    optimizer.backward_step(&loss)?;
    println!("Batch loss: {:.6}", loss.to_scalar::<f32>()?);
    Ok(())
}

pub fn train(network: &mut Network, positions_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let reader = PositionsReader::open(positions_path)?;
    println!("Loaded {} positions", reader.len());

    let mut optimizer = AdamW::new(
        network.varmap.all_vars(),
        ParamsAdamW {
            lr: 0.001,
            ..Default::default()
        },
    )?;

    let mut rng = rand::rng();
    for epoch in 0..EPOCHS {
        let mut train_indices: Vec<usize> = (0..reader.len()).collect();
        train_indices.shuffle(&mut rng);

        for batch_indices in train_indices.chunks(BATCH_SIZE) {
            let mut board_data = Vec::with_capacity(batch_indices.len() * 225);
            let mut global_data = Vec::with_capacity(batch_indices.len() * 56);
            let mut targets = Vec::with_capacity(batch_indices.len());

            for &idx in batch_indices {
                let pos = reader.get(idx).unwrap();

                for row in 0..15 {
                    for col in 0..15 {
                        let val = if rng.random_bool(0.5) {
                            pos.board[row][col]
                        } else {
                            pos.board[col][row]
                        };
                        board_data.push(if val == 0 { 0.0 } else { val as f32 / 27.0 });
                    }
                }

                for &count in &pos.rack_counts {
                    global_data.push(count as f32 / 7.0);
                }
                for &count in &pos.bag_counts {
                    global_data.push(count as f32 / 12.0);
                }
                global_data.push(((pos.my_score - pos.opp_score) as f32 / 100.0).tanh());
                global_data.push(pos.scoreless_turns as f32 / 6.0);
                targets.push(pos.target_equity / 100.0); // squish
            }

            let board_tensor = Tensor::from_vec(board_data, &[batch_indices.len(), 1, 15, 15], &network.device)?;
            let global_tensor = Tensor::from_vec(global_data, &[batch_indices.len(), 56], &network.device)?;
            train_batch(network, &mut optimizer, &board_tensor, &global_tensor, &targets)?;
        }
        if epoch % 10 == 0 {
            network.save(&format!("models/checkpoint.safetensors"))?;
        }
        println!("Epoch {} complete", epoch);
        let lr = 0.001 * 0.95_f64.powi(epoch as i32); // decay
        optimizer.set_learning_rate(lr);
    }

    network.save(&format!("models/model.safetensors"))?;
    Ok(())
}
