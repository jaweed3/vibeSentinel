pub mod dataset;
pub mod model;
pub mod train;
pub mod export;

use burn::backend::Autodiff;
use burn::module::AutodiffModule;
use burn::tensor::{Tensor, TensorData};
use burn_ndarray::NdArray;
use vibesentinel_features::{
    INPUT_DIM, HIDDEN1_DIM, LATENT_DIM, HIDDEN2_DIM, OUTPUT_DIM, FEATURE_DIM,
};
use crate::train::{train_and_calibrate, TrainingConfig};
use crate::export::export_weights;
use clap::Parser;

type MyBackend = NdArray<f32>;
type MyAutodiffBackend = Autodiff<MyBackend>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
struct Cli {
    #[arg(long, short)]
    data: Option<String>,
    #[arg(long, short)]
    output_path: Option<String>,
    #[arg(long, short)]
    epochs: usize,
    #[arg(long, short)]
    learning_rate: f64,
    #[arg(long, short)]
    sigma: f32,
    #[arg(long, short)]
    help: bool
}


fn main() -> anyhow::Result<()> {
    let args = parse_args();

    let data_path = args.get("--data").map(|s| s.as_str());
    let output_path = args.get("--output")
        .map(|s| s.as_str())
        .unwrap_or("crates/vibesentinel-model/src/weights.rs");
    let epochs: usize = args.get("--epochs").and_then(|s| s.parse().ok()).unwrap_or(200);
    let lr: f64 = args.get("--lr").and_then(|s| s.parse().ok()).unwrap_or(1e-3);
    let sigma: f32 = args.get("--sigma").and_then(|s| s.parse().ok()).unwrap_or(3.0);

    let data = if let Some(path) = data_path {
        println!("Loading data from {}...", path);
        dataset::CsvVibrationDataset::from_csv(path)?.windows
    } else {
        println!("No --data specified. Generating synthetic normal data...");
        dataset::generate_synthetic_normal(1000)
    };

    let config = TrainingConfig {
        epochs,
        learning_rate: lr,
        threshold_sigma: sigma,
        ..Default::default()
    };

    let device = Default::default();

    println!("Starting training ({} epochs, lr={}, sigma={})...", epochs, lr, sigma);
    let (model, threshold, mean, std) = train_and_calibrate::<MyAutodiffBackend>(&data, &config, &device);

    // Generate golden features for cross-architecture parity test (#9)
    println!("Generating golden features for parity test...");
    let val_model = model.valid();
    let n_golden = 5.min(data.len());

    let mut golden_inputs: Vec<Vec<f32>> = Vec::with_capacity(n_golden);
    let mut golden_outputs: Vec<Vec<f32>> = Vec::with_capacity(n_golden);

    for i in 0..n_golden {
        let mut norm = [0.0f32; FEATURE_DIM];
        for j in 0..FEATURE_DIM {
            let z = (data[i][j] - mean[j]) / (std[j] + 1e-8);
            norm[j] = z.clamp(-3.0, 3.0);
        }
        golden_inputs.push(norm.to_vec());

        let flat: Vec<f32> = norm.to_vec();
        let tensor_data = TensorData::new(flat, [1, FEATURE_DIM]);
        let input_tensor = Tensor::<MyBackend, 2>::from_data(tensor_data, &device);
        let recon_tensor = val_model.forward(input_tensor);
        let recon: Vec<f32> = recon_tensor.into_data().to_vec::<f32>().unwrap();
        golden_outputs.push(recon);
    }

    println!("Extracting weights...");
    let w_enc1: Vec<f32> = model.enc1.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_enc1: Vec<f32> = model.enc1.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();

    let w_enc2: Vec<f32> = model.enc2.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_enc2: Vec<f32> = model.enc2.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();

    let w_dec1: Vec<f32> = model.dec1.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_dec1: Vec<f32> = model.dec1.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();

    let w_dec2: Vec<f32> = model.dec2.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_dec2: Vec<f32> = model.dec2.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();

    let w_enc1_t = transpose(&w_enc1, INPUT_DIM, HIDDEN1_DIM);
    let w_enc2_t = transpose(&w_enc2, HIDDEN1_DIM, LATENT_DIM);
    let w_dec1_t = transpose(&w_dec1, LATENT_DIM, HIDDEN2_DIM);
    let w_dec2_t = transpose(&w_dec2, HIDDEN2_DIM, OUTPUT_DIM);

    println!("Exporting to {}...", output_path);
    export_weights(
        &w_enc1_t, &b_enc1,
        &w_enc2_t, &b_enc2,
        &w_dec1_t, &b_dec1,
        &w_dec2_t, &b_dec2,
        threshold,
        &mean,
        &std,
        &golden_inputs,
        &golden_outputs,
        output_path,
    )?;

    println!("Done!");
    Ok(())
}

fn transpose(data: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut out = vec![0.0; data.len()];
    for r in 0..rows {
        for c in 0..cols {
            out[c * rows + r] = data[r * cols + c];
        }
    }
    out
}
