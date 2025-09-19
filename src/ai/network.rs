use crate::BOARD_SIZE;
use candle_core::{D, DType, Device, Result, Tensor};
use candle_nn::{BatchNorm, BatchNormConfig, Conv2d, Conv2dConfig, Linear, ModuleT, VarBuilder, VarMap, batch_norm, conv2d, linear};

// network
const NUM_FILTERS: usize = 64;
const NUM_BLOCKS: usize = 6;

pub const LETTER_TYPES: usize = 27; // 26 letters + 1 blank
const RACK_SIZE: usize = LETTER_TYPES; // counts of each letter in rack
const BAG_SIZE: usize = LETTER_TYPES; // counts of each letter in bag

pub const BOARD_CHANNELS: usize = 1;
pub const FEATURES: usize = RACK_SIZE + BAG_SIZE + 1 + 1;

pub struct Network {
    pub device: Device,
    board_conv: Conv2d,
    res_blocks: Vec<ResBlock>,
    fc: Linear,
    combined_conv: Conv2d,
    combined_bn: BatchNorm,
    value_conv: Conv2d,
    fc_out: Linear,
    pub varmap: VarMap,
}

impl Network {
    pub fn init() -> Result<Self> {
        Network::new(VarMap::new())
    }

    pub fn save(&self, path: &str) -> Result<()> {
        self.varmap.save(path)
    }

    pub fn load(path: &str) -> Result<Self> {
        let mut varmap = VarMap::new();
        varmap.load(path)?;
        Self::new(varmap)
    }

    pub fn new(varmap: VarMap) -> Result<Self> {
        let device = Device::new_cuda(0).unwrap_or(Device::Cpu);
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        // board
        let cfg = Conv2dConfig {
            padding: 1,
            ..Default::default()
        };
        let board_conv = conv2d(BOARD_CHANNELS, NUM_FILTERS, 3, cfg, vb.pp("initial"))?;

        // residual blocks
        let mut res_blocks = Vec::new();
        for i in 0..NUM_BLOCKS {
            res_blocks.push(ResBlock::new(vb.pp(format!("block{}", i)))?);
        }

        let fc = linear(FEATURES, NUM_FILTERS, vb.pp("global_fc"))?;

        let combined_conv = conv2d(NUM_FILTERS * 2, NUM_FILTERS, 1, Conv2dConfig::default(), vb.pp("combined_conv"))?;
        let combined_bn = batch_norm(NUM_FILTERS, BatchNormConfig::default(), vb.pp("combined_bn"))?;

        // value
        let value_cfg = Conv2dConfig {
            padding: 0,
            ..Default::default()
        };
        let value_conv = conv2d(NUM_FILTERS, 1, 1, value_cfg, vb.pp("value_conv"))?;
        let fc_out = linear(BOARD_SIZE * BOARD_SIZE, 1, vb.pp("fc_out"))?;

        Ok(Self {
            device,
            board_conv,
            res_blocks,
            fc,
            combined_conv,
            combined_bn,
            value_conv,
            fc_out,
            varmap: varmap.clone(),
        })
    }

    pub fn forward(&self, board_input: &Tensor, global_input: &Tensor, train: bool) -> Result<Tensor> {
        let batch_size = board_input.shape().dims()[0];
        let mut x = board_input.apply(&self.board_conv)?.relu()?;
        for block in &self.res_blocks {
            x = block.forward_t(&x, train)?;
        }

        let global_features = global_input.apply(&self.fc)?.relu()?;
        let global_broadcast = global_features
            .unsqueeze(2)?
            .unsqueeze(3)?
            .broadcast_as(&[batch_size, NUM_FILTERS, BOARD_SIZE, BOARD_SIZE])?;

        let combined = Tensor::cat(&[x, global_broadcast], 1)?;
        let x = combined.apply(&self.combined_conv)?;
        let x = self.combined_bn.forward_t(&x, train)?.relu()?;

        let x = x.apply(&self.value_conv)?;
        let x = x.flatten(1, D::Minus1)?;
        let values = x.apply(&self.fc_out)?;

        values.squeeze(D::Minus1)
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
