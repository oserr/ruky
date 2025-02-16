// This module contains components for a trainer.

use crate::Board;
use burn::prelude::{Backend, Device};
use std::path::PathBuf;

// The purpose of the Trainer is to play games of self-play to generate training
// data, and to train the model with the data generated during self-play.
pub struct Trainer<B: Backend> {
    // The initial starting position.
    board: Board,
    // The device to use for neural network computations.
    device: Device<B>,
    // The total number of simulations per move.
    sims: u32,
    // The maximum number of moves to play before declaring draw.
    max_moves: u32,
    // If true, noise is added to the root node priors.
    use_noise: bool,
    // If true, the best move is selected by sampling, otherwise visit count
    // is used to select the best move.
    sample_action: bool,
    // The batch size to use during inference.
    inference_batch_size: usize,
    // The number of worker threads to use in search.
    num_workers: usize,
    // The number of games to play for training.
    num_games: usize,
    // The path to the directory to use for checkpoints.
    check_point_dir: Option<PathBuf>,
    // The number of training steps to run before creating a checkpoint.
    check_point_step: usize,
    // The batch size to use during training.
    batch_size: usize,
}

// The purpose of the Trainer is to play games of self-play to generate training
// data, and to train the model with the data generated during self-play.
pub struct TrainerBuilder<B: Backend> {
    // The initial starting position.
    board: Option<Board>,
    // The device to use for neural network computations.
    device: Option<Device<B>>,
    // The total number of simulations per move.
    sims: u32,
    // The maximum number of moves to play before declaring draw.
    max_moves: u32,
    // If true, noise is added to the root node priors.
    use_noise: bool,
    // If true, the best move is selected by sampling, otherwise visit count
    // is used to select the best move.
    sample_action: bool,
    // The batch size to use during inference.
    inference_batch_size: usize,
    // The number of worker threads to use in search.
    num_workers: usize,
    // The number of games to play for training.
    num_games: Option<usize>,
    // The path to the directory to use for checkpoints.
    check_point_dir: Option<PathBuf>,
    // The number of training steps to run before creating a checkpoint.
    check_point_step: Option<usize>,
    // The batch size to use during training.
    training_batch_size: usize,
}
