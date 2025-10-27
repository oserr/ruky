// To use Candle backend with Cuda:
// use burn::backend::candle::{Candle, CandleDevice};
#[cfg(feature = "cuda")]
use burn::backend::cuda::{Cuda, CudaDevice};

#[cfg(feature = "wgpu")]
use burn::backend::wgpu::{Wgpu, WgpuDevice};

use ruky::game::TrainingGameBuilder;
use ruky::Ruky;
use std::time::{Duration, Instant};

#[cfg(feature = "cuda")]
type Backend = Cuda;
#[cfg(feature = "wgpu")]
type Backend = Wgpu;

// TODO: flesh this out into something more usable and configurable.
fn main() {
    let ruky = Ruky::new();

    // To use Candle backend with Cuda support:
    //   let device =  CandleDevice::cuda(0);
    //   let game = TrainingGameBuilder::<Candle>::new()...;
    #[cfg(feature = "cuda")]
    let device = CudaDevice::new(0);
    #[cfg(feature = "wgpu")]
    let device = WgpuDevice::DefaultDevice;

    let mut game = TrainingGameBuilder::<Backend>::new()
        .device(device)
        .board(ruky.new_board())
        .sims(800)
        .max_moves(300)
        .use_noise(true)
        .sample_action(true)
        .batch_size(30)
        .num_workers(30)
        .build()
        .expect("Expecting a new game.");
    println!("Starting a game of self play...");
    let verbose = false;
    let now = Instant::now();
    let result = game.play().expect("Expecting game result.");
    let dur = now.elapsed();
    let mut color = result.board.color();
    for (i, search_result) in result.moves.iter().enumerate() {
        let eval_time_per_expansion = search_result.eval_time_per_expansion();
        let search_time_per_expansion = search_result.search_time_per_expansion();
        let avg_eval_time = search_result.avg_eval_time();
        println!(
            "i={} color={:?} prior={} value={} move={:?}
            nodes_expanded={} nodes_visited={} depth={}
            eval_time_per_expansion: micros={}
            search_time_per_expansion: micros={}
            total_eval_time: mins={} secs={} millis={}
            avg_eval_time: micros={} ns={}
            total_search_time: mins={} secs={} millis={}
            avg_move_gen_time: micros={} ns={}
            max_move_gen_time: micros={} ns={}",
            i,
            color,
            search_result.best.prior,
            search_result.value,
            search_result.best_move(),
            search_result.nodes_expanded,
            search_result.nodes_visited,
            search_result.depth,
            eval_time_per_expansion.as_micros(),
            search_time_per_expansion.as_micros(),
            as_mins(&search_result.total_eval_time),
            search_result.total_eval_time.as_secs_f32(),
            search_result.total_eval_time.as_millis(),
            avg_eval_time.as_micros(),
            avg_eval_time.as_nanos(),
            as_mins(&search_result.total_search_time),
            search_result.total_search_time.as_secs_f32(),
            search_result.total_search_time.as_millis(),
            search_result.avg_move_gen_time.as_micros(),
            search_result.avg_move_gen_time.as_nanos(),
            search_result.max_move_gen_time.as_micros(),
            search_result.max_move_gen_time.as_nanos(),
        );
        if verbose {
            for mp in &search_result.moves {
                println!("\tprior={} visits={} move={:?}", mp.prior, mp.visits, mp.pm);
            }
        }
        color = color.flip();
    }
    println!(
        "Game finished in {} moves with winner {:?}.",
        result.moves.len(),
        result.winner
    );
    println!(
        "Time spent in play: mins=[{}] secs=[{}] millis=[{}] total_tree_nodes=[{}]",
        as_mins(&dur),
        dur.as_secs_f32(),
        dur.as_millis(),
        result.total_tree_nodes,
    );
    let game_stats = result.stats();
    println!(
        "======== Game Stats ========
        moves={}
        total_nodes_expanded={}
        nodes_expanded_per_move={}
        nodes_visited={}
        nodes_visited_per_move={}
        max_depth={}
        total_evals={}
        evals_per_move={}
        avg_eval_time: micros={}
        avg_search_time: micros={}
        avg_move_gen_time: micros={}
        max_move_gen_time: micros={}",
        game_stats.moves,
        game_stats.nodes_expanded,
        game_stats.avg_nodes_expanded(),
        game_stats.nodes_visited,
        game_stats.avg_nodes_visited(),
        game_stats.max_depth,
        game_stats.total_evals,
        game_stats.evals_per_move(),
        game_stats.avg_eval_time_micros(),
        game_stats.avg_search_time_micros(),
        game_stats.avg_move_gen_time_micros(),
        game_stats.max_move_gen_time.as_micros(),
    );
}

fn as_mins(dur: &Duration) -> f32 {
    dur.as_secs_f32() / 60.0
}
