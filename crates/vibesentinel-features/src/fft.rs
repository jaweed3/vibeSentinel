use microfft::real::rfft_128;
use libm::sqrtf;

pub const WINDOW_SIZE: usize = 128;
pub const FFT_BINS: usize = 2;

/// Compute magnitude of first N FFT bins from 128 raw samples
pub fn fft_magnitudes(samples: &[f32; WINDOW_SIZE]) -> [f32; FFT_BINS] {
    let mut buf: [f32; WINDOW_SIZE] = *samples;
    let spectrum = rfft_128(&mut buf);

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
    fn test_fft() {
        let mut samples = [0.0f32; WINDOW_SIZE];
        use core::f32::consts::PI;
        for i in 0..WINDOW_SIZE {
            let t = i as f32 / 200.0; // 200Hz sample rate
            samples[i] = (2.0 * PI * 50.0 * t).sin(); // 50Hz sine wave
        }
        
        let mut buf = samples;
        let spectrum = rfft_128(&mut buf);
        
        // At 200Hz sample rate, 128 samples, bin frequency resolution is 200/128 = 1.5625 Hz
        // 50Hz should be at bin 50 / 1.5625 = 32
        
        let re = spectrum[32].re;
        let im = spectrum[32].im;
        let mag_32 = sqrtf(re * re + im * im);
        
        // Assert it's the dominant bin
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
        
        assert_eq!(max_bin, 32);
    }
}
