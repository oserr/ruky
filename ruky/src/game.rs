// This module contains components to play games of chess.

use crate::board::{Board, GameState};
use crate::err::RukyErr;
use crate::eval::AzEval;
use crate::mcts::{Mcts, SpMcts, SpMctsBuilder};
use crate::mt_mcts::ParMcts;
use crate::nn::AlphaZeroNet;
use crate::piece::Color;
use crate::search::{Search, SearchResult, SpSearch, TreeSize};
use crate::tensor_decoder::AzDecoder;
use crate::tensor_encoder::AzEncoder;
use burn::prelude::{Backend, Device};
use std::{cmp::max, mem::swap, sync::Arc, time::Duration};

// Parallel training game builder.
#[derive(Clone, Debug)]
pub struct ParTrGameBuilder<B: Backend> {
    board: Option<Board>,
    device: Option<Device<B>>,
    sims: usize,
    max_moves: usize,
    use_noise: bool,
    sample_action: bool,
    batch_size: Option<usize>,
    num_workers: Option<usize>,
}

impl<B: Backend> ParTrGameBuilder<B> {
    pub fn new() -> Self {
        Self {
            board: None,
            device: None,
            sims: 800,
            max_moves: 300,
            use_noise: true,
            sample_action: true,
            batch_size: None,
            num_workers: None,
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

    pub fn sims(mut self, sims: usize) -> Self {
        self.sims = sims;
        self
    }

    pub fn max_moves(mut self, max_moves: usize) -> Self {
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

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size.replace(batch_size);
        self
    }

    pub fn num_workers(mut self, num_workers: usize) -> Self {
        self.num_workers.replace(num_workers);
        self
    }

    pub fn build(self) -> Result<TrainingGame<ParMcts<AzEval<B>>, B>, RukyErr> {
        match (self.board, self.device) {
            (Some(board), Some(device)) => {
                let encoder = AzEncoder::new(device.clone());
                let decoder = AzDecoder::new();
                let net = Arc::new(AlphaZeroNet::new(&device));
                let eval = Arc::new(AzEval::create(encoder, decoder, net.clone()));
                let mcts = ParMcts::create(
                    eval,
                    board.clone(),
                    self.sims,
                    self.use_noise,
                    self.sample_action,
                    self.batch_size.unwrap_or(16),
                    self.num_workers.unwrap_or(16),
                );
                Ok(TrainingGame::create(board, mcts, net, self.max_moves))
            }
            (_, _) => Err(RukyErr::PreconditionErr),
        }
    }
}

pub struct TrGameBuilder<B: Backend> {
    board: Option<Board>,
    device: Option<Device<B>>,
    sims: usize,
    max_moves: usize,
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

    pub fn sims(mut self, sims: usize) -> Self {
        self.sims = sims;
        self
    }

    pub fn max_moves(mut self, max_moves: usize) -> Self {
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

    pub fn build(self) -> Result<TrainingGame<SpMcts<AzEval<B>>, B>, RukyErr> {
        match (self.board, self.device) {
            (Some(board), Some(device)) => {
                let encoder = AzEncoder::new(device.clone());
                let decoder = AzDecoder::new();
                let net = Arc::new(AlphaZeroNet::new(&device));
                let eval = Arc::new(AzEval::create(encoder, decoder, net.clone()));
                let mcts = SpMctsBuilder::new()
                    .eval(eval)
                    .board(board.clone())
                    .sims(self.sims)
                    .use_noise(self.use_noise)
                    .sample_action(self.sample_action)
                    .build()?;
                Ok(TrainingGame::create(board, mcts, net, self.max_moves))
            }
            (_, _) => Err(RukyErr::PreconditionErr),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrainingGame<S: SpSearch + TreeSize, B: Backend> {
    board: Board,
    // Search is used for white and black pieces.
    wb_search: S,
    pub(crate) net: Arc<AlphaZeroNet<B>>,
    max_moves: usize,
}

impl<S: SpSearch + TreeSize, B: Backend> TrainingGame<S, B> {
    pub fn create(board: Board, wb_search: S, net: Arc<AlphaZeroNet<B>>, max_moves: usize) -> Self {
        Self {
            board,
            wb_search,
            net,
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

    pub fn reset(&mut self) {
        self.wb_search.reset();
    }
}

// TODO: make this generic over Search once we have different types of Search.
pub struct GameBuilder<B: Backend> {
    board: Option<Board>,
    device: Option<Device<B>>,
    sims: usize,
    max_moves: usize,
    use_noise: bool,
}

impl<B: Backend> GameBuilder<B> {
    pub fn new() -> Self {
        Self {
            board: None,
            device: None,
            sims: 800,
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

    pub fn sims(mut self, sims: usize) -> Self {
        self.sims = sims;
        self
    }

    pub fn max_moves(mut self, max_moves: usize) -> Self {
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
                        Mcts::create_with_noise(evaluator.clone(), self.sims),
                        Mcts::create_with_noise(evaluator, self.sims),
                    )
                } else {
                    (
                        Mcts::create(evaluator.clone(), self.sims),
                        Mcts::create(evaluator, self.sims),
                    )
                };
                white_mcts.enable_sample_action(true);
                black_mcts.enable_sample_action(true);
                Ok(Game::create(
                    board,
                    Box::new(white_mcts),
                    Box::new(black_mcts),
                    self.max_moves,
                ))
            }
            (_, _) => Err(RukyErr::PreconditionErr),
        }
    }
}

// A struct to represent a game between two players.
#[derive(Debug)]
pub struct Game<S: Search> {
    board: Board,
    white_search: Box<S>,
    black_search: Box<S>,
    max_moves: usize,
}

impl<S: Search> Game<S> {
    pub fn create(
        board: Board,
        white_search: Box<S>,
        black_search: Box<S>,
        max_moves: usize,
    ) -> Self {
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

    // Flips the players.
    pub fn flip(&mut self) {
        swap(&mut self.white_search, &mut self.black_search);
    }
}

#[derive(Clone, Debug)]
pub struct GameResult {
    pub board: Board,
    pub moves: Vec<SearchResult>,
    pub winner: GameWinner,
    pub total_tree_nodes: usize,
}

impl GameResult {
    pub fn stats(&self) -> GameStats {
        let mut game_stats = GameStats::new();
        game_stats.moves = self.moves.len();
        for result in &self.moves {
            game_stats.nodes_expanded += result.nodes_expanded as u128;
            game_stats.nodes_visited += result.nodes_visited as u128;
            game_stats.max_depth = max(game_stats.max_depth, result.depth.into());
            game_stats.total_evals += result.total_evals as u64;
            game_stats.eval_time += result.total_eval_time;
            game_stats.search_time += result.total_search_time;
            game_stats.move_gen_time += result.avg_move_gen_time;
            game_stats.max_move_gen_time =
                max(game_stats.max_move_gen_time, result.max_move_gen_time);
        }
        game_stats
    }
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

// Represents a game's stats.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GameStats {
    // Total number of moves in the game.
    pub moves: usize,
    // Number of nodes expanded in the game.
    pub nodes_expanded: u128,
    // Number of nodes visited in the game.
    pub nodes_visited: u128,
    // Max depth throught the game.
    pub max_depth: u64,
    // Number of evals.
    pub total_evals: u64,
    // Total time spent in eval mode - the component of the engine that computes
    // the score for a given position.
    pub eval_time: Duration,
    // Total time spent in search mode - includes eval mode + search time.
    pub search_time: Duration,
    // The total time spent generate moves across all moves in the game.
    pub move_gen_time: Duration,
    // The maximum time taken to generate moves across all moves in the game.
    pub max_move_gen_time: Duration,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            moves: 0,
            nodes_expanded: 0,
            nodes_visited: 0,
            max_depth: 0,
            total_evals: 0,
            eval_time: Duration::ZERO,
            search_time: Duration::ZERO,
            move_gen_time: Duration::ZERO,
            max_move_gen_time: Duration::ZERO,
        }
    }

    pub fn avg_nodes_expanded(&self) -> f32 {
        self.nodes_expanded as f32 / self.moves as f32
    }

    pub fn avg_nodes_visited(&self) -> f32 {
        self.nodes_visited as f32 / self.moves as f32
    }

    pub fn evals_per_move(&self) -> f32 {
        self.total_evals as f32 / self.moves as f32
    }

    pub fn avg_eval_time_micros(&self) -> u128 {
        self.eval_time.as_micros() / self.moves as u128
    }

    pub fn avg_search_time_micros(&self) -> u128 {
        self.search_time.as_micros() / self.moves as u128
    }

    pub fn avg_move_gen_time_micros(&self) -> u128 {
        self.move_gen_time.as_micros() / self.moves as u128
    }
}

impl Default for GameStats {
    fn default() -> Self {
        Self::new()
    }
}

// A struct for playing a series of games between two players.
#[derive(Debug)]
pub struct MatchGames<S: Search> {
    game: Game<S>,
    // The name of the first player. This corresponds to the player playing as
    // white the first game, black the second game, and so on.
    name_player1: String,
    // The name of the second player, who plays black the first game, white the
    // second, and so on.
    name_player2: String,
    // The number of games to be played in the match.
    num_games: usize,
}
