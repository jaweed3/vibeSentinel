use crate::{stats::*, fft::*};

pub const FEATURE_DIM: usize = 26;

pub struct AccelWindow {
    pub x: [f32; WINDOW_SIZE],
    pub y: [f32; WINDOW_SIZE],
    pub z: [f32; WINDOW_SIZE],
}

/// Extract 26-dimensional feature vector from one window.
/// Per-axis: RMS, Peak, Kurtosis, Crest Factor + 4 FFT bins = 8 per axis.
/// Cross-axis: axial/radial ratio, total RMS.
pub fn extract_features(window: &AccelWindow) -> [f32; FEATURE_DIM] {
    let (rms_x, peak_x, kurt_x, crest_x) = axis_stats(&window.x);
    let (rms_y, peak_y, kurt_y, crest_y) = axis_stats(&window.y);
    let (rms_z, peak_z, kurt_z, crest_z) = axis_stats(&window.z);

    let mut scratch: [f32; WINDOW_SIZE];
    let fft_x: [f32; FFT_BINS];
    let fft_y: [f32; FFT_BINS];
    let fft_z: [f32; FFT_BINS];

    scratch = window.x;
    fft_x = fft_magnitudes(&mut scratch);
    scratch = window.y;
    fft_y = fft_magnitudes(&mut scratch);
    scratch = window.z;
    fft_z = fft_magnitudes(&mut scratch);

    let total_rms = libm::sqrtf(rms_x * rms_x + rms_y * rms_y + rms_z * rms_z);
    let axial_radial = if (rms_x + rms_y) > 1e-10 {
        rms_z / (rms_x + rms_y)
    } else {
        0.0
    };

    [
        rms_x, peak_x, kurt_x, crest_x,
        fft_x[0], fft_x[1], fft_x[2], fft_x[3],
        rms_y, peak_y, kurt_y, crest_y,
        fft_y[0], fft_y[1], fft_y[2], fft_y[3],
        rms_z, peak_z, kurt_z, crest_z,
        fft_z[0], fft_z[1], fft_z[2], fft_z[3],
        axial_radial,
        total_rms,
    ]
}

#[inline]
fn axis_stats(samples: &[f32; WINDOW_SIZE]) -> (f32, f32, f32, f32) {
    (rms(samples), peak(samples), kurtosis(samples), crest_factor(samples))
}
