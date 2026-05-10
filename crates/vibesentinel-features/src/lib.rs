#![no_std]

pub mod window;
pub mod stats;
pub mod fft;
pub mod extractor;

// Centralized shared constants — single source of truth for all crates.
// Model and trainer import from here to avoid hardcoded magic numbers.
pub use fft::WINDOW_SIZE;
pub use fft::FFT_BINS;
pub use extractor::FEATURE_DIM;

/// Autoencoder architecture dimensions.
/// Must match vibesentinel-trainer model definition exactly.
pub const INPUT_DIM: usize = 20;
pub const HIDDEN1_DIM: usize = 10;
pub const LATENT_DIM: usize = 4;
pub const HIDDEN2_DIM: usize = 10;
pub const OUTPUT_DIM: usize = 20;
