pub mod dataset;
pub mod model;
pub mod train;
pub mod export;

use burn::backend::Autodiff;
use burn_ndarray::NdArray;
use crate::train::{train_and_calibrate, TrainingConfig};
use crate::export::export_weights;

type MyBackend = NdArray<f32>;
type MyAutodiffBackend = Autodiff<MyBackend>;

fn main() -> anyhow::Result<()> {
    println!("Generating synthetic normal data...");
    let data = dataset::generate_synthetic_normal(1000);
    
    let config = TrainingConfig {
        epochs: 50,
        ..Default::default()
    };
    
    let device = Default::default();
    
    println!("Starting training...");
    let (model, threshold, mean, std) = train_and_calibrate::<MyAutodiffBackend>(&data, &config, &device);
    
    println!("Extracting weights...");
    let w_enc1: Vec<f32> = model.enc1.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_enc1: Vec<f32> = model.enc1.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();
    
    let w_enc2: Vec<f32> = model.enc2.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_enc2: Vec<f32> = model.enc2.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();
    
    let w_dec1: Vec<f32> = model.dec1.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_dec1: Vec<f32> = model.dec1.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();
    
    let w_dec2: Vec<f32> = model.dec2.weight.val().into_data().to_vec::<f32>().unwrap();
    let b_dec2: Vec<f32> = model.dec2.bias.unwrap().val().into_data().to_vec::<f32>().unwrap();
    
    let w_enc1_t = transpose(&w_enc1, 20, 10);
    let w_enc2_t = transpose(&w_enc2, 10, 4);
    let w_dec1_t = transpose(&w_dec1, 4, 10);
    let w_dec2_t = transpose(&w_dec2, 10, 20);

    let output_path = "crates/vibesentinel-model/src/weights.rs";
    println!("Exporting to {}...", output_path);
    export_weights(
        &w_enc1_t, &b_enc1,
        &w_enc2_t, &b_enc2,
        &w_dec1_t, &b_dec1,
        &w_dec2_t, &b_dec2,
        threshold,
        &mean,
        &std,
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
