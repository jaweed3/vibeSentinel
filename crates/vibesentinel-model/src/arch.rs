use vibesentinel_features::{
    INPUT_DIM, HIDDEN1_DIM, LATENT_DIM, HIDDEN2_DIM, OUTPUT_DIM,
};
use crate::{activations::{apply_relu, apply_sigmoid}, matmul::linear_forward, weights::*};

/// Full autoencoder forward pass — all stack-allocated, no heap, no_std safe.
pub fn forward(input: &[f32; INPUT_DIM]) -> ([f32; OUTPUT_DIM], [f32; LATENT_DIM]) {
    // Encoder layer 1: INPUT_DIM → HIDDEN1_DIM
    let mut h1 = [0.0f32; HIDDEN1_DIM];
    linear_forward::<INPUT_DIM, HIDDEN1_DIM>(input, &W_ENC1, &B_ENC1, &mut h1);
    apply_relu::<HIDDEN1_DIM>(&mut h1);

    // Encoder layer 2: HIDDEN1_DIM → LATENT_DIM (bottleneck)
    let mut latent = [0.0f32; LATENT_DIM];
    linear_forward::<HIDDEN1_DIM, LATENT_DIM>(&h1, &W_ENC2, &B_ENC2, &mut latent);
    apply_relu::<LATENT_DIM>(&mut latent);

    // Decoder layer 1: LATENT_DIM → HIDDEN2_DIM
    let mut h2 = [0.0f32; HIDDEN2_DIM];
    linear_forward::<LATENT_DIM, HIDDEN2_DIM>(&latent, &W_DEC1, &B_DEC1, &mut h2);
    apply_relu::<HIDDEN2_DIM>(&mut h2);

    // Decoder layer 2: HIDDEN2_DIM → OUTPUT_DIM
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

/// Z-score normalize using training-set statistics. Clips to [-3, 3].
pub fn normalize_features(raw: &[f32; INPUT_DIM]) -> [f32; INPUT_DIM] {
    let mut out = [0.0f32; INPUT_DIM];
    for i in 0..INPUT_DIM {
        let z = (raw[i] - FEATURE_MEAN[i]) / (FEATURE_STD[i] + 1e-8);
        out[i] = z.clamp(-3.0, 3.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_output_range() {
        let input = [0.0f32; INPUT_DIM];
        let (output, latent) = forward(&input);
        assert_eq!(output.len(), OUTPUT_DIM);
        assert_eq!(latent.len(), LATENT_DIM);
        for &val in output.iter() {
            assert!(val >= 0.0 && val <= 1.0, "output value {} out of [0,1]", val);
        }
    }

    #[test]
    fn test_reconstruction_error_perfect() {
        let x = [0.5f32; INPUT_DIM];
        let err = reconstruction_error(&x, &x);
        assert!(err < 1e-5, "perfect reconstruction should give ~0 error, got {}", err);
    }

    #[test]
    fn test_reconstruction_error_nonzero() {
        let a = [0.5f32; INPUT_DIM];
        let b = [0.0f32; INPUT_DIM];
        let err = reconstruction_error(&a, &b);
        assert!(err > 0.1, "different vectors should give nonzero error");
    }

    #[test]
    fn test_normalize_clips_at_3() {
        let mut raw = [0.0f32; INPUT_DIM];
        raw[0] += 100.0;
        let norm = normalize_features(&raw);
        assert!(norm[0] <= 3.0);
        assert!(norm[0] >= -3.0);
    }

    #[test]
    fn test_normalize_zero_mean() {
        let mut raw = [0.0f32; INPUT_DIM];
        for i in 0..INPUT_DIM {
            raw[i] = FEATURE_MEAN[i];
        }
        let norm = normalize_features(&raw);
        for i in 0..INPUT_DIM {
            assert!(norm[i].abs() < 1.0, "feature {} normalized to {}, expected near 0 for mean input", i, norm[i]);
        }
    }

    #[test]
    #[ignore = "Run trainer to regenerate weights after architecture change"]
    fn test_cross_architecture_parity() {
        for i in 0..GOLDEN_FEATURE_COUNT {
            let (output, _) = forward(&GOLDEN_INPUTS[i]);
            for j in 0..OUTPUT_DIM {
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
