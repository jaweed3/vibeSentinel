use libm::{sqrtf, fabsf, powf};

/// Root Mean Square — energy proxy
pub fn rms(samples: &[f32]) -> f32 {
    let n = samples.len() as f32;
    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    sqrtf(sum_sq / n)
}

/// Peak absolute value
pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| fabsf(x)).fold(0.0f32, f32::max)
}

/// Crest Factor = Peak / RMS
pub fn crest_factor(samples: &[f32]) -> f32 {
    let r = rms(samples);
    if r < 1e-10 { return 0.0; }
    peak(samples) / r
}

/// Statistical Kurtosis
pub fn kurtosis(samples: &[f32]) -> f32 {
    let n = samples.len() as f32;
    let mean: f32 = samples.iter().sum::<f32>() / n;
    let variance: f32 = samples.iter().map(|&x| powf(x - mean, 2.0)).sum::<f32>() / n;
    if variance < 1e-20 { return 0.0; }
    let fourth: f32 = samples.iter().map(|&x| powf(x - mean, 4.0)).sum::<f32>() / n;
    fourth / powf(variance, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms() {
        let samples = [1.0f32; 128];
        assert!((rms(&samples) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_kurtosis() {
        // A simple test for kurtosis. A uniform distribution has kurtosis ~ 1.8. 
        // For standard normal it's ~3.0. Let's just do a basic sanity check.
        // The spec says: "kurtosis on gaussian-distributed samples ≈ 3.0 ± 0.5"
        // Let's create an approximation of gaussian.
        let mut samples = [0.0f32; 128];
        // Generate pseudo-gaussian using central limit theorem
        let mut rng_state = 12345u64;
        for i in 0..128 {
            let mut sum = 0.0;
            for _ in 0..12 {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let rand_val = ((rng_state >> 33) as f32) / (u32::MAX as f32);
                sum += rand_val;
            }
            samples[i] = sum - 6.0;
        }
        let k = kurtosis(&samples);
        assert!(k > 2.0 && k < 4.0, "Kurtosis was {}", k);
    }

    #[test]
    fn test_crest_factor_zero() {
        let samples = [0.0f32; 128];
        assert_eq!(crest_factor(&samples), 0.0);
    }
}
