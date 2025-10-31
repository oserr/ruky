#[cfg(feature = "cuda")]
use burn::backend::cuda::{Cuda, CudaDevice};

#[cfg(feature = "wgpu")]
use burn::backend::wgpu::{Wgpu, WgpuDevice};

use clap::Parser;
use log::LevelFilter;
use ruky::trainer::TrainerBuilder;
use ruky::Ruky;
use std::time::{Duration, Instant};

#[cfg(feature = "cuda")]
type Backend = Cuda;
#[cfg(feature = "wgpu")]
type Backend = Wgpu;

fn main() {
    let args = Args::parse();
    let ruky = Ruky::new();

    #[cfg(feature = "cuda")]
    let device = CudaDevice::new(0);
    #[cfg(feature = "wgpu")]
    let device = WgpuDevice::DefaultDevice;

    let trainer = TrainerBuilder::<Backend>::new()
        .device(device)
        .board(ruky.new_board())
        .num_games(args.training_games)
        .match_games(args.match_games)
        .num_sessions(args.sessions)
        .check_point_dir(&args.out_dir)
        .training_batch_size(args.training_batch_size)
        .training_percent(args.training_percent)
        .num_epochs(args.epochs)
        .build()
        .expect("Expecting a trainer.");

    if let Err(_) = simple_logging::log_to_file("training.log", LevelFilter::max()) {
        eprintln!("Unable to initialize logging.");
    }

    println!("Running the training pipeline...");
    let now = Instant::now();
    if let Err(_) = trainer.run_training() {
        eprintln!("Error running running training pipeline.");
    } else {
        let mins = as_mins(&now.elapsed());
        println!("Finished training in {} minutes.", mins);
    }
}

fn as_mins(dur: &Duration) -> f32 {
    dur.as_secs_f32() / 60.0
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The number of games to self-play for one session. These games are
    /// subsequently used in a round of training.
    #[arg(long, default_value_t = 50)]
    training_games: usize,

    /// The number of games to between newly trained and an old network to
    /// determine which network to use for self-play.
    #[arg(long, default_value_t = 20)]
    match_games: usize,

    /// The number of sessions to train for. 1 session includes one round of
    /// self-play, a round of training, and a tournament match between the old
    /// and newly trained model.
    #[arg(short, long)]
    sessions: usize,

    /// The output directory where training artifacts are saved, which may
    /// include games of self-play, training check-points, and models.
    #[arg(short, long)]
    out_dir: String,

    /// The training batch size.
    #[arg(short, long, default_value_t = 32)]
    training_batch_size: usize,

    /// The percent of games that are used for training data. The difference are
    /// used as a validation set.
    #[arg(short, long, default_value_t = 0.9)]
    training_percent: f32,

    /// The number of epochs to use for training.
    #[arg(short, long, default_value_t = 75)]
    epochs: usize,
}
