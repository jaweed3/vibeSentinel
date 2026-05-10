#![no_std]
#![deny(unsafe_code)]

pub mod stats;
pub mod fft;
pub mod extractor;

pub use fft::WINDOW_SIZE;
pub use fft::FFT_BINS;
pub use extractor::FEATURE_DIM;

pub const INPUT_DIM: usize = 26;
pub const HIDDEN1_DIM: usize = 13;
pub const LATENT_DIM: usize = 6;
pub const HIDDEN2_DIM: usize = 13;
pub const OUTPUT_DIM: usize = 26;
