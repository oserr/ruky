// This is simply a scratch pad to combine and tests stuff, either to verify
// that it works, or simply to figure out how something works.

use burn::backend::candle::{Candle, CandleDevice};
use burn::tensor::Tensor;
use ruky::nn::AlphaZeroNet;

fn main() {
    let device = CandleDevice::cuda(0);
    let t = Tensor::zeros([1, 119, 8, 8], &device);
    let alpha_zero = AlphaZeroNet::<Candle>::new(&device);
    let (pol, val) = alpha_zero.forward(t);
    println!("pol={}", pol);
    println!("val={}", val);
}
