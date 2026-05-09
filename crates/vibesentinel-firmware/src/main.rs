use esp_idf_hal::{peripherals::Peripherals, delay::FreeRtos};
use esp_idf_sys as _;

use vibesentinel_features::{extractor::*, fft::WINDOW_SIZE};
use vibesentinel_model::{arch::*, weights::ANOMALY_THRESHOLD};

mod imu; mod alert; mod config;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_hal::i2c::I2cConfig::new().baudrate(400_000.into()),
    )?;

    let mut imu = imu::ImuDriver::new(i2c)?;
    let mut led = alert::LedAlert::new(peripherals.pins.gpio2);

    let mut buf_x = [0.0f32; WINDOW_SIZE];
    let mut buf_y = [0.0f32; WINDOW_SIZE];
    let mut buf_z = [0.0f32; WINDOW_SIZE];
    let mut sample_idx = 0usize;

    log::info!("VibeSentinel online. Threshold: {:.6}", ANOMALY_THRESHOLD);

    loop {
        FreeRtos::delay_ms(5); // 200Hz sampling

        let (x, y, z) = match imu.read_accel() {
            Ok(v) => v,
            Err(e) => { 
                log::error!("IMU error: {:?}", e); 
                FreeRtos::delay_ms(100);
                continue; 
            }
        };

        let slot = sample_idx % WINDOW_SIZE;
        buf_x[slot] = x;
        buf_y[slot] = y;
        buf_z[slot] = z;
        sample_idx += 1;

        if sample_idx % WINDOW_SIZE == 0 {
            let window = AccelWindow { x: buf_x, y: buf_y, z: buf_z };
            let raw_features = extract_features(&window);
            let features = normalize_features(&raw_features);

            let (reconstruction, _latent) = forward(&features);
            let error = reconstruction_error(&features, &reconstruction);
            let is_anomaly = error > ANOMALY_THRESHOLD;

            log::info!(
                "MSE: {:.6} | Threshold: {:.6} | {}",
                error,
                ANOMALY_THRESHOLD,
                if is_anomaly { "!!! ANOMALY !!!" } else { "NORMAL" }
            );

            led.set(is_anomaly);
        }
    }
}
