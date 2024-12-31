use burn::backend::candle::{Candle, CandleDevice};
use ruky::game::GameBuilder;
use ruky::Ruky;
use std::time::{Duration, Instant};

// TODO: flesh this out into something more usable and configurable.
fn main() {
    let ruky = Ruky::new();
    let device = CandleDevice::cuda(0);
    let game = GameBuilder::<Candle>::new()
        .device(device)
        .board(ruky.new_board())
        .sims(800)
        .max_moves(300)
        .use_noise(true)
        .build()
        .expect("Expecting a new game.");
    println!("Starting a game of self play...");
    let now = Instant::now();
    let result = game.play().expect("Expecting game result.");
    let dur = now.elapsed();
    let mut color = result.board.color();
    for (i, search_result) in result.moves.iter().enumerate() {
        println!(
            "i={} color={:?} prior={} value={} move={:?}",
            i,
            color,
            search_result.best.prior,
            search_result.value,
            search_result.best_move()
        );
        for mp in &search_result.moves {
            println!("\tprior={} visits={} move={:?}", mp.prior, mp.visits, mp.pm);
        }
        color = color.flip();
    }
    println!(
        "Game finished in {} moves with winner {:?}.",
        result.moves.len(),
        result.winner
    );
    println!(
        "Time spent in play: mins=[{}] secs=[{}] millis=[{}]",
        as_mins(&dur),
        dur.as_secs_f32(),
        dur.as_millis(),
    );
}

fn as_mins(dur: &Duration) -> f32 {
    dur.as_secs_f32() / 60.0
}
