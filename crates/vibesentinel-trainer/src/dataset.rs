use vibesentinel_features::{extractor::*, fft::WINDOW_SIZE};
use serde::Deserialize;

pub struct CsvVibrationDataset {
    pub windows: Vec<[f32; FEATURE_DIM]>,
}

#[derive(Deserialize)]
pub struct CsvRow {
    pub timestamp: f64,
    pub x: f32,
    pub y: f32,
    pub z: f32
}


impl CsvVibrationDataset {
    pub fn from_csv(path: &str) -> anyhow::Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut zs = Vec::new();

        for result in reader.deserialize::<CsvRow>() {
            let row = result?;
            xs.push(row.x); ys.push(row.y); zs.push(row.z);
        }

        let step = WINDOW_SIZE / 2;
        let windows = (0..xs.len().saturating_sub(WINDOW_SIZE))
            .step_by(step)
            .map(|i| {
                let win = AccelWindow {
                    x: xs[i..i+WINDOW_SIZE].try_into().unwrap(),
                    y: ys[i..i+WINDOW_SIZE].try_into().unwrap(),
                    z: zs[i..i+WINDOW_SIZE].try_into().unwrap(),
                };
                extract_features(&win)
            })
            .collect();

        Ok(Self { windows })
    }
}

pub fn generate_synthetic_normal(n_windows: usize) -> Vec<[f32; FEATURE_DIM]> {
    use core::f32::consts::PI;
    let mut rng_state = 12345u64;
    let mut lcg_rand = move || -> f32 {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng_state >> 33) as f32) / (u32::MAX as f32) * 2.0 - 1.0
    };

    (0..n_windows).map(|_| {
        let mut x = [0.0f32; WINDOW_SIZE];
        let mut y = [0.0f32; WINDOW_SIZE];
        let mut z = [0.0f32; WINDOW_SIZE];
        let phase_x = lcg_rand() * PI;
        let phase_y = lcg_rand() * PI;

        for i in 0..WINDOW_SIZE {
            let t = i as f32 / 200.0;
            x[i] = 0.8*(2.0*PI*50.0*t + phase_x).sin()
                 + 0.2*(2.0*PI*100.0*t).sin()
                 + 0.05*lcg_rand();
            y[i] = 0.6*(2.0*PI*50.0*t + phase_y).sin()
                 + 0.15*(2.0*PI*100.0*t).sin()
                 + 0.05*lcg_rand();
            z[i] = 0.3*(2.0*PI*50.0*t).cos() + 0.05*lcg_rand();
        }

        let win = AccelWindow { x, y, z };
        extract_features(&win)
    }).collect()
}
