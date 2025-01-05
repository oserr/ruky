use burn::backend::candle::{Candle, CandleDevice};
use ruky::game::TrGameBuilder;
use ruky::Ruky;
use std::time::{Duration, Instant};

// TODO: flesh this out into something more usable and configurable.
fn main() {
    let ruky = Ruky::new();
    let device = CandleDevice::cuda(0);
    let mut game = TrGameBuilder::<Candle>::new()
        .device(device)
        .board(ruky.new_board())
        .sims(800)
        .max_moves(300)
        .use_noise(true)
        .sample_action(true)
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
        println!(
            "i={} color={:?} prior={} value={} move={:?}
            \tnodes_expanded={} nodes_visited={} depth={}
            \teval_time_per_expansion_millis={} search_time_per_expansion_millis={}
            \ttotal_eval_time: mins={} secs={}
            \ttotal_search_time: mins={} secs={}",
            i,
            color,
            search_result.best.prior,
            search_result.value,
            search_result.best_move(),
            search_result.nodes_expanded,
            search_result.nodes_visited,
            search_result.depth,
            eval_time_per_expansion.as_millis(),
            search_time_per_expansion.as_millis(),
            as_mins(&search_result.total_eval_time),
            search_result.total_eval_time.as_secs_f32(),
            as_mins(&search_result.total_search_time),
            search_result.total_search_time.as_secs_f32(),
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
}

fn as_mins(dur: &Duration) -> f32 {
    dur.as_secs_f32() / 60.0
}
