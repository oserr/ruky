// This module contains components for a trainer.

use crate::dataset::{GamesBatcher, GamesDataset};
use crate::err::RukyErr;
use crate::game::{GameResult, MatchGamesBuilder, TrainingGameBuilder};
use crate::nn::{AlphaZeroNet, AlphaZeroNetRecord};
use crate::Board;
use burn::{
    backend::Autodiff,
    data::dataloader::DataLoaderBuilder,
    module::Module,
    optim::SgdConfig,
    prelude::{Backend, Device},
    record::{CompactRecorder, NoStdTrainingRecorder, Recorder},
    train::{LearnerBuilder, LearningStrategy},
};
use log;
use rand::rng;
use rand::seq::SliceRandom;
use std::{
    fs::{create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
    sync::Arc,
};

// The purpose of the Trainer is to play games of self-play to generate training
// data, and to train the model with the data generated during self-play.
pub struct Trainer<B: Backend> {
    // The initial starting position.
    board: Board,
    // The device to use for neural network computations.
    device: Device<B>,
    // The total number of simulations per move.
    sims: usize,
    // The maximum number of moves to play before declaring draw.
    max_moves: usize,
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
    // The number of sessions to train. 1 session includes 1 round of self play
    // for num_games, plus a round of training with the games generated from
    // self play.
    num_sessions: usize,
    // The path to the directory to use for checkpoints.
    check_point_dir: PathBuf,
    // The number of training steps to run before creating a checkpoint.
    check_point_step: Option<usize>,
    // The batch size to use during training.
    training_batch_size: usize,
    // The number of games to use as training data. A number between (0,1). The
    // remaining percent of games are used as validation data.
    training_percent: f32,
    // The number of epochs used for training.
    num_epochs: usize,
    // The minimum winning rate required from a newly trained net to begin using
    // it to generate games by playing against itself.
    min_win_rate: f32,
}

impl<B: Backend> Trainer<B> {
    fn play_self(&self) -> Result<(Arc<AlphaZeroNet<B>>, Vec<GameResult>), RukyErr> {
        log::info!("Trainer::play_self()...");
        let mut training_game = TrainingGameBuilder::<B>::new()
            .board(self.board.clone())
            .device(self.device.clone())
            .sims(self.sims)
            .max_moves(self.max_moves)
            .use_noise(self.use_noise)
            .sample_action(self.sample_action)
            .batch_size(self.inference_batch_size)
            .num_workers(self.num_workers)
            .build()?;

        let mut game_results = Vec::new();

        for i in 0..self.num_games {
            let game_result = training_game.play()?;
            log::info!("game={} winner={:?}", i, game_result.winner);
            game_results.push(game_result);
            training_game.reset();
        }

        Ok((training_game.net, game_results))
    }

    // TODO: pass in the current training session ID so we can keep better track
    // of trained models.
    fn train_net(
        &self,
        games: Vec<GameResult>,
        session_id: usize,
    ) -> Result<Arc<AlphaZeroNet<B>>, RukyErr> {
        log::info!("Trainer::train_net()...");
        remove_dir_all(&self.check_point_dir).ok();
        create_dir_all(&self.check_point_dir).ok();

        let (games_training, games_validation) = split_game_results(games, self.training_percent);
        let data_training = GamesDataset::new(games_training);
        let data_validation = GamesDataset::new(games_validation);

        let dataloader_train = DataLoaderBuilder::new(GamesBatcher::<Autodiff<B>>::new())
            .batch_size(self.training_batch_size)
            .shuffle(0) // TODO: make seed configurable.
            .num_workers(self.num_workers)
            .build(data_training);

        let dataloader_test = DataLoaderBuilder::new(GamesBatcher::<B>::new())
            .batch_size(self.training_batch_size)
            .shuffle(0) // TODO: make seed configurable.
            .num_workers(self.num_workers)
            .build(data_validation);

        // TODO: once we have trained models, we'll need to be able to load the
        // current best trained model. This could be an older model trained in a
        // previously, or it could be one from one of the current training
        // sessions.
        let model = AlphaZeroNet::<Autodiff<B>>::new(&self.device);

        // TODO: configure learner to log metrics and to use a learning rate
        // scheduler.
        let learner = LearnerBuilder::new(&self.check_point_dir)
            .with_file_checkpointer(CompactRecorder::new())
            .learning_strategy(LearningStrategy::SingleDevice(self.device.clone()))
            .num_epochs(self.num_epochs)
            .build(model, SgdConfig::new().init(), 1e-3);

        let model_trained = learner.fit(dataloader_train, dataloader_test);

        let mut model_path = self.check_point_dir.clone();
        model_path.push("model");

        model_trained
            .model
            .save_file(
                format!("{}-{}", model_path.display(), session_id),
                &NoStdTrainingRecorder::new(),
            )
            .expect("Failed to save trained model.");

        let record: AlphaZeroNetRecord<B> = NoStdTrainingRecorder::new()
            .load(
                format!("{}-{}", model_path.display(), session_id).into(),
                &self.device,
            )
            .expect("Model just saved - should exist.");

        let model = AlphaZeroNet::<B>::new(&self.device).load_record(record);
        Ok(Arc::new(model))
    }

    // TODO: Play multiple games with the networks. If the new network wins more
    // then a percentage of games, then use the new network to play self-play
    // games to generate training games.
    fn play_match(
        &self,
        new_net: Arc<AlphaZeroNet<B>>,
        old_net: Arc<AlphaZeroNet<B>>,
    ) -> Result<Arc<AlphaZeroNet<B>>, RukyErr> {
        let mut match_games = MatchGamesBuilder::new()
            .board(self.board.clone())
            .name_player1(&"NewNet")
            .name_player2(&"OldNet")
            .net_player1(new_net.clone())
            .net_player2(old_net.clone())
            .num_games(10)
            .sims(self.sims)
            .max_moves(self.max_moves)
            .batch_size(self.inference_batch_size)
            .num_workers(self.num_workers)
            .device(self.device.clone())
            .build()?;

        let match_result = match_games.play()?;
        log::info!("MatchResult={:?}", match_result);

        let net = match match_result.winner() {
            Some(ref winner) => {
                log::info!("winner={:?} with win rate={}", winner, winner.win_rate());
                if match_games.is_player1(&winner.name_player)
                    && winner.win_rate() >= self.min_win_rate
                {
                    log::info!("using new network to for self-play");
                    new_net
                } else {
                    old_net
                }
            }
            _ => {
                log::info!("match ended in a draw");
                old_net
            }
        };

        Ok(net)
    }

    pub fn run_training(&self) -> Result<(), RukyErr> {
        for i in 0..self.num_sessions {
            let (old_net, game_results) = self.play_self()?;
            let new_net = self.train_net(game_results, i)?;
            let _net = self.play_match(new_net, old_net)?;
            // TODO: need to pass the new network to self_play to build the Mcts
            // with it.
        }
        Ok(())
    }
}

// The purpose of the Trainer is to play games of self-play to generate training
// data, and to train the model with the data generated during self-play.
pub struct TrainerBuilder<B: Backend> {
    // The initial starting position.
    board: Option<Board>,
    // The device to use for neural network computations.
    device: Option<Device<B>>,
    // The total number of simulations per move.
    sims: usize,
    // The maximum number of moves to play before declaring draw.
    max_moves: usize,
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
    // The number of sessions to train for.
    num_sessions: usize,
    // The path to the directory to use for checkpoints. If none is provided,
    // ./check_point_dir is used as the default location.
    check_point_dir: Option<PathBuf>,
    // The number of training steps to run before creating a checkpoint.
    check_point_step: Option<usize>,
    // The batch size to use during training.
    training_batch_size: usize,
    // The number of games to use as training data. A number between (0,1). The
    // remaining percent of games are used as validation data.
    training_percent: f32,
    // The number of epochs used for training. If not set, 100 is used.
    num_epochs: Option<usize>,
    // The minimum winning rate required from a newly trained net to begin using
    // it to generate games by playing against itself.
    min_win_rate: f32,
}

impl<B: Backend> TrainerBuilder<B> {
    pub fn new() -> Self {
        let num_threads = num_cpus::get();
        let num_workers = if num_threads > 1 {
            num_threads - 1
        } else {
            num_threads
        };
        Self {
            board: None,
            device: None,
            sims: 800,
            max_moves: 300,
            use_noise: true,
            sample_action: true,
            inference_batch_size: num_threads,
            num_workers: num_workers,
            num_games: None,
            num_sessions: 100,
            check_point_dir: None,
            check_point_step: None,
            training_batch_size: num_threads,
            training_percent: 0.95,
            num_epochs: None,
            min_win_rate: 0.55,
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

    pub fn num_sessions(mut self, num_sessions: usize) -> Self {
        self.num_sessions = num_sessions;
        self
    }

    pub fn check_point_dir<P: AsRef<Path>>(mut self, check_point_dir: &P) -> Self {
        self.check_point_dir
            .replace(check_point_dir.as_ref().to_path_buf());
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

    pub fn training_percent(mut self, percent: f32) -> Self {
        self.training_percent = percent;
        self
    }

    pub fn num_epochs(mut self, num_epochs: usize) -> Self {
        self.num_epochs.replace(num_epochs);
        self
    }

    pub fn min_win_rate(mut self, min_win_rate: f32) -> Self {
        self.min_win_rate = min_win_rate;
        self
    }

    pub fn build(self) -> Result<Trainer<B>, RukyErr> {
        if self.board.is_none() || self.device.is_none() || self.num_games.is_none() {
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
            num_sessions: self.num_sessions,
            check_point_dir: self
                .check_point_dir
                .unwrap_or(PathBuf::from("check_points")),
            check_point_step: self.check_point_step,
            training_batch_size: self.training_batch_size,
            training_percent: self.training_percent,
            num_epochs: self.num_epochs.unwrap_or(100),
            min_win_rate: self.min_win_rate,
        })
    }
}

// Splits a vector of game results into two separate sets for use as training
// and validation sets. The training ratio represents the percent of game resuls
// that should be used for training, and is expected to be greater than 0 and
// less than 1, otherwise the function panics.
//
// The first vector in the tuple represents the training set, and the second the
// validation set.
fn split_game_results(
    mut games: Vec<GameResult>,
    training_ratio: f32,
) -> (Vec<GameResult>, Vec<GameResult>) {
    assert!(training_ratio > 0.0 && training_ratio < 1.0);

    let index = (training_ratio * games.len() as f32) as usize;
    let mut rng = rng();
    games.shuffle(&mut rng);

    // Split the shuffled vector into two parts
    let validation_set = games.drain(index..).collect();
    let training_set = games;

    (training_set, validation_set)
}
