use std::time::Instant;

use esp_idf_hal::{peripherals::Peripherals, delay::FreeRtos};
use esp_idf_hal::task::watchdog::TWDTDriver;
use esp_idf_sys as _;

use vibesentinel_features::{extractor::*, fft::WINDOW_SIZE, stats::variance};
use vibesentinel_model::{arch::*, weights::ANOMALY_THRESHOLD};

mod imu;
mod alert;
mod config;
mod debug;

use config::*;
use debug::DiagnosticLogger;

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
    // XIAO ESP32S3 Sense: I2C on GPIO6 (SDA) / GPIO7 (SCL)
    let i2c = esp_idf_hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        &esp_idf_hal::i2c::I2cConfig::new().baudrate(400_000.into()),
    )?;

    let mut imu = imu::ImuDriver::new(i2c, imu::AccelRange::G8)?;
    // XIAO ESP32S3 Sense: user LED on GPIO21
    let mut led = alert::LedAlert::new(peripherals.pins.gpio21);

    // ── Boot Diagnostics ─────────────────────────────────────
    let heap_start = unsafe { esp_idf_sys::esp_get_free_heap_size() };
    let stack_wm = unsafe { esp_idf_sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut()) };

    DiagnosticLogger::boot_report(BOARD_NAME, heap_start, stack_wm as usize, ANOMALY_THRESHOLD);

    log::info!("[INIT] I2C pins: SDA=GPIO{} SCL=GPIO{} baud={}kHz",
        PIN_I2C_SDA, PIN_I2C_SCL, I2C_BAUDRATE_HZ / 1000);
    log::info!("[INIT] IMU range: {} | addr: 0x{:02X}", IMU_DEFAULT_RANGE, IMU_I2C_ADDR);
    log::info!("[INIT] Sampling: {}Hz ({}ms period)", SAMPLE_RATE_HZ, SAMPLE_INTERVAL_MS);
    log::info!("[INIT] Sensor health: min variance = {}", MIN_SIGNAL_VARIANCE);
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
    let mut last_health_report = Instant::now();

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
                log::error!("[E001] I2C err #{}: {}", count, e);
                if count >= MAX_I2C_ERRORS {
                    if let Err(re) = imu.recover() {
                        log::error!("[E009] Recovery FAILED: {}", re);
                    }
                }
                continue;
            }
        };

        // Saturation check — warn if signal hitting range limit
        if let Some(warn) = imu.check_saturation(x, y, z) {
            log::warn!("{}", warn);
        }

        let slot = sample_idx % WINDOW_SIZE;
        buf_x[slot] = x;
        buf_y[slot] = y;
        buf_z[slot] = z;
        sample_idx += 1;

        // Process full window
        if sample_idx % WINDOW_SIZE == 0 {
            let wnum = sample_idx / WINDOW_SIZE;

            // Sensor health check
            let var_x = variance(&buf_x);
            let var_y = variance(&buf_y);
            let var_z = variance(&buf_z);
            if var_x < MIN_SIGNAL_VARIANCE && var_y < MIN_SIGNAL_VARIANCE && var_z < MIN_SIGNAL_VARIANCE {
                log::error!(
                    "[E003] SENSOR_FROZEN (window #{}) | var_x={:.10} var_y={:.10} var_z={:.10}",
                    wnum, var_x, var_y, var_z
                );
                log::error!("[E003] Tap sensor, check wiring, verify 3.3V power to IMU");
                // Blink pattern: fast triple blink
                led.set(true); FreeRtos::delay_ms(100);
                led.set(false); FreeRtos::delay_ms(100);
                led.set(true); FreeRtos::delay_ms(100);
                led.set(false); FreeRtos::delay_ms(100);
                led.set(true); FreeRtos::delay_ms(100);
                led.set(false);
                continue;
            }

            // Feature extraction + inference
            let window = AccelWindow { x: buf_x, y: buf_y, z: buf_z };
            let raw_features = extract_features(&window);

            // NaN guard — check features before feeding to model
            let mut feature_nan = false;
            for (i, &v) in raw_features.iter().enumerate() {
                if v.is_nan() || v.is_infinite() {
                    log::error!("[E007] FEATURE_NAN: feature[{}] = {} | window #{} | Input anomaly", i, v, wnum);
                    feature_nan = true;
                    break;
                }
            }
            if feature_nan {
                continue;
            }

            let features = normalize_features(&raw_features);
            let (reconstruction, _latent) = forward(&features);
            let error = reconstruction_error(&features, &reconstruction);

            // NaN guard — check reconstruction
            if error.is_nan() || error.is_infinite() {
                log::error!("[E008] INFERENCE_NAN: MSE={} | window #{} | Check weights.rs integrity", error, wnum);
                continue;
            }

            let is_anomaly = error > ANOMALY_THRESHOLD;

            // Log on state change only
            if is_anomaly != was_anomaly {
                if is_anomaly {
                    log::warn!(
                        "[W#{:<6}] !!! ANOMALY !!! MSE={:.6} > thresh={:.6} | LED ON",
                        wnum, error, ANOMALY_THRESHOLD
                    );
                } else {
                    log::info!(
                        "[W#{:<6}] NORMAL returned | MSE={:.6} < thresh={:.6} | LED OFF",
                        wnum, error, ANOMALY_THRESHOLD
                    );
                }
                was_anomaly = is_anomaly;
            }

            led.set(is_anomaly);

            // Periodic health report
            if last_health_report.elapsed().as_secs() >= HEALTH_REPORT_INTERVAL_SECS {
                let heap_now = unsafe { esp_idf_sys::esp_get_free_heap_size() };
                log::info!(
                    "[HEALTH] uptime={:.0}s | windows={} | heap={}KB free | I2C errors: {}",
                    logger.uptime_secs(),
                    wnum,
                    heap_now / 1024,
                    imu.error_count(),
                );
                DiagnosticLogger::check_heap(heap_now);
                last_health_report = Instant::now();
            }
        }
    }
}
