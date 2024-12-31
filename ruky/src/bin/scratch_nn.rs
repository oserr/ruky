// This is simply a scratch pad to combine and tests stuff, either to verify
// that it works, or simply to figure out how something works.

use burn::backend::candle::{Candle, CandleDevice};
use burn::tensor::Tensor;
use ruky::nn::AlphaZeroNet;
use ruky::tensor_decoder::{AzDecoder, TensorDecoder};
use ruky::tensor_encoder::{AzEncoder, TensorEncoder};
use ruky::Ruky;

fn main() {
    let device = CandleDevice::cuda(0);
    let t = Tensor::zeros([1, 119, 8, 8], &device);
    let alpha_zero = AlphaZeroNet::<Candle>::new(&device);
    let (pol, val) = alpha_zero.forward(t);
    println!("pol={}", pol);
    println!("val={}", val);

    let ruky = Ruky::new();
    let board = ruky.new_board();
    let enc = AzEncoder::new(device.clone());
    let t = enc.encode_board(&board);
    println!("t={}", t);
    let (pol, val) = alpha_zero.forward(t);
    println!("pol={}", pol);
    println!("val={}", val);

    let dec = AzDecoder::<Candle>::new();
    let dec_boards = dec
        .decode_boards(&board, pol, val)
        .expect("Expecting decdoed boards.");
    println!("Decoded tensor:-----");
    println!("value={}", dec_boards.value);
    for (board, prior) in dec_boards.board_probs {
        let last_move = board.last_move().expect("Expecting a last move.");
        println!("\tprior={} move={:?}", prior, last_move);
    }
}
