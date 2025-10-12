use burn::backend::cuda::{Cuda, CudaDevice};
use clap::{Parser, ValueEnum};
use ruky::{game::MatchGamesBuilder, nn::AlphaZeroNet, Ruky};
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

fn main() {
    let args = Args::parse();
    let ruky = Ruky::new();
    let device = CudaDevice::new(0);
    let net = Arc::new(AlphaZeroNet::new(&device));
    let mut match_games = MatchGamesBuilder::<Cuda>::new()
        .device(device)
        .board(ruky.new_board())
        .net_player1(net.clone())
        .net_player2(net)
        .num_games(args.games)
        .batch_size(args.batch_size)
        .num_workers(args.workers.unwrap_or(32))
        .build()
        .expect("Expecting a match of games");
    println!("Running a match of {} games...", args.games);
    let now = Instant::now();
    match match_games.play() {
        Err(_) => eprintln!("Error playing match"),
        Ok(match_result) => {
            let mins = as_mins(&now.elapsed());
            println!("Finished match in {} minutes.", mins);
            println!("Match results are: {:?}", match_result);
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The number of games to self-play for one session. These games are
    /// subsequently used in a round of training.
    #[arg(short, long, default_value_t = 10)]
    games: usize,

    /// The inference batch size. This is number of board positions that we try
    /// to evaluate in one batch.
    #[arg(short, long, default_value_t = 16)]
    batch_size: usize,

    /// The number of workers used by each MCTS.
    #[arg(short, long)]
    workers: Option<usize>,

    /// The initialization strategy for player 1. If ModelPath is set, then
    /// model_path1 must be set.
    #[arg(long, default_value_t = InitStrategy::New)]
    init_player1: InitStrategy,

    /// The path to the model for player 1 when the init strategy is ModelPath.
    #[arg(long)]
    model_path1: Option<PathBuf>,

    /// The initialization strategy for player 2. If ModelPath is set, then
    /// model_path2 must be set.
    #[arg(long, default_value_t = InitStrategy::New)]
    init_player2: InitStrategy,

    /// The path to the model for player 2 when the init strategy is ModelPath.
    #[arg(long)]
    model_path2: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum InitStrategy {
    New,
    ModelPath,
}

impl InitStrategy {
    fn as_str(&self) -> &str {
        match *self {
            InitStrategy::New => &"New",
            InitStrategy::ModelPath => &"ModelPath",
        }
    }
}

impl Display for InitStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn as_mins(dur: &Duration) -> f32 {
    dur.as_secs_f32() / 60.0
}
