// net is a module containing the building blocks of the neural network used in AlphaZero.

use burn::{
    nn::{
        conv::{Conv2d, Conv2dConfig},
        BatchNorm, BatchNormConfig, PaddingConfig2d,
    },
    prelude::{Backend, Device, Tensor},
    tensor::activation::relu,
};

//---------------
// Residual Block
//---------------

// ResBlockNset implements the residual block in the AlphaZero network, which has 19 of these
// blocks connected together.
struct ResBlockNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    batch_norm1: BatchNorm<B, 2>,
    batch_norm2: BatchNorm<B, 2>,
}

impl<B: Backend> ResBlockNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        let conv1 = Conv2dConfig::new([256, 256], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let conv2 = Conv2dConfig::new([256, 256], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let batch_norm1 = BatchNormConfig::new(256).init(device);
        let batch_norm2 = BatchNormConfig::new(256).init(device);

        Self {
            conv1,
            conv2,
            batch_norm1,
            batch_norm2,
        }
    }

    pub fn formward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv1.forward(x);
        let x = self.batch_norm1.forward(x);
        let x = relu(x);
        let x = self.conv2.forward(x);
        let x = self.batch_norm2.forward(x);
        let x = x.clone() + x;
        relu(x)
    }
}
