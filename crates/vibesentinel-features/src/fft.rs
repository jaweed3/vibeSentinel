use microfft::real::rfft_128;
use libm::sqrtf;

pub const WINDOW_SIZE: usize = 128;
pub const FFT_BINS: usize = 4;

/// Apply Hann window in-place to reduce spectral leakage.
pub fn apply_hann(buf: &mut [f32; WINDOW_SIZE]) {
    let n = WINDOW_SIZE as f32;
    for i in 0..WINDOW_SIZE {
        let i_f = i as f32;
        buf[i] *= 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * i_f / (n - 1.0)));
    }
}

pub fn spectral_centroid(
    mags: &[f32; FFT_BINS],
    sample_rate: f32
) -> f32 {
    let mut weighted_sum: f32 = 0.0f32;
    let mut magnitude_sum: f32 = 0.0f32;
    
    for i in 0..FFT_BINS {
        let freq = 
            i as f32 * sample_rate / WINDOW_SIZE as f32;

        let mag = mags[i];

        weighted_sum += freq * mag;
        magnitude_sum += mag;
    }

    if magnitude_sum <= 1e-12 {
        return 0.0;
    }

    weighted_sum / magnitude_sum
}

/// Compute magnitude of first N FFT bins from 128 raw samples.
/// Applies Hann window, then FFT. Input buffer is destroyed.
pub fn fft_magnitudes(buf: &mut [f32; WINDOW_SIZE]) -> [f32; FFT_BINS] {
    apply_hann(buf);
    let spectrum = rfft_128(buf);

    let mut result = [0.0f32; FFT_BINS];
    for i in 0..FFT_BINS {
        let re = spectrum[i].re;
        let im = spectrum[i].im;
        result[i] = sqrtf(re * re + im * im);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_dominant_bin() {
        let mut samples = [0.0f32; WINDOW_SIZE];
        use core::f32::consts::PI;
        for i in 0..WINDOW_SIZE {
            let t = i as f32 / 200.0;
            samples[i] = (2.0 * PI * 50.0 * t).sin();
        }

        apply_hann(&mut samples);
        let spectrum = rfft_128(&mut samples);

        let mut max_mag = 0.0;
        let mut max_bin = 0;
        for i in 0..64 {
            let re = spectrum[i].re;
            let im = spectrum[i].im;
            let mag = sqrtf(re * re + im * im);
            if mag > max_mag {
                max_mag = mag;
                max_bin = i;
            }
        }

        // With Hann window, the dominant bin for 50Hz should still be at bin 32,
        // though the peak is slightly broader due to windowing.
        assert_eq!(max_bin, 32);
    }

    #[test]
    fn test_fft_magnitudes_output_size() {
        let mut buf = [0.0f32; WINDOW_SIZE];
        let result = fft_magnitudes(&mut buf);
        assert_eq!(result.len(), FFT_BINS);
    }

    #[test]
    fn test_hann_window_positive() {
        let mut buf = [1.0f32; WINDOW_SIZE];
        apply_hann(&mut buf);
        // Hann window should be zero at edges
        assert!((buf[0] - 0.0).abs() < 1e-6);
        assert!((buf[WINDOW_SIZE - 1] - 0.0).abs() < 1e-6);
        // And positive in the middle
        assert!(buf[WINDOW_SIZE / 2] > 0.9);
    }

    #[test]
    fn test_input_zero_mags() {
        let mags = [0.0f32, 0.0f32, 0.0f32, 0.0f32];
        assert_eq!(spectral_centroid(&mags, 16000.0), 0.0);
    }

    #[test]
    fn test_centroid_matches_single_peak() {
        let mags = [0.0, 0.0, 10.0, 0.0];
        let c = spectral_centroid(&mags, 16000.0);

        assert!((c - 250.0).abs() < 1e-3, "c = {}, expected ~250.0", c);
    }

    #[test]
    fn higher_frequency_has_higher_centroid() {
        let low = [10.0, 1.0, 0.0, 0.0];
        let high = [0.0, 0.0, 1.0, 10.0];

        assert!(spectral_centroid(&low, 16000.0) < spectral_centroid(&high, 16000.0))
    }

    #[test]
    fn uniform_spectrum_centered() {
        let mags = [1.0; 4];

        let c = spectral_centroid(&mags, 16000.0);
        assert!(c > 100.0, "c = {}, expected > 100.0", c);
        assert!(c < 200.0, "c = {}, expected < 200.0", c);
    }

    #[test]
    fn test_centroid_negative_magnitudes() {
        let mags = [0.0, -1.0, -2.0, -3.0];
        let c = spectral_centroid(&mags, 16000.0);
        assert!(!c.is_nan());
        assert!(c >= 0.0);
    }

}
