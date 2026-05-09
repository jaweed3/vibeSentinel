use crate::{stats::*, fft::*};

pub const FEATURE_DIM: usize = 20;

pub struct AccelWindow {
    pub x: [f32; WINDOW_SIZE],
    pub y: [f32; WINDOW_SIZE],
    pub z: [f32; WINDOW_SIZE],
}

pub fn extract_features(window: &AccelWindow) -> [f32; FEATURE_DIM] {
    let rms_x = rms(&window.x);
    let rms_y = rms(&window.y);
    let rms_z = rms(&window.z);

    let fft_x = fft_magnitudes(&window.x);
    let fft_y = fft_magnitudes(&window.y);
    let fft_z = fft_magnitudes(&window.z);

    let total_rms = libm::sqrtf(rms_x*rms_x + rms_y*rms_y + rms_z*rms_z);
    let axial_radial = if (rms_x + rms_y) > 1e-10 {
        rms_z / (rms_x + rms_y)
    } else {
        0.0
    };

    [
        rms_x,
        peak(&window.x),
        kurtosis(&window.x),
        crest_factor(&window.x),
        fft_x[0],
        fft_x[1],
        rms_y,
        peak(&window.y),
        kurtosis(&window.y),
        crest_factor(&window.y),
        fft_y[0],
        fft_y[1],
        rms_z,
        peak(&window.z),
        kurtosis(&window.z),
        crest_factor(&window.z),
        fft_z[0],
        fft_z[1],
        axial_radial,
        total_rms,
    ]
}
