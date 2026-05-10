use std::time::Instant;

use esp_idf_hal::{peripherals::Peripherals, delay::FreeRtos};
use esp_idf_hal::task::watchdog::TWDTDriver;
use esp_idf_sys as _;

use vibesentinel_features::{extractor::*, fft::WINDOW_SIZE, stats::variance};
use vibesentinel_model::{arch::*, weights::ANOMALY_THRESHOLD};

mod imu;
mod alert;
mod config;

use config::SAMPLE_INTERVAL_MS;

/// Minimum signal variance considered "physically alive."
/// Below this, the sensor is likely frozen/stuck.
const MIN_SIGNAL_VARIANCE: f32 = 0.0001;

/// How often to feed the task watchdog (ms).
const WDT_FEED_INTERVAL_MS: u64 = 100;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Initialize hardware watchdog (5 second timeout)
    let mut twdt = TWDTDriver::new(peripherals.twdt, &Default::default())?;
    twdt.watch_current_task()?;

    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_hal::i2c::I2cConfig::new().baudrate(400_000.into()),
    )?;

    let mut imu = imu::ImuDriver::new(i2c, imu::AccelRange::G8)?;
    let mut led = alert::LedAlert::new(peripherals.pins.gpio2);

    let mut buf_x = [0.0f32; WINDOW_SIZE];
    let mut buf_y = [0.0f32; WINDOW_SIZE];
    let mut buf_z = [0.0f32; WINDOW_SIZE];
    let mut sample_idx = 0usize;

    // State tracking for change-only logging and sensor health
    let mut was_anomaly = false;
    let mut last_wdt_feed = Instant::now();
    let mut last_sample = Instant::now();

    log::info!("VibeSentinel online. Range: 8G | Threshold: {:.6} | {}ms period",
        ANOMALY_THRESHOLD, SAMPLE_INTERVAL_MS);

    loop {
        // --- Watchdog feeding ---
        if last_wdt_feed.elapsed().as_millis() as u64 >= WDT_FEED_INTERVAL_MS {
            twdt.feed()?;
            last_wdt_feed = Instant::now();
        }

        // --- Precise sampling timer (#1 fix) ---
        let elapsed = last_sample.elapsed();
        let elapsed_ms = elapsed.as_millis() as u32;
        if elapsed_ms < SAMPLE_INTERVAL_MS {
            FreeRtos::delay_ms((SAMPLE_INTERVAL_MS - elapsed_ms) as u32);
        }
        last_sample = Instant::now();

        // --- I2C read with error recovery (#5 fix) ---
        let (x, y, z) = match imu.read_accel() {
            Ok(v) => v,
            Err(e) => {
                let count = imu.report_error();
                log::error!("IMU error (consecutive: {}): {:?}", count, e);
                if count >= 5 {
                    if let Err(re) = imu.recover() {
                        log::error!("I2C recovery failed: {:?}", re);
                    }
                }
                continue;
            }
        };

        let slot = sample_idx % WINDOW_SIZE;
        buf_x[slot] = x;
        buf_y[slot] = y;
        buf_z[slot] = z;
        sample_idx += 1;

        // Process full window (every 128 samples)
        if sample_idx % WINDOW_SIZE == 0 {
            // --- Sensor health check (#10 fix) ---
            let var_x = variance(&buf_x);
            let var_y = variance(&buf_y);
            let var_z = variance(&buf_z);
            if var_x < MIN_SIGNAL_VARIANCE && var_y < MIN_SIGNAL_VARIANCE && var_z < MIN_SIGNAL_VARIANCE {
                log::error!(
                    "SENSOR_FAILURE: frozen signal detected (var_x={:.8}, var_y={:.8}, var_z={:.8})",
                    var_x, var_y, var_z
                );
                // Fast LED blink to indicate sensor failure (distinct from anomaly alert)
                led.set(true);
                FreeRtos::delay_ms(100);
                led.set(false);
                FreeRtos::delay_ms(100);
                led.set(true);
                FreeRtos::delay_ms(100);
                led.set(false);
                continue;
            }

            let window = AccelWindow { x: buf_x, y: buf_y, z: buf_z };
            let raw_features = extract_features(&window);
            let features = normalize_features(&raw_features);

            let (reconstruction, _latent) = forward(&features);
            let error = reconstruction_error(&features, &reconstruction);
            let is_anomaly = error > ANOMALY_THRESHOLD;

            // --- Log only on state change (#6 fix) ---
            if is_anomaly != was_anomaly {
                if is_anomaly {
                    log::warn!(
                        "!!! ANOMALY DETECTED !!! MSE: {:.6} > Threshold: {:.6}",
                        error, ANOMALY_THRESHOLD
                    );
                } else {
                    log::info!("NORMAL: MSE: {:.6} (returned to normal)", error);
                }
                was_anomaly = is_anomaly;
            }

            led.set(is_anomaly);
        }
    }
}
