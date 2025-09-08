use candle_core::{Device, Result, Tensor};
use std::time::Instant;

fn main() -> Result<()> {
    let device = Device::new_cuda(0)?;
    println!("🔥 CUDA Stress Test Starting!");

    // Big matrix multiplication stress test
    let size = 4096;
    let a = Tensor::randn(0f32, 1.0, (size, size), &device)?;
    let b = Tensor::randn(0f32, 1.0, (size, size), &device)?;

    let start = Instant::now();

    // Chain multiple operations to stress the GPU
    for i in 0..50 {
        let c = a.matmul(&b)?;
        let c = c.relu()?;
        let c = c.powf(2.0)?;
        let _result = c.sum_all()?;

        if i % 10 == 0 {
            println!("Iteration {}/50 - GPU cooking! 🌡️", i);
        }
    }

    let duration = start.elapsed();
    println!("✅ Completed in {:.2}s - Your GPU survived!", duration.as_secs_f32());

    Ok(())
}
