// This module contains components to play games of chess.

use crate::board::{Board, GameState};
use crate::err::RukyErr;
use crate::nn::AlphaZeroNet;
use crate::piece::Color;
use crate::search::{Search, SearchResult};
use burn::prelude::{Backend, Device};
use std::rc::Rc;
use std::sync::Arc;

pub struct GameBuilder<B: Backend> {
    white_net: Option<Arc<AlphaZeroNet<B>>>,
    black_net: Option<Arc<AlphaZeroNet<B>>>,
    white_sims: u32,
    black_sims: u32,
    max_moves: u32,
}

impl<B: Backend> GameBuilder<B> {
    pub fn new() -> Self {
        Self {
            white_net: None,
            black_net: None,
            white_sims: 800,
            black_sims: 800,
            max_moves: 300,
        }
    }

    pub fn device(mut self, device: Device<B>) -> Self {
        self.white_net.replace(Arc::new(AlphaZeroNet::new(&device)));
        self.black_net
            .replace(self.white_net.as_ref().unwrap().clone());
        self
    }
}

// A struct to represent a game between two players.
pub struct Game<S: Search> {
    board: Board,
    white_search: Rc<S>,
    black_search: Rc<S>,
    max_moves: u32,
}

impl<S: Search> Game<S> {
    pub fn create(board: Board, white_search: Rc<S>, black_search: Rc<S>, max_moves: u32) -> Self {
        Self {
            board,
            white_search,
            black_search,
            max_moves,
        }
    }

    pub fn play(&self) -> Result<GameResult, RukyErr> {
        let mut moves = Vec::<SearchResult>::new();
        let mut next_board = &self.board;
        for _ in 0..self.max_moves {
            let result = match next_board.is_white_next() {
                true => self.white_search.search_board(next_board)?,
                false => self.black_search.search_board(next_board)?,
            };
            moves.push(result);
            next_board = moves.last().unwrap().best_board();
            if next_board.is_terminal() {
                break;
            }
        }
        let winner = GameWinner::from(moves.last().unwrap().best_board().game_state());
        Ok(GameResult {
            board: self.board.clone(),
            moves,
            winner,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GameResult {
    board: Board,
    moves: Vec<SearchResult>,
    winner: GameWinner,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameWinner {
    Black,
    White,
    Draw,
}

impl From<GameState> for GameWinner {
    fn from(game_state: GameState) -> Self {
        match game_state {
            GameState::Mate(color) => match color {
                Color::White => GameWinner::Black,
                Color::Black => GameWinner::White,
            },
            _ => GameWinner::Draw,
        }
    }
}
