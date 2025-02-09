// This is simply a scratch pad to combine and tests stuff, either to verify
// that it works, or simply to figure out how something works.

use burn::backend::candle::{Candle, CandleDevice};
use burn::module::Module;
use burn::record::{FullPrecisionSettings, NamedMpkFileRecorder};
use ruky::nn::AlphaZeroNet;

fn main() {
    let device = CandleDevice::cuda(0);
    let alpha_zero = AlphaZeroNet::<Candle>::new(&device);
    let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
    alpha_zero
        .save_file("/home/omar/burn_model", &recorder)
        .expect("Alpha zero model is saved.");

    let other_net = AlphaZeroNet::<Candle>::new(&device);
    other_net
        .load_file("/home/omar/burn_model", &recorder, &device)
        .expect("Alpha zero model is saved.");
}
