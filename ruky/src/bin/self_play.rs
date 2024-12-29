use burn::backend::candle::{Candle, CandleDevice};
use ruky::game::GameBuilder;
use ruky::Ruky;

fn main() {
    let ruky = Ruky::new();
    let device = CandleDevice::cuda(0);
    let game = GameBuilder::<Candle>::new()
        .device(device)
        .board(ruky.new_board())
        .sims(100)
        .max_moves(100)
        .build()
        .expect("Expecting a new game.");
    let result = game.play().expect("Expecting game result.");
    println!(
        "Game finished in {} moves with winner {:?}.",
        result.moves.len(),
        result.winner
    );
}
