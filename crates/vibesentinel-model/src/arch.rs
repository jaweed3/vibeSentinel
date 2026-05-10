use vibesentinel_features::{
    INPUT_DIM, HIDDEN1_DIM, LATENT_DIM, HIDDEN2_DIM, OUTPUT_DIM,
};
use crate::{activations::{apply_relu, apply_sigmoid}, matmul::linear_forward, weights::*};

/// Full autoencoder forward pass — all stack-allocated, no heap, no_std safe.
/// Returns: (reconstruction [f32; 20], latent_vec [f32; 4])
pub fn forward(input: &[f32; INPUT_DIM]) -> ([f32; OUTPUT_DIM], [f32; LATENT_DIM]) {
    // Encoder layer 1: 20 → 10
    let mut h1 = [0.0f32; HIDDEN1_DIM];
    linear_forward::<INPUT_DIM, HIDDEN1_DIM>(input, &W_ENC1, &B_ENC1, &mut h1);
    apply_relu::<HIDDEN1_DIM>(&mut h1);

    // Encoder layer 2: 10 → 4 (bottleneck)
    let mut latent = [0.0f32; LATENT_DIM];
    linear_forward::<HIDDEN1_DIM, LATENT_DIM>(&h1, &W_ENC2, &B_ENC2, &mut latent);
    apply_relu::<LATENT_DIM>(&mut latent);

    // Decoder layer 1: 4 → 10
    let mut h2 = [0.0f32; HIDDEN2_DIM];
    linear_forward::<LATENT_DIM, HIDDEN2_DIM>(&latent, &W_DEC1, &B_DEC1, &mut h2);
    apply_relu::<HIDDEN2_DIM>(&mut h2);

    // Decoder layer 2: 10 → 20
    let mut output = [0.0f32; OUTPUT_DIM];
    linear_forward::<HIDDEN2_DIM, OUTPUT_DIM>(&h2, &W_DEC2, &B_DEC2, &mut output);
    apply_sigmoid::<OUTPUT_DIM>(&mut output);

    (output, latent)
}

/// MSE reconstruction error = anomaly score
pub fn reconstruction_error(input: &[f32; INPUT_DIM], output: &[f32; OUTPUT_DIM]) -> f32 {
    let sum: f32 = input.iter()
        .zip(output.iter())
        .map(|(&a, &b)| { let d = a - b; d * d })
        .sum();
    sum / INPUT_DIM as f32
}

/// Z-score normalize feature vector using training-set statistics.
/// Clips to [-3, 3] to prevent extreme values destabilizing sigmoid output.
pub fn normalize_features(raw: &[f32; INPUT_DIM]) -> [f32; INPUT_DIM] {
    let mut out = [0.0f32; INPUT_DIM];
    for i in 0..INPUT_DIM {
        let z = (raw[i] - FEATURE_MEAN[i]) / (FEATURE_STD[i] + 1e-8);
        out[i] = if z > 3.0 { 3.0 } else if z < -3.0 { -3.0 } else { z };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_output_range() {
        let input = [0.0f32; 20];
        let (output, latent) = forward(&input);
        assert_eq!(output.len(), 20);
        assert_eq!(latent.len(), 4);
        for &val in output.iter() {
            assert!(val >= 0.0 && val <= 1.0, "output value {} out of [0,1]", val);
        }
    }

    #[test]
    fn test_reconstruction_error_perfect() {
        let x = [0.5f32; 20];
        let err = reconstruction_error(&x, &x);
        assert!(err < 1e-5, "perfect reconstruction should give ~0 error, got {}", err);
    }

    #[test]
    fn test_reconstruction_error_nonzero() {
        let a = [0.5f32; 20];
        let b = [0.0f32; 20];
        let err = reconstruction_error(&a, &b);
        assert!(err > 0.1, "different vectors should give nonzero error");
    }

    #[test]
    fn test_normalize_clips_at_3() {
        let mut raw = FEATURE_MEAN;
        raw[0] += 100.0; // far above mean
        let norm = normalize_features(&raw);
        assert!(norm[0] <= 3.0);
        assert!(norm[0] >= -3.0);
    }

    #[test]
    fn test_normalize_zero_mean() {
        let raw = FEATURE_MEAN;
        let norm = normalize_features(&raw);
        // Values at mean should normalize to ~0
        for i in 0..20 {
            assert!(norm[i].abs() < 1.0, "feature {} normalized to {}, expected near 0", i, norm[i]);
        }
    }

    #[test]
    fn test_cross_architecture_parity() {
        // Verifies no_std forward() matches the Burn trainer that generated
        // these golden values. Tolerance 1e-5 for f32 display round-trip.
        for i in 0..GOLDEN_FEATURE_COUNT {
            let (output, _) = forward(&GOLDEN_INPUTS[i]);
            for j in 0..20 {
                let diff = (output[j] - GOLDEN_RECONSTRUCTIONS[i][j]).abs();
                assert!(
                    diff < 1e-5,
                    "Parity fail golden[{}][{}]: expected {:.10}, got {:.10}",
                    i, j, GOLDEN_RECONSTRUCTIONS[i][j], output[j]
                );
            }
        }
    }
}
