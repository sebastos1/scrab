use super::CHANNELS;
use crate::{
    BOARD_SIZE,
    ai::{game_to_tensor, games_to_batch_tensor},
    game::Game,
};
use candle_core::{D, DType, Device, Result, Tensor};
use candle_nn::{BatchNorm, BatchNormConfig, Conv2d, Conv2dConfig, Linear, ModuleT, VarBuilder, VarMap, batch_norm, conv2d, linear};

// network
const NUM_FILTERS: usize = 64;
const NUM_BLOCKS: usize = 10;
const VALUE_HEAD_DIM: usize = 128;
const BATCH_SIZE: usize = 1024;

pub struct Network {
    device: Device,
    initial_conv: Conv2d,
    initial_bn: BatchNorm,
    res_blocks: Vec<ResBlock>,
    value_conv: Conv2d,
    value_bn: BatchNorm,
    fc1: Linear,
    fc2: Linear,
    fc_out: Linear,
    _varmap: VarMap,
}

impl Network {
    pub fn init() -> Result<Self> {
        Network::new(VarMap::new())
    }

    pub fn new(varmap: VarMap) -> Result<Self> {
        let device = Device::new_cuda(0).unwrap_or(Device::Cpu);
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let cfg = Conv2dConfig {
            padding: 1,
            ..Default::default()
        };
        let initial_conv = conv2d(CHANNELS, NUM_FILTERS, 3, cfg, vb.pp("initial"))?;
        let initial_bn = batch_norm(NUM_FILTERS, BatchNormConfig::default(), vb.pp("initial_bn"))?;

        // residual blocks
        let mut res_blocks = Vec::new();
        for i in 0..NUM_BLOCKS {
            res_blocks.push(ResBlock::new(vb.pp(format!("block{}", i)))?);
        }

        // value
        let value_cfg = Conv2dConfig {
            padding: 0,
            ..Default::default()
        };
        let value_conv = conv2d(NUM_FILTERS, 1, 1, value_cfg, vb.pp("value_conv"))?;
        let value_bn = batch_norm(1, BatchNormConfig::default(), vb.pp("value_bn"))?;
        let fc1 = linear(BOARD_SIZE * BOARD_SIZE, VALUE_HEAD_DIM, vb.pp("fc1"))?;
        let fc2 = linear(VALUE_HEAD_DIM, VALUE_HEAD_DIM / 2, vb.pp("fc2"))?;
        let fc_out = linear(VALUE_HEAD_DIM / 2, 1, vb.pp("fc_out"))?;

        Ok(Self {
            device,
            initial_conv,
            initial_bn,
            res_blocks,
            value_conv,
            value_bn,
            fc1,
            fc2,
            fc_out,
            _varmap: varmap.clone(),
        })
    }

    pub fn evaluate_batch(&self, games: &[Game]) -> Result<Vec<f32>> {
        let mut all_results = Vec::with_capacity(games.len());

        for chunk in games.chunks(BATCH_SIZE) {
            let batch_tensor = games_to_batch_tensor(&self.device, chunk)?;
            let batch_result = self.forward_batch(&batch_tensor, false)?;
            let chunk_results = batch_result.to_vec1::<f32>()?;
            all_results.extend(chunk_results);
        }

        Ok(all_results)
    }

    pub fn forward_batch(&self, input: &Tensor, train: bool) -> Result<Tensor> {
        let x = input.apply(&self.initial_conv)?;
        let x = self.initial_bn.forward_t(&x, train)?.relu()?;

        let mut x = x;
        for block in &self.res_blocks {
            x = block.forward_t(&x, train)?;
        }

        let x = x.apply(&self.value_conv)?;
        let x = self.value_bn.forward_t(&x, train)?.relu()?;
        let x = x.flatten(1, D::Minus1)?;
        let x = x.apply(&self.fc1)?.relu()?;
        let x = x.apply(&self.fc2)?.relu()?;
        let values = x.apply(&self.fc_out)?;

        values.squeeze(D::Minus1)
    }

    pub fn _evaluate_position(&self, game: &Game) -> Result<f32> {
        self._forward(&game_to_tensor(&self.device, game)?)
    }

    pub fn _forward(&self, input: &Tensor) -> Result<f32> {
        let batch_result = self.forward_batch(input, false)?;
        batch_result.squeeze(0)?.to_scalar::<f32>()
    }
}

struct ResBlock {
    conv1: Conv2d,
    bn1: BatchNorm,
    conv2: Conv2d,
    bn2: BatchNorm,
}

impl ResBlock {
    fn new(vb: VarBuilder) -> Result<Self> {
        let cfg = Conv2dConfig {
            padding: 1,
            ..Default::default()
        };
        let conv1 = conv2d(NUM_FILTERS, NUM_FILTERS, 3, cfg, vb.pp("conv1"))?;
        let bn1 = batch_norm(NUM_FILTERS, BatchNormConfig::default(), vb.pp("bn1"))?;
        let conv2 = conv2d(NUM_FILTERS, NUM_FILTERS, 3, cfg, vb.pp("conv2"))?;
        let bn2 = batch_norm(NUM_FILTERS, BatchNormConfig::default(), vb.pp("bn2"))?;
        Ok(Self { conv1, bn1, conv2, bn2 })
    }

    fn forward_t(&self, xs: &Tensor, train: bool) -> Result<Tensor> {
        let out = xs.apply(&self.conv1)?;
        let out = self.bn1.forward_t(&out, train)?.relu()?;
        let out = out.apply(&self.conv2)?;
        let out = self.bn2.forward_t(&out, train)?;
        (out + xs)?.relu() // residual + relu
    }
}
