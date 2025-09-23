use crate::board::Board;
use crate::game::{GameResult, GameWinner};
use crate::search::Mp;
use burn::{
    data::{dataloader::batcher::Batcher, dataset::Dataset},
    prelude::{Backend, Tensor},
};

// The game position is a struct with all the information needed to construct
// the inputs and targets used to train the neural network.
pub(crate) struct GamePosition {
    // The board represents the board position, including board state, e.g.
    // color to move next.
    pub board: Board,
    // If game winner is White, and player to move is black, then the value for
    // the current move is -1. If white is next to move, then value is 1, and if
    // this is a draw then target valulue is 0.
    pub game_winner: GameWinner,
    // The legal moves in the current positions, with visit counts. These are
    // used to create the target policy tensors.
    pub moves: Vec<Mp>,
}

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

#[derive(Clone, Copy, Debug)]
struct GamesBatcher {}

impl<B: Backend> Batcher<B, GameResult, GamesBatch<B>> for GamesBatcher {
    fn batch(&self, _games: Vec<GameResult>, _device: &B::Device) -> GamesBatch<B> {
        todo!();
    }
}
