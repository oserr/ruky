// This module contains components to play games of chess.

use crate::search::Search;
use std::rc::Rc;

// A struct to represent a game between two players.
pub struct Game<S: Search> {
    white_player: Rc<S>,
    black_player: Rc<S>,
    max_moves: u32,
}

impl<S: Search> Game<S> {
    pub fn create(white_player: Rc<S>, black_player: Rc<S>, max_moves: u32) -> Self {
        Self {
            white_player,
            black_player,
            max_moves,
        }
    }
}
