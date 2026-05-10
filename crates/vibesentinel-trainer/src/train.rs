use burn::{
    module::AutodiffModule,
    optim::{AdamConfig, Optimizer, GradientsParams},
    tensor::{backend::AutodiffBackend, Tensor, TensorData},
};
use crate::model::VibeSentinelAutoencoder;
use vibesentinel_features::FEATURE_DIM;
use rand::seq::SliceRandom;

pub struct TrainingConfig {
    pub epochs:           usize,
    pub batch_size:       usize,
    pub learning_rate:    f64,
    pub validation_split: f32,
    pub threshold_sigma:  f32,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 200,
            batch_size: 32,
            learning_rate: 1e-3,
            validation_split: 0.15,
            threshold_sigma: 3.0,
        }
    }
}

pub fn train_and_calibrate<B: AutodiffBackend>(
    data: &[[f32; FEATURE_DIM]],
    config: &TrainingConfig,
    device: &B::Device,
) -> (VibeSentinelAutoencoder<B>, f32, [f32; FEATURE_DIM], [f32; FEATURE_DIM]) {
    let mut feature_mean = [0.0f32; FEATURE_DIM];
    let mut feature_std = [0.0f32; FEATURE_DIM];
    let n = data.len() as f32;

    for i in 0..FEATURE_DIM {
        let sum: f32 = data.iter().map(|w| w[i]).sum();
        feature_mean[i] = sum / n;
        let var_sum: f32 = data.iter().map(|w| (w[i] - feature_mean[i]).powi(2)).sum();
        feature_std[i] = (var_sum / n).sqrt();
    }

    let mut normalized_data = Vec::with_capacity(data.len());
    for w in data {
        let mut norm = [0.0f32; FEATURE_DIM];
        for i in 0..FEATURE_DIM {
            let z = (w[i] - feature_mean[i]) / (feature_std[i] + 1e-8);
            norm[i] = z.clamp(-3.0, 3.0);
        }
        normalized_data.push(norm);
    }

    let mut rng = rand::thread_rng();
    normalized_data.shuffle(&mut rng);

    let split_idx = (normalized_data.len() as f32 * (1.0 - config.validation_split)) as usize;
    let train_set = &normalized_data[..split_idx];
    let val_set = &normalized_data[split_idx..];

    let mut model = VibeSentinelAutoencoder::<B>::new(device);
    let mut optim = AdamConfig::new().init();
    
    // For epoch loop
    for epoch in 0..config.epochs {
        let mut train_loss_sum = 0.0;
        let mut train_batches = 0;
        
        let mut current_train = train_set.to_vec();
        current_train.shuffle(&mut rng);
        
        for chunk in current_train.chunks(config.batch_size) {
            let flat: Vec<f32> = chunk.iter().flat_map(|w| w.iter().copied()).collect();
            let data = TensorData::new(flat, [chunk.len(), FEATURE_DIM]);
            let tensor = Tensor::<B, 2>::from_data(data, device);
            
            let loss = model.loss(tensor.clone()).mean();
            
            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &model);
            
            model = optim.step(config.learning_rate, model, grads);
            
            train_loss_sum += loss.into_data().to_vec::<f32>().unwrap()[0];
            train_batches += 1;
        }

        if epoch % 10 == 0 || epoch == config.epochs - 1 {
            let flat: Vec<f32> = val_set.iter().flat_map(|w| w.iter().copied()).collect();
            let data = TensorData::new(flat, [val_set.len(), FEATURE_DIM]);
            let val_tensor = Tensor::<B::InnerBackend, 2>::from_data(data, device);
            let val_model = model.valid();
            let val_loss = val_model.loss(val_tensor).mean().into_data().to_vec::<f32>().unwrap()[0];
            
            println!("Epoch {}: train={:.6}, val={:.6}", epoch, train_loss_sum / train_batches as f32, val_loss);
        }
    }

    let val_model = model.valid();
    
    let flat: Vec<f32> = val_set.iter().flat_map(|w| w.iter().copied()).collect();
    let data = TensorData::new(flat, [val_set.len(), FEATURE_DIM]);
    let val_tensor = Tensor::<B::InnerBackend, 2>::from_data(data, device);
    let recons = val_model.forward(val_tensor.clone());
    let diff = val_tensor - recons;
    let mut val_errors = (diff.clone() * diff).mean_dim(1).squeeze::<1>(1).into_data().to_vec::<f32>().unwrap();
    
    let val_n = val_errors.len() as f32;
    let mean_err: f32 = val_errors.iter().sum::<f32>() / val_n;
    let std_err: f32 = (val_errors.iter().map(|e| (e - mean_err).powi(2)).sum::<f32>() / val_n).sqrt();
    let threshold = mean_err + config.threshold_sigma * std_err;

    val_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    println!("=== VibeSentinel Calibration Report ===");
    println!("  Validation samples:    {}", val_set.len());
    println!("  Mean recon error:      {:.6}", mean_err);
    println!("  Std  recon error:      {:.6}", std_err);
    println!("  Threshold (k={}):     {:.6}", config.threshold_sigma, threshold);
    println!("\n  Percentile distribution of validation errors:");
    println!("    P50:   {:.6}", val_errors[(val_n * 0.5) as usize]);
    println!("    P90:   {:.6}", val_errors[(val_n * 0.9) as usize]);
    println!("    P95:   {:.6}", val_errors[(val_n * 0.95) as usize]);
    println!("    P99:   {:.6}", val_errors[(val_n * 0.99) as usize]);
    println!("    P99.9: {:.6}", val_errors[(val_n * 0.999).min(val_n - 1.0) as usize]);
    println!("=======================================");

    (model, threshold, feature_mean, feature_std)
}
