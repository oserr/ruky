// This module contains components for a trainer.

use std::path::PathBuf;

// The purpose of the Trainer is to play games of self-play to generate training
// data, and to train the model with the data generated during self-play.
pub struct Trainer {
    num_games: usize,
    // The path to the directory to use for checkpoints.
    check_point_dir: Option<PathBuf>,
    // The number of training steps to run before creating a checkpoint.
    check_point_step: usize,
    // The batch size to use during training.
    batch_size: usize,
}
