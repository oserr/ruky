// net is a module containing the building blocks of the neural network used in
// AlphaZero.

use crate::dataset::GamesBatch;
use burn::{
    module::Module,
    nn::{
        conv::{Conv2d, Conv2dConfig},
        loss::{MseLoss, Reduction::Mean},
        BatchNorm, BatchNormConfig, Initializer, Linear, LinearConfig, PaddingConfig2d,
    },
    prelude::{Backend, Device, Tensor},
    tensor::activation::{relu, tanh},
};

//---------------
// Residual Block
//---------------

// ResBlockNet implements the residual block in the AlphaZero network, which
// has 19 of these blocks connected together.
#[derive(Debug, Module)]
struct ResBlockNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    batch_norm1: BatchNorm<B>,
    batch_norm2: BatchNorm<B>,
}

impl<B: Backend> ResBlockNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        let conv1 = Conv2dConfig::new([256, 256], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .with_initializer(Initializer::KaimingNormal {
                gain: 0.5,
                fan_out_only: true,
            })
            .init(device);
        let conv2 = Conv2dConfig::new([256, 256], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .with_initializer(Initializer::KaimingNormal {
                gain: 0.5,
                fan_out_only: true,
            })
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

    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let orig_x = x.clone();
        let x = self.conv1.forward(x);
        let x = self.batch_norm1.forward(x);
        let x = relu(x);
        let x = self.conv2.forward(x);
        let x = self.batch_norm2.forward(x);
        let x = x + orig_x;
        relu(x)
    }
}

//----------------
// Policy head net
//----------------

#[derive(Debug, Module)]
struct PolicyNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    batch_norm: BatchNorm<B>,
}

impl<B: Backend> PolicyNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        Self {
            conv1: Conv2dConfig::new([256, 256], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .with_initializer(Initializer::KaimingNormal {
                    gain: 0.5,
                    fan_out_only: true,
                })
                .init(device),
            conv2: Conv2dConfig::new([256, 73], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .with_initializer(Initializer::KaimingNormal {
                    gain: 0.5,
                    fan_out_only: true,
                })
                .init(device),
            batch_norm: BatchNormConfig::new(256).init(device),
        }
    }

    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv1.forward(x);
        let x = self.batch_norm.forward(x);
        let x = relu(x);
        let x = self.conv2.forward(x);
        let x = x.reshape([0, 8, 8, 73]);
        x
    }
}

//----------------
// Value head net
//----------------

#[derive(Debug, Module)]
struct ValueNet<B: Backend> {
    conv: Conv2d<B>,
    batch_norm: BatchNorm<B>,
    fc1: Linear<B>,
    fc2: Linear<B>,
}

impl<B: Backend> ValueNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        Self {
            conv: Conv2dConfig::new([256, 1], [1, 1])
                .with_padding(PaddingConfig2d::Same)
                .with_initializer(Initializer::KaimingNormal {
                    gain: 0.5,
                    fan_out_only: true,
                })
                .init(device),
            batch_norm: BatchNormConfig::new(1).init(device),
            fc1: LinearConfig::new(64, 256)
                .with_initializer(Initializer::KaimingNormal {
                    gain: 0.5,
                    fan_out_only: true,
                })
                .init(device),
            fc2: LinearConfig::new(256, 1)
                .with_initializer(Initializer::KaimingNormal {
                    gain: 0.5,
                    fan_out_only: true,
                })
                .init(device),
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

//--------------
// AlphaZero net
//--------------

#[derive(Debug, Module)]
pub struct AlphaZeroNet<B: Backend> {
    conv: Conv2d<B>,
    batch_norm: BatchNorm<B>,
    res_blocks: Vec<ResBlockNet<B>>,
    policy_net: PolicyNet<B>,
    value_net: ValueNet<B>,
}

#[derive(Clone, Debug)]
pub struct AzOutput<B: Backend> {
    pub loss: Tensor<B, 1>,
    pub out_policy: Tensor<B, 4>,
    pub out_values: Tensor<B, 2>,
    pub targets_policy: Tensor<B, 4>,
    pub targets_values: Tensor<B, 2>,
}

impl<B: Backend> AlphaZeroNet<B> {
    pub fn new(device: &Device<B>) -> Self {
        Self {
            conv: Conv2dConfig::new([119, 256], [3, 3])
                .with_padding(PaddingConfig2d::Same)
                .with_initializer(Initializer::Normal {
                    mean: 0.0,
                    std: 1.0,
                })
                .init(device),
            batch_norm: BatchNormConfig::new(256).init(device),
            res_blocks: vec![ResBlockNet::new(device); 19],
            policy_net: PolicyNet::new(device),
            value_net: ValueNet::new(device),
        }
    }

    pub fn forward(&self, x: Tensor<B, 4>) -> (Tensor<B, 4>, Tensor<B, 2>) {
        let x = self.conv.forward(x);
        let x = self.batch_norm.forward(x);
        let x = self
            .res_blocks
            .iter()
            .fold(x, |acc, res_block| res_block.forward(acc));
        let policy = self.policy_net.forward(x.clone());
        let value = self.value_net.forward(x);
        (policy, value)
    }

    pub fn forward_step(&self, batch: GamesBatch<B>) -> AzOutput<B> {
        let (_out_policy, out_values) = self.forward(batch.inputs);

        let _mse_loss =
            MseLoss::new().forward(out_values.clone(), batch.targets_values.clone(), Mean);
        // TODO: Need to compute the cross entropy loss of the policy values, but we
        // need to change the network to return the logits rather than the
        // probabilities, and then we'll need to reshape tensor from [8, 8, 73]
        // -> [8 * 8 * 73].
        todo!();

        // AzOutput {
        //     loss,
        //     out_policy,
        //     out_values,
        //     targets.targets_policy,
        //     targets.targets_values,
        // }
    }
}
