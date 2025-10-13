use burn::{
    backend::cuda::{Cuda, CudaDevice},
    module::Module,
    prelude::{Backend, Device},
    record::{NoStdTrainingRecorder, Recorder},
};
use clap::{Parser, ValueEnum};
use ruky::{
    game::MatchGamesBuilder,
    nn::{AlphaZeroNet, AlphaZeroNetRecord},
    Ruky,
};
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
    let (net1, net2) = build_nets(&args, &device);
    let mut match_games = MatchGamesBuilder::<Cuda>::new()
        .device(device)
        .board(ruky.new_board())
        .net_player1(net1)
        .net_player2(net2)
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
            InitStrategy::New => &"new",
            InitStrategy::ModelPath => &"model-path",
        }
    }

    fn is_new(&self) -> bool {
        matches!(*self, InitStrategy::New)
    }
}

impl Display for InitStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn build_nets<B: Backend>(
    args: &Args,
    device: &Device<B>,
) -> (Arc<AlphaZeroNet<B>>, Arc<AlphaZeroNet<B>>) {
    if args.init_player1.is_new() && args.init_player2.is_new() {
        let net_first = Arc::new(AlphaZeroNet::new(device));
        let net_second = net_first.clone();
        return (net_first, net_second);
    }

    let net_first = init_net(args.init_player1, &args.model_path1, device);
    let net_second = init_net(args.init_player2, &args.model_path2, device);

    (net_first, net_second)
}

fn init_net<B: Backend>(
    init_strategy: InitStrategy,
    model_path: &Option<PathBuf>,
    device: &Device<B>,
) -> Arc<AlphaZeroNet<B>> {
    match init_strategy {
        InitStrategy::New => Arc::new(AlphaZeroNet::new(device)),
        InitStrategy::ModelPath => match model_path {
            None => {
                eprintln!("Expecting the model path to be set, but is not.");
                std::process::exit(1);
            }
            Some(model_path) => {
                let record: AlphaZeroNetRecord<B> = NoStdTrainingRecorder::new()
                    .load(model_path.clone(), device)
                    .expect("Expecting to read model");
                let model = AlphaZeroNet::<B>::new(device).load_record(record);
                Arc::new(model)
            }
        },
    }
}

fn as_mins(dur: &Duration) -> f32 {
    dur.as_secs_f32() / 60.0
}
