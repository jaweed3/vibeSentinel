use std::time::Instant;

use esp_idf_hal::{peripherals::Peripherals, delay::FreeRtos};
use esp_idf_hal::task::watchdog::TWDTDriver;
use esp_idf_sys as _;

use vibesentinel_features::{extractor::*, fft::WINDOW_SIZE, stats::variance, INPUT_DIM, OUTPUT_DIM};
use vibesentinel_model::{arch::*, weights::ANOMALY_THRESHOLD};

mod imu;
mod alert;
mod config;
mod debug;

use config::*;
use debug::DiagnosticLogger;

// ── Window processing result ─────────────────────────────────

struct ProcessedWindow {
    mse: f32,
    is_anomaly: bool,
    features: [f32; INPUT_DIM],
    reconstruction: [f32; OUTPUT_DIM],
}

/// Process one full window of 128 samples through the full pipeline.
/// Returns None if the window should be skipped (frozen sensor, NaN).
fn process_window(
    buf_x: &[f32; WINDOW_SIZE],
    buf_y: &[f32; WINDOW_SIZE],
    buf_z: &[f32; WINDOW_SIZE],
    window_num: usize,
    logger: &mut DiagnosticLogger,
) -> Option<ProcessedWindow> {
    // ── Sensor health check ──────────────────────────────────
    let var_x = variance(buf_x);
    let var_y = variance(buf_y);
    let var_z = variance(buf_z);
    if var_x < MIN_SIGNAL_VARIANCE && var_y < MIN_SIGNAL_VARIANCE && var_z < MIN_SIGNAL_VARIANCE {
        logger.log_sensor_frozen(var_x, var_y, var_z);
        return None;
    }

    // ── Feature extraction ───────────────────────────────────
    let window = AccelWindow { x: *buf_x, y: *buf_y, z: *buf_z };
    let raw_features = extract_features(&window);

    // ── NaN guard on features ────────────────────────────────
    for (i, &v) in raw_features.iter().enumerate() {
        if v.is_nan() || v.is_infinite() {
            logger.log_feature_nan(i, v, window_num);
            return None;
        }
    }

    // ── Normalize + Inference ────────────────────────────────
    let features = normalize_features(&raw_features);
    let (reconstruction, _latent) = forward(&features);
    let mse = reconstruction_error(&features, &reconstruction);

    // ── NaN guard on reconstruction ──────────────────────────
    if mse.is_nan() || mse.is_infinite() {
        logger.log_inference_nan(mse, window_num);
        return None;
    }

    Some(ProcessedWindow {
        mse,
        is_anomaly: mse > ANOMALY_THRESHOLD,
        features,
        reconstruction,
    })
}

