// This module contains components to play games of chess.

use crate::board::{Board, GameState};
use crate::err::RukyErr;
use crate::eval::AzEval;
use crate::mcts::Mcts;
use crate::nn::AlphaZeroNet;
use crate::piece::Color;
use crate::search::{Search, SearchResult};
use crate::tensor_decoder::AzDecoder;
use crate::tensor_encoder::AzEncoder;
use burn::prelude::{Backend, Device};
use std::sync::Arc;

// TODO: make this generic over Search once we have different types of Search.
pub struct GameBuilder<B: Backend> {
    board: Option<Board>,
    device: Option<Device<B>>,
    white_sims: u32,
    black_sims: u32,
    max_moves: u32,
    use_noise: bool,
}

impl<B: Backend> GameBuilder<B> {
    pub fn new() -> Self {
        Self {
            board: None,
            device: None,
            white_sims: 800,
            black_sims: 800,
            max_moves: 300,
            use_noise: false,
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
        self.white_sims = sims;
        self.black_sims = sims;
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

    pub fn build(self) -> Result<Game<Mcts<AzEval<B>>>, RukyErr> {
        match (self.board, self.device) {
            (Some(board), Some(device)) => {
                let encoder = AzEncoder::new(device.clone());
                let decoder = AzDecoder::new();
                let net = Arc::new(AlphaZeroNet::new(&device));
                let evaluator = Arc::new(AzEval::create(encoder, decoder, net));
                let (white_mcts, black_mcts) = if self.use_noise {
                    (
                        Mcts::create_with_noise(evaluator.clone(), self.white_sims),
                        Mcts::create_with_noise(evaluator, self.black_sims),
                    )
                } else {
                    (
                        Mcts::create(evaluator.clone(), self.white_sims),
                        Mcts::create(evaluator, self.black_sims),
                    )
                };
                white_mcts.enable_sample_action(true);
                black_mcts.enable_sample_action(true);
                Ok(Game::create(board, white_mcts, black_mcts, self.max_moves))
            }
            (_, _) => Err(RukyErr::PreconditionErr),
        }
    }
}

// A struct to represent a game between two players.
pub struct Game<S: Search> {
    board: Board,
    white_search: S,
    black_search: S,
    max_moves: u32,
}

impl<S: Search> Game<S> {
    pub fn create(board: Board, white_search: S, black_search: S, max_moves: u32) -> Self {
        Self {
            board,
            white_search,
            black_search,
            max_moves,
        }
    }

    pub fn play(&mut self) -> Result<GameResult, RukyErr> {
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
        let game_state = moves.last().unwrap().best_board().game_state();
        let winner = GameWinner::from(game_state);
        Ok(GameResult {
            board: self.board.clone(),
            moves,
            winner,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GameResult {
    pub board: Board,
    pub moves: Vec<SearchResult>,
    pub winner: GameWinner,
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
