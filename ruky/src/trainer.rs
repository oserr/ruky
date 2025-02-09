// This module contains components for a trainer.

use crate::game::TrainingGame;
use crate::search::{SpSearch, TreeSize};

pub struct Trainer<S: SpSearch + TreeSize> {
    game: TrainingGame<S>,
}
