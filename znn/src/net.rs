// net is a module containing the building blocks of the neural network used in
// AlphaZero.

use burn::{
    nn::{
        conv::{Conv2d, Conv2dConfig},
        BatchNorm, BatchNormConfig, Linear, LinearConfig, PaddingConfig2d,
    },
    prelude::{Backend, Device, Tensor},
    tensor::activation::{relu, tanh},
};

//---------------
// Residual Block
//---------------

// ResBlockNset implements the residual block in the AlphaZero network, which
// has 19 of these blocks connected together.
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

//----------------
// Policy head net
//----------------

struct PolicyNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    batch_norm: BatchNorm<B, 2>,
}

impl<B: Backend> PolicyNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        Self {
            conv1: Conv2dConfig::new([256, 256], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .init(device),
            conv2: Conv2dConfig::new([256, 73], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .init(device),
            batch_norm: BatchNormConfig::new(256).init(device),
        }
    }

    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv1.forward(x);
        let x = self.batch_norm.forward(x);
        let x = relu(x);
        self.conv2.forward(x)
    }
}

//----------------
// Value head net
//----------------

struct ValueNet<B: Backend> {
    conv: Conv2d<B>,
    batch_norm: BatchNorm<B, 2>,
    fc1: Linear<B>,
    fc2: Linear<B>,
}

impl<B: Backend> ValueNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        Self {
            conv: Conv2dConfig::new([256, 1], [1, 1])
                .with_padding(PaddingConfig2d::Same)
                .init(device),
            batch_norm: BatchNormConfig::new(1).init(device),
            fc1: LinearConfig::new(64, 256).init(device),
            fc2: LinearConfig::new(256, 1).init(device),
        }
    }

    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.conv.forward(x);
        let x = self.batch_norm.forward(x);
        let x = relu(x);
        let x = x.flatten(1, 3);
        let x = self.fc1.forward(x);
        let x = relu(x);
        let x = self.fc2.forward(x);
        tanh(x)
    }
}
