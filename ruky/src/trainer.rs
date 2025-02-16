// This module contains components for a trainer.

use crate::err::RukyErr;
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
    check_point_step: Option<usize>,
    // The batch size to use during training.
    training_batch_size: usize,
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

impl<B: Backend> TrainerBuilder<B> {
    pub fn new() -> Self {
        let num_threads = num_cpus::get();
        Self {
            board: None,
            device: None,
            sims: 800,
            max_moves: 300,
            use_noise: true,
            sample_action: true,
            inference_batch_size: num_threads,
            num_workers: num_threads,
            num_games: None,
            check_point_dir: None,
            check_point_step: None,
            training_batch_size: num_threads,
        }
    }

    pub fn board(mut self, board: Board) -> Self {
        self.board.replace(board);
        self
    }

    pub fn device(mut self, device: Device<B>) -> Self {
        self.device.replace(device);
        self
    }

    pub fn sims(mut self, sims: u32) -> Self {
        self.sims = sims;
        self
    }

    pub fn max_moves(mut self, max_moves: u32) -> Self {
        self.max_moves = max_moves;
        self
    }

    pub fn use_noise(mut self, use_noise: bool) -> Self {
        self.use_noise = use_noise;
        self
    }

    pub fn sample_action(mut self, sample_action: bool) -> Self {
        self.sample_action = sample_action;
        self
    }

    pub fn inference_batch_size(mut self, batch_size: usize) -> Self {
        self.inference_batch_size = batch_size;
        self
    }

    pub fn num_workers(mut self, num_workers: usize) -> Self {
        self.num_workers = num_workers;
        self
    }

    pub fn num_games(mut self, num_games: usize) -> Self {
        self.num_games.replace(num_games);
        self
    }

    pub fn check_point_dir(mut self, check_point_dir: PathBuf) -> Self {
        self.check_point_dir.replace(check_point_dir);
        self
    }

    pub fn check_point_step(mut self, check_point_step: usize) -> Self {
        self.check_point_step.replace(check_point_step);
        self
    }

    pub fn training_batch_size(mut self, batch_size: usize) -> Self {
        self.training_batch_size = batch_size;
        self
    }

    pub fn build(self) -> Result<Trainer<B>, RukyErr> {
        if self.board.is_none() || self.device.is_none() || self.num_games.is_none() {
            return Err(RukyErr::PreconditionErr);
        }

        if (self.check_point_dir.is_some() && self.check_point_step.is_none())
            || (self.check_point_dir.is_none() && self.check_point_step.is_some())
        {
            return Err(RukyErr::PreconditionErr);
        }

        Ok(Trainer {
            board: self.board.unwrap(),
            device: self.device.unwrap(),
            sims: self.sims,
            max_moves: self.max_moves,
            use_noise: self.use_noise,
            sample_action: self.sample_action,
            inference_batch_size: self.inference_batch_size,
            num_workers: self.num_workers,
            num_games: self.num_games.unwrap(),
            check_point_dir: self.check_point_dir,
            check_point_step: self.check_point_step,
            training_batch_size: self.training_batch_size,
        })
    }
}
