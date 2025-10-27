// This is simply a scratch pad to combine and tests stuff, either to verify
// that it works, or simply to figure out how something works.
#![recursion_limit = "256"]

#[cfg(feature = "cuda")]
use burn::backend::cuda::{Cuda, CudaDevice};

#[cfg(feature = "wgpu")]
use burn::backend::wgpu::{Wgpu, WgpuDevice};

use burn::module::Module;
use burn::record::{FullPrecisionSettings, NamedMpkFileRecorder};
use ruky::nn::AlphaZeroNet;

#[cfg(feature = "cuda")]
type Backend = Cuda;
#[cfg(feature = "wgpu")]
type Backend = Wgpu;

fn main() {
    #[cfg(feature = "cuda")]
    let device = CudaDevice::new(0);
    #[cfg(feature = "wgpu")]
    let device = WgpuDevice::DefaultDevice;

    let alpha_zero = AlphaZeroNet::<Backend>::new(&device);
    let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
    alpha_zero
        .save_file("/home/omar/burn_model", &recorder)
        .expect("Alpha zero model is saved.");

    let other_net = AlphaZeroNet::<Backend>::new(&device);
    other_net
        .load_file("/home/omar/burn_model", &recorder, &device)
        .expect("Alpha zero model is saved.");
}
