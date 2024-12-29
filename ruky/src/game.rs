// This module contains components to play games of chess.

use crate::board::{Board, GameState};
use crate::err::RukyErr;
use crate::piece::Color;
use crate::search::{Search, SearchResult};
use std::rc::Rc;

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
