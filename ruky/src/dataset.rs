use crate::game::GameResult;
use burn::{
    data::dataset::Dataset,
    prelude::{Backend, Tensor},
};

#[derive(Clone)]
pub(crate) struct GamesDataset {
    dataset: Vec<GameResult>,
}

impl Dataset<GameResult> for GamesDataset {
    fn get(&self, index: usize) -> Option<GameResult> {
        self.dataset.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.dataset.len()
    }
}

#[derive(Clone, Debug)]
pub struct GamesBatch<B: Backend> {
    // A single input represents a game position.
    pub inputs: Tensor<B, 4>,

    // A single target represents two outputs: 1) the tensor of probabilities
    // for moves and 2) the value of a given of the given position.
    pub targets: (Tensor<B, 4>, Tensor<B, 2>),
}
