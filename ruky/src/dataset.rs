use crate::board::Board;
use crate::game::{GameResult, GameWinner};
use crate::piece::Color;
use crate::search::Mp;
use crate::tensor_encoder::{AzEncoder, TensorEncoder};
use burn::{
    data::{dataloader::batcher::Batcher, dataset::Dataset},
    prelude::{Backend, Tensor, TensorData},
};

// The game position is a struct with all the information needed to construct
// the inputs and targets used to train the neural network.
pub(crate) struct GamePosition {
    // The board represents the board position, including board state, e.g.
    // color to move next.
    pub board: Board,
    // The legal moves in the current positions, with visit counts. These are
    // used to create the target policy tensors.
    pub moves: Vec<Mp>,
    // If game winner is White, and player to move is black, then the value for
    // the current move is -1. If white is next to move, then value is 1, and if
    // this is a draw then target valulue is 0.
    pub winner: GameWinner,
}

#[derive(Clone)]
pub(crate) struct GamesDataset {
    dataset: Vec<GameResult>,
    game_positions: usize,
}

impl GamesDataset {
    pub fn new(dataset: Vec<GameResult>) -> Self {
        let game_positions = dataset
            .iter()
            .fold(0, |acc, game_result| acc + game_result.moves.len());
        Self {
            dataset,
            game_positions,
        }
    }
}

impl Dataset<GamePosition> for GamesDataset {
    fn get(&self, mut index: usize) -> Option<GamePosition> {
        for game_result in &self.dataset {
            if index >= game_result.moves.len() {
                index -= game_result.moves.len();
                continue;
            }
            match game_result.moves.get(index) {
                None => return None,
                Some(search_result) => {
                    return Some(GamePosition {
                        board: search_result.board.clone(),
                        moves: search_result.moves.clone(),
                        winner: game_result.winner,
                    })
                }
            }
        }
        None
    }

    fn len(&self) -> usize {
        self.game_positions
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

impl<B: Backend> Batcher<B, GamePosition, GamesBatch<B>> for GamesBatcher {
    fn batch(&self, games: Vec<GamePosition>, device: &B::Device) -> GamesBatch<B> {
        let encoder = AzEncoder::new(device.clone());

        let n = games.len();
        let mut inputs = Vec::with_capacity(n);
        let mut targets_policy = Vec::with_capacity(n);
        let mut targets_value = Vec::with_capacity(n);

        for GamePosition {
            board,
            moves,
            winner,
        } in games
        {
            inputs.push(encoder.encode_board(&board));
            targets_policy.push(encoder.encode_mps(&moves));
            let value = match winner {
                GameWinner::Draw => 0.0,
                GameWinner::White => match board.color() {
                    Color::White => 1.0,
                    _ => -1.0,
                },
                _ => match board.color() {
                    Color::Black => 1.0,
                    _ => -1.0,
                },
            };
            targets_value.push(value);
        }

        let inputs = Tensor::cat(inputs, 0);
        let targets_policy = Tensor::cat(targets_policy, 0);
        let targets_value = Tensor::from_data(TensorData::new(targets_value, [n, 1]), device);

        GamesBatch {
            inputs,
            targets: (targets_policy, targets_value),
        }
    }
}
