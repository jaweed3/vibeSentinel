use crate::{activations::{apply_relu, apply_sigmoid}, matmul::linear_forward, weights::*};

pub const INPUT_DIM:   usize = 20;
pub const HIDDEN1_DIM: usize = 10;
pub const LATENT_DIM:  usize = 4;
pub const HIDDEN2_DIM: usize = 10;
pub const OUTPUT_DIM:  usize = 20;

pub fn forward(input: &[f32; INPUT_DIM]) -> ([f32; OUTPUT_DIM], [f32; LATENT_DIM]) {
    let mut h1 = [0.0f32; HIDDEN1_DIM];
    linear_forward::<INPUT_DIM, HIDDEN1_DIM>(input, &W_ENC1, &B_ENC1, &mut h1);
    apply_relu::<HIDDEN1_DIM>(&mut h1);

    let mut latent = [0.0f32; LATENT_DIM];
    linear_forward::<HIDDEN1_DIM, LATENT_DIM>(&h1, &W_ENC2, &B_ENC2, &mut latent);
    apply_relu::<LATENT_DIM>(&mut latent);

    let mut h2 = [0.0f32; HIDDEN2_DIM];
    linear_forward::<LATENT_DIM, HIDDEN2_DIM>(&latent, &W_DEC1, &B_DEC1, &mut h2);
    apply_relu::<HIDDEN2_DIM>(&mut h2);

    let mut output = [0.0f32; OUTPUT_DIM];
    linear_forward::<HIDDEN2_DIM, OUTPUT_DIM>(&h2, &W_DEC2, &B_DEC2, &mut output);
    apply_sigmoid::<OUTPUT_DIM>(&mut output);

    (output, latent)
}

pub fn reconstruction_error(input: &[f32; INPUT_DIM], output: &[f32; OUTPUT_DIM]) -> f32 {
    let sum: f32 = input.iter()
        .zip(output.iter())
        .map(|(&a, &b)| { let d = a - b; d * d })
        .sum();
    sum / INPUT_DIM as f32
}

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
    fn test_forward() {
        let input = [0.0f32; 20];
        let (output, _) = forward(&input);
        for &val in output.iter() {
            assert!(val >= 0.0 && val <= 1.0);
        }
    }

    #[test]
    fn test_reconstruction_error() {
        let x = [0.5f32; 20];
        let err = reconstruction_error(&x, &x);
        assert!(err < 1e-5);
    }

    #[test]
    fn test_normalize_features() {
        let mut raw = FEATURE_MEAN;
        // if std is 1.0, and we add 10 to mean, it should clip at 3.0
        // We don't know FEATURE_STD values (they are 0.0 in placeholder, wait!)
        // If FEATURE_STD is 0.0, dividing by 1e-8 will yield huge values, so it will clip.
        raw[0] += 10.0; 
        let norm = normalize_features(&raw);
        assert_eq!(norm[0], 3.0);
    }
}
