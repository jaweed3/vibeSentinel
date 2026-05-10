//! Structured error codes & health diagnostics for VibeSentinel firmware.
//!
//! Every error gets a code, a human-readable description, and a suggested fix.
//! Grep serial output for `[E###]` to find issues fast.
//!
//! ## Error Code Reference
//!
//! | Code | Name | Meaning | Fix |
//! |------|------|---------|-----|
//! | E001 | I2C_TIMEOUT | I2C bus not responding | Check SDA/SCL wiring, pull-up resistors |
//! | E002 | IMU_INIT_FAIL | IMU not detected at 0x6A | Check power (3.3V), check I2C address |
//! | E003 | SENSOR_FROZEN | Signal variance near zero | Sensor stuck; tap sensor, check wiring |
//! | E004 | STACK_OVERFLOW | Stack watermark too low | Increase CONFIG_ESP_MAIN_TASK_STACK_SIZE |
//! | E005 | HEAP_LOW | Free heap below 50KB | Check for memory leak in loop |
//! | E006 | WDT_RESET | Watchdog triggered reset | Check loop duration < 5s |
//! | E007 | FEATURE_NAN | NaN in feature vector | Input signal anomaly; check sensor |
//! | E008 | INFERENCE_NAN | NaN in model output | Corrupted weights or extreme input |
//! | E009 | I2C_RECOVERY_FAIL | I2C bus recovery unsuccessful | Power-cycle sensor, check EMI shielding |
//! | E010 | SATURATION | IMU signal at range limit | Increase G-range (8G → 16G) |

use std::time::Instant;
use core::fmt::Write;

/// Minimal structured log — every important event.
pub struct DiagnosticLogger {
    boot_time: Instant,
    window_count: u64,
    error_counts: ErrorCounts,
    last_heap_report: Instant,
}

#[derive(Default)]
pub struct ErrorCounts {
    pub i2c_errors: u32,
    pub sensor_failures: u32,
    pub saturations: u32,
    pub inference_nans: u32,
    pub feature_nans: u32,
}

impl DiagnosticLogger {
    pub fn new() -> Self {
        Self {
            boot_time: Instant::now(),
            window_count: 0,
            error_counts: ErrorCounts::default(),
            last_heap_report: Instant::now(),
        }
    }

    /// Call once at boot. Prints board info, free heap, threshold.
    pub fn boot_report(board: &str, heap_free: usize, stack_watermark: usize, threshold: f32) {
        log::info!("╔══════════════════════════════════════════╗");
        log::info!("║   VibeSentinel v0.2.0 — BOOT REPORT     ║");
        log::info!("╠══════════════════════════════════════════╣");
        log::info!("║ Board:     {: <30}║", board);
        log::info!("║ Free heap: {: >8} bytes ({:.1} KB)     ║", heap_free, heap_free as f32 / 1024.0);
        log::info!("║ Stack wm:  {: >8} bytes ({:.1} KB)     ║", stack_watermark, stack_watermark as f32 / 1024.0);
        log::info!("║ Threshold: {:.6}                       ║", threshold);
        log::info!("║ Sampling:  200Hz (5ms target)           ║");
        log::info!("╚══════════════════════════════════════════╝");
    }

    /// Call when IMU initializes successfully.
    pub fn imu_ok(addr: u8, range_name: &str, odr_hz: u16) {
        log::info!("[INIT] IMU: 0x{:02X} | range={} | ODR={}Hz | OK", addr, range_name, odr_hz);
    }

    /// Call when watchdog is armed.
    pub fn wdt_ok(timeout_secs: u8) {
        log::info!("[INIT] WDT: armed ({:.0}s timeout) | OK", timeout_secs);
    }

    /// Call on each completed window. Prints compact one-line summary.
    pub fn window_summary(
        &mut self,
        mse: f32,
        threshold: f32,
        is_anomaly: bool,
        heap_free: usize,
        was_anomaly: bool,
    ) {
        self.window_count += 1;

        // Only log on state change — not every window
        if is_anomaly == was_anomaly {
            return;
        }

        let state = if is_anomaly { "ANOMALY" } else { "NORMAL " };
        let marker = if is_anomaly { "!!!" } else { "   " };

        log::info!(
            "[W#{:<6}] MSE={:.6} thresh={:.6} | {} {} | heap={}KB free",
            self.window_count,
            mse,
            threshold,
            marker,
            state,
            heap_free / 1024,
        );
    }

