// This module contains components to play games of chess.

use crate::board::{Board, GameState};
use crate::err::RukyErr;
use crate::eval::AzEval;
use crate::mcts::{Mcts, SpMcts, SpMctsBuilder};
use crate::nn::AlphaZeroNet;
use crate::piece::Color;
use crate::search::{Search, SearchResult, SpSearch, TreeSize};
use crate::tensor_decoder::AzDecoder;
use crate::tensor_encoder::AzEncoder;
use burn::prelude::{Backend, Device};
use std::sync::Arc;

// Parallel training game builder.
#[derive(Clone, Debug)]
pub struct ParTrGameBuilder<B: Backend> {
    board: Option<Board>,
    device: Option<Device<B>>,
    sims: u32,
    max_moves: u32,
    use_noise: bool,
    sample_action: bool,
    batch_size: Option<u32>,
    num_workers: Option<u32>,
}

pub struct TrGameBuilder<B: Backend> {
    board: Option<Board>,
    device: Option<Device<B>>,
    sims: u32,
    max_moves: u32,
    use_noise: bool,
    sample_action: bool,
}

impl<B: Backend> TrGameBuilder<B> {
    pub fn new() -> Self {
        Self {
            board: None,
            device: None,
            sims: 800,
            max_moves: 300,
            use_noise: true,
            sample_action: true,
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

    pub fn build(self) -> Result<TrainingGame<SpMcts<AzEval<B>>>, RukyErr> {
        match (self.board, self.device) {
            (Some(board), Some(device)) => {
                let encoder = AzEncoder::new(device.clone());
                let decoder = AzDecoder::new();
                let net = Arc::new(AlphaZeroNet::new(&device));
                let eval = Arc::new(AzEval::create(encoder, decoder, net));
                let mcts = SpMctsBuilder::new()
                    .eval(eval)
                    .board(board.clone())
                    .sims(self.sims)
                    .use_noise(self.use_noise)
                    .sample_action(self.sample_action)
                    .build()?;
                Ok(TrainingGame::create(board, mcts, self.max_moves))
            }
            (_, _) => Err(RukyErr::PreconditionErr),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrainingGame<S: SpSearch + TreeSize> {
    board: Board,
    // Search is used for white and black pieces.
    wb_search: S,
    max_moves: u32,
}

impl<S: SpSearch + TreeSize> TrainingGame<S> {
    pub fn create(board: Board, wb_search: S, max_moves: u32) -> Self {
        Self {
            board,
            wb_search,
            max_moves,
        }
    }

    pub fn play(&mut self) -> Result<GameResult, RukyErr> {
        let mut moves = Vec::<SearchResult>::new();
        for _ in 0..self.max_moves {
            let result = self.wb_search.search()?;
            moves.push(result);
            let board = moves.last().unwrap().best_board();
            if board.is_terminal() {
                break;
            }
        }
        let game_state = moves.last().unwrap().best_board().game_state();
        let winner = GameWinner::from(game_state);
        Ok(GameResult {
            board: self.board.clone(),
            moves,
            winner,
            total_tree_nodes: self.wb_search.total_tree_nodes(),
        })
    }
}

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
                let (mut white_mcts, mut black_mcts) = if self.use_noise {
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
            total_tree_nodes: 0,
        })
    }
}

#[derive(Clone, Debug)]
pub struct GameResult {
    pub board: Board,
    pub moves: Vec<SearchResult>,
    pub winner: GameWinner,
    pub total_tree_nodes: usize,
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