// ── Main ─────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // ── Board & Peripherals ──────────────────────────────────
    let peripherals = Peripherals::take().unwrap();
    let mut logger = DiagnosticLogger::new();

    // ── Watchdog ─────────────────────────────────────────────
    let mut twdt = TWDTDriver::new(peripherals.twdt, &Default::default())?;
    twdt.watch_current_task()?;
    log::info!("[INIT] WDT: armed ({}s timeout) | OK", WDT_TIMEOUT_SECS);

    // ── I2C & IMU ────────────────────────────────────────────
    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        &esp_idf_hal::i2c::I2cConfig::new().baudrate(400_000.into()),
    )?;

    let mut imu = imu::ImuDriver::new(i2c, imu::AccelRange::G8)?;
    let mut led = alert::LedAlert::new(peripherals.pins.gpio21);

    // ── Boot Diagnostics ─────────────────────────────────────
    let heap_start = unsafe { esp_idf_sys::esp_get_free_heap_size() };
    let stack_wm = unsafe { esp_idf_sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };

    DiagnosticLogger::boot_report(BOARD_NAME, heap_start, stack_wm as usize, ANOMALY_THRESHOLD);

    log::info!("[INIT] I2C pins: SDA=GPIO{} SCL=GPIO{} baud={}kHz",
        PIN_I2C_SDA, PIN_I2C_SCL, I2C_BAUDRATE_HZ / 1000);
    log::info!("[INIT] IMU range: {} | addr: 0x{:02X}", IMU_DEFAULT_RANGE, IMU_I2C_ADDR);
    log::info!("[INIT] Sampling: {}Hz ({}ms period)", SAMPLE_RATE_HZ, SAMPLE_INTERVAL_MS);
    log::info!("[INIT] Feature dim: {} | Window: {} samples", INPUT_DIM, WINDOW_SIZE);
    log::info!("══════════════════════════════════════════");
    log::info!("System ready. Waiting for first window...");

    // ── Runtime State ────────────────────────────────────────
    let mut buf_x = [0.0f32; WINDOW_SIZE];
    let mut buf_y = [0.0f32; WINDOW_SIZE];
    let mut buf_z = [0.0f32; WINDOW_SIZE];
    let mut sample_idx = 0usize;

    let mut was_anomaly = false;
    let mut last_wdt_feed = Instant::now();
    let mut last_sample = Instant::now();

    // ── Main Loop ────────────────────────────────────────────
    loop {
        // Watchdog feeding
        if last_wdt_feed.elapsed().as_millis() as u64 >= WDT_FEED_INTERVAL_MS {
            twdt.feed()?;
            last_wdt_feed = Instant::now();
        }

        // Precise sampling timer
        let elapsed_ms = last_sample.elapsed().as_millis() as u32;
        if elapsed_ms < SAMPLE_INTERVAL_MS {
            FreeRtos::delay_ms((SAMPLE_INTERVAL_MS - elapsed_ms) as u32);
        }
        last_sample = Instant::now();

        // I2C read with recovery
        let (x, y, z) = match imu.read_accel() {
            Ok(v) => v,
            Err(e) => {
                let count = imu.error_count();
                logger.log_i2c_error(count);
                if count >= MAX_I2C_ERRORS {
                    if let Err(re) = imu.recover() {
                        logger.log_i2c_recovery_fail(&re);
                    } else {
                        logger.log_i2c_recovery_ok();
                    }
                }
                continue;
            }
        };

        // Saturation check
        if imu.check_saturation(x, y, z).is_some() {
            logger.log_saturation();
        }

        let slot = sample_idx % WINDOW_SIZE;
        buf_x[slot] = x;
        buf_y[slot] = y;
        buf_z[slot] = z;
        sample_idx += 1;

        // Process full window
        if sample_idx % WINDOW_SIZE == 0 {
            let wnum = sample_idx / WINDOW_SIZE;

            logger.start_window_timer();

            match process_window(&buf_x, &buf_y, &buf_z, wnum, &mut logger) {
                Some(result) => {
                    logger.end_window_timer();
                    logger.record_mse(result.mse);

                    // Track anomaly sessions
                    if result.is_anomaly && !was_anomaly {
                        logger.on_anomaly_start();
                    } else if !result.is_anomaly && was_anomaly {
                        logger.on_anomaly_end();
                    }

                    // State change logging
                    logger.log_window_result(wnum, result.mse, ANOMALY_THRESHOLD, result.is_anomaly, was_anomaly);

                    // On anomaly, log per-feature error breakdown
                    if result.is_anomaly && !was_anomaly {
                        logger.log_top_feature_errors(&result.features, &result.reconstruction);
                    }

                    was_anomaly = result.is_anomaly;
                    led.set(result.is_anomaly);
                }
                None => {
                    logger.end_window_timer();
                    // Frozen sensor — blink pattern
                    led.set(true); FreeRtos::delay_ms(100);
                    led.set(false); FreeRtos::delay_ms(100);
                    led.set(true); FreeRtos::delay_ms(100);
                    led.set(false); FreeRtos::delay_ms(100);
                    led.set(true); FreeRtos::delay_ms(100);
                    led.set(false);
                }
            }

            // Periodic health report
            if logger.should_health_report() {
                let heap_now = unsafe { esp_idf_sys::esp_get_free_heap_size() };
                logger.log_health_report(heap_now);
            }
        }
    }
}
