use libm::{sqrtf, fabsf, powf};

/// Root Mean Square — energy proxy
pub fn rms(samples: &[f32]) -> f32 {
    let n = samples.len() as f32;
    if n < 1.0 {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    sqrtf(sum_sq / n)
}

/// Peak absolute value
pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| fabsf(x)).fold(0.0f32, f32::max)
}

pub fn skewness(samples: &[f32]) -> f32 {
    let n = samples.len() as f32;
    if n < 2.0 {
        return 0.0;
    }
    let mean: f32 = samples.iter().sum::<f32>() / n;
    let var = variance(samples);
    
    if var < 1e-20 {
        return 0.0;
    }

    let stddev: f32 = var.sqrt();
    if stddev == 0.0 {
        return 0.0;
    }

    let third_moment: f32 = samples
        .iter()
        .map(|x| {
            let d = x - mean;
            d * d * d
        })
        .sum::<f32>() / n;

    third_moment / (stddev * stddev * stddev)
}

/// Variance (population): E[(x - mu)^2]
pub fn variance(samples: &[f32]) -> f32 {
    let n = samples.len() as f32;
    if n < 2.0 {
        return 0.0;
    }
    let mean: f32 = samples.iter().sum::<f32>() / n;
    let sum_sq: f32 = samples.iter().map(|&x| powf(x - mean, 2.0)).sum();
    sum_sq / n
}

/// Crest Factor = Peak / RMS
/// High value (>6) indicates impulsive events — early bearing fault
pub fn crest_factor(samples: &[f32]) -> f32 {
    let r = rms(samples);
    if r < 1e-10 {
        return 0.0;
    }
    let cf = peak(samples) / r;
    if cf.is_nan() || cf.is_infinite() {
        return 0.0;
    }
    cf
}

/// Statistical Kurtosis (4th standardized moment)
/// Normal vibration ≈ 3.0. Bearing fault > 4.0-6.0.
pub fn kurtosis(samples: &[f32]) -> f32 {
    let n = samples.len() as f32;
    if n < 2.0 {
        return 0.0;
    }
    let mean: f32 = samples.iter().sum::<f32>() / n;
    let var = variance(samples);
    // Guard: zero variance (frozen sensor, constant signal)
    if var < 1e-20 {
        return 0.0;
    }
    let fourth: f32 = samples.iter().map(|&x| powf(x - mean, 4.0)).sum::<f32>() / n;
    let k = fourth / powf(var, 2.0);
    if k.is_nan() || k.is_infinite() {
        return 0.0;
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_constant() {
        let samples = [1.0f32; 128];
        assert!((rms(&samples) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rms_empty() {
        assert_eq!(rms(&[]), 0.0);
    }

    #[test]
    fn test_peak() {
        let samples = [1.0, -5.0, 3.0, -2.0];
        assert!((peak(&samples) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_variance_constant() {
        let samples = [5.0f32; 128];
        assert!(variance(&samples) < 1e-10);
    }

    #[test]
    fn test_variance_nonzero() {
        let samples = [1.0f32, -1.0f32];
        assert!((variance(&samples) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_crest_factor_zero() {
        let samples = [0.0f32; 128];
        assert_eq!(crest_factor(&samples), 0.0);
    }

    #[test]
    fn test_crest_factor_normal() {
        let mut samples = [0.0f32; 128];
        for i in 0..128 {
            samples[i] = (i as f32 * 0.1).sin();
        }
        let cf = crest_factor(&samples);
        assert!(cf > 0.0);
        assert!(!cf.is_nan());
    }

    #[test]
    fn test_skewness_zero() {
        let samples = [1.0];
        assert_eq!(skewness(&samples), 0.0);
    }

    #[test]
    fn test_skewness_constant() {
        let samples = [3.0, 3.0, 3.0, 3.0];
        assert_eq!(skewness(&samples), 0.0);
    }

    #[test]
    fn test_skewness_symmetric() {
        let samples = [-2.0, -1.0, 0.0, 1.0, 2.0];
        assert!(skewness(&samples) < 1e-6);
    }

    #[test]
    fn test_skewness_positive() {
        let samples = [1.0, 2.0, 3.0, 4.0, 8.0];
        assert!(skewness(&samples) > 0.0);
    }
    #[test]
    fn test_kurtosis_gaussian() {
        let mut samples = [0.0f32; 128];
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
    fn test_kurtosis_constant_signal() {
        let samples = [3.0f32; 128];
        assert_eq!(kurtosis(&samples), 0.0);
    }

    #[test]
    fn test_kurtosis_impulsive() {
        let mut samples = [0.0f32; 128];
        samples[64] = 10.0; // single large impulse
        let k = kurtosis(&samples);
        // Impulsive signal should have high kurtosis
        assert!(k > 4.0, "Expected k > 4.0 for impulsive signal, got {}", k);
    }
}