    /// Call on I2C read error.
    pub fn i2c_error(&mut self, consecutive: u32) {
        self.error_counts.i2c_errors += 1;
        log::error!(
            "[E001] I2C_TIMEOUT (consecutive #{}, total #{}) | Check SDA/SCL wiring & pull-ups",
            consecutive,
            self.error_counts.i2c_errors,
        );
    }

    /// Call when I2C recovery succeeds.
    pub fn i2c_recovery_ok(&mut self) {
        log::warn!("[I2C] Recovery OK — bus nominal");
        self.error_counts.i2c_errors = 0;
    }

    /// Call when I2C recovery fails.
    pub fn i2c_recovery_fail(&mut self, err: &dyn core::fmt::Debug) {
        log::error!(
            "[E009] I2C_RECOVERY_FAIL: {:?} | Power-cycle sensor, check EMI shielding",
            err
        );
    }

    /// Call when frozen sensor detected.
    pub fn sensor_frozen(&mut self, var_x: f32, var_y: f32, var_z: f32) {
        self.error_counts.sensor_failures += 1;
        log::error!(
            "[E003] SENSOR_FROZEN (total #{}) | var_x={:.10} var_y={:.10} var_z={:.10} | Tap sensor, check wiring",
            self.error_counts.sensor_failures,
            var_x, var_y, var_z,
        );
    }

    /// Call when IMU signal is saturating.
    pub fn saturation_warning(&mut self, raw_x: i16, raw_y: i16, raw_z: i16) {
        self.error_counts.saturations += 1;
        if self.error_counts.saturations % 10 == 1 {
            // Don't spam — log every 10th
            log::warn!(
                "[E010] SATURATION (#{}) raw=({},{},{}) | Increase G-range (8G→16G)",
                self.error_counts.saturations,
                raw_x, raw_y, raw_z,
            );
        }
    }

    /// Call when NaN detected in features.
    pub fn feature_nan(&mut self, index: usize, value: f32) {
        self.error_counts.feature_nans += 1;
        log::error!(
            "[E007] FEATURE_NAN (total #{}) | feature[{}]={} | Input signal anomaly",
            self.error_counts.feature_nans, index, value,
        );
    }

    /// Call when NaN detected in model output.
    pub fn inference_nan(&mut self, mse: f32) {
        self.error_counts.inference_nans += 1;
        log::error!(
            "[E008] INFERENCE_NAN (total #{}) | MSE={} | Corrupted weights or extreme input?",
            self.error_counts.inference_nans, mse,
        );
    }

    /// Periodic health report — call every ~60 seconds.
    pub fn health_report(&mut self, heap_free: usize) {
        let uptime_secs = self.boot_time.elapsed().as_secs();
        log::info!(
            "[HEALTH] uptime={}s | windows={} | heap={}KB free | errors: i2c={} sensor={} sat={} nan={}",
            uptime_secs,
            self.window_count,
            heap_free / 1024,
            self.error_counts.i2c_errors,
            self.error_counts.sensor_failures,
            self.error_counts.saturations,
            self.error_counts.inference_nans + self.error_counts.feature_nans,
        );
        self.last_heap_report = Instant::now();
    }

    /// Check if it's time for a health report (call in main loop).
    pub fn should_health_report(&self) -> bool {
        self.last_heap_report.elapsed().as_secs() >= 60
    }

    /// Check heap and warn if critically low.
    pub fn check_heap(heap_free: usize) {
        if heap_free < 50 * 1024 {
            log::error!(
                "[E005] HEAP_LOW: {}KB free (< 50KB) | Possible memory leak",
                heap_free / 1024,
            );
        }
    }

    /// Returns the uptime in seconds.
    pub fn uptime_secs(&self) -> u64 {
        self.boot_time.elapsed().as_secs()
    }
}
