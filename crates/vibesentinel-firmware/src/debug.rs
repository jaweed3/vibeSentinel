//! Structured error codes & health diagnostics for VibeSentinel firmware.
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
use vibesentinel_features::INPUT_DIM;

// ── Rolling Statistics ────────────────────────────────────────

struct RollingStats {
    count: u64,
    sum: f32,
    sum_sq: f32,
    min: f32,
    max: f32,
}

impl RollingStats {
    fn new() -> Self {
        Self { count: 0, sum: 0.0, sum_sq: 0.0, min: f32::MAX, max: f32::MIN }
    }

    fn push(&mut self, val: f32) {
        self.count += 1;
        self.sum += val;
        self.sum_sq += val * val;
        if val < self.min { self.min = val; }
        if val > self.max { self.max = val; }
    }

    fn mean(&self) -> f32 { self.sum / self.count.max(1) as f32 }
    fn std(&self) -> f32 {
        let m = self.mean();
        (self.sum_sq / self.count.max(1) as f32 - m * m).sqrt()
    }

    fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        self.sum_sq = 0.0;
        self.min = f32::MAX;
        self.max = f32::MIN;
    }
}

struct TimingStats {
    count: u64,
    total_us: u64,
    min_us: u64,
    max_us: u64,
}

impl TimingStats {
    fn new() -> Self {
        Self { count: 0, total_us: 0, min_us: u64::MAX, max_us: 0 }
    }

    fn push(&mut self, elapsed_us: u64) {
        self.count += 1;
        self.total_us += elapsed_us;
        if elapsed_us < self.min_us { self.min_us = elapsed_us; }
        if elapsed_us > self.max_us { self.max_us = elapsed_us; }
    }

    fn avg_us(&self) -> u64 { self.total_us / self.count.max(1) }

    fn reset(&mut self) {
        self.count = 0;
        self.total_us = 0;
        self.min_us = u64::MAX;
        self.max_us = 0;
    }
}

// ── Error Counts ─────────────────────────────────────────────

#[derive(Default)]
pub struct ErrorCounts {
    pub i2c_errors: u32,
    pub sensor_failures: u32,
    pub saturations: u32,
    pub inference_nans: u32,
    pub feature_nans: u32,
}

// ── Anomaly Session ──────────────────────────────────────────

struct AnomalySession {
    start_time: Instant,
}

// ── DiagnosticLogger ─────────────────────────────────────────

pub struct DiagnosticLogger {
    boot_time: Instant,
    window_count: u64,
    pub error_counts: ErrorCounts,
    last_health_report: Instant,
    last_report_windows: u64,

    // Advanced: timing
    timing: TimingStats,
    window_timer: Option<Instant>,

    // Advanced: rolling MSE stats per report period
    mse_stats: RollingStats,

    // Advanced: anomaly sessions
    current_anomaly: Option<AnomalySession>,
    anomaly_sessions: u64,
    total_anomaly_time_us: u64,
}

impl DiagnosticLogger {
    pub fn new() -> Self {
        Self {
            boot_time: Instant::now(),
            window_count: 0,
            error_counts: ErrorCounts::default(),
            last_health_report: Instant::now(),
            last_report_windows: 0,
            timing: TimingStats::new(),
            window_timer: None,
            mse_stats: RollingStats::new(),
            current_anomaly: None,
            anomaly_sessions: 0,
            total_anomaly_time_us: 0,
        }
    }

    // ── Boot ─────────────────────────────────────────────────

    pub fn boot_report(board: &str, heap_free: usize, stack_watermark: usize, threshold: f32) {
        log::info!("╔══════════════════════════════════════════╗");
        log::info!("║   VibeSentinel v0.3.0 — BOOT REPORT     ║");
        log::info!("╠══════════════════════════════════════════╣");
        log::info!("║ Board:     {: <30}║", board);
        log::info!("║ Free heap: {: >8} bytes ({:.1} KB)     ║", heap_free, heap_free as f32 / 1024.0);
        log::info!("║ Stack wm:  {: >8} bytes ({:.1} KB)     ║", stack_watermark, stack_watermark as f32 / 1024.0);
        log::info!("║ Threshold: {:.6}                       ║", threshold);
        log::info!("║ Sampling:  200Hz (5ms target)           ║");
        log::info!("║ Feature dim: {}                           ║", INPUT_DIM);
        log::info!("╚══════════════════════════════════════════╝");
    }

    // ── Window processing timing ─────────────────────────────

    pub fn start_window_timer(&mut self) {
        self.window_timer = Some(Instant::now());
    }

    pub fn end_window_timer(&mut self) {
        if let Some(start) = self.window_timer {
            let elapsed = start.elapsed().as_micros() as u64;
            self.timing.push(elapsed);
            self.window_timer = None;
        }
    }

    // ── MSE tracking ─────────────────────────────────────────

    pub fn record_mse(&mut self, mse: f32) {
        self.mse_stats.push(mse);
    }

    // ── Anomaly tracking ─────────────────────────────────────

    pub fn on_anomaly_start(&mut self) {
        if self.current_anomaly.is_none() {
            self.current_anomaly = Some(AnomalySession { start_time: Instant::now() });
            self.anomaly_sessions += 1;
        }
    }

    pub fn on_anomaly_end(&mut self) {
        if let Some(session) = self.current_anomaly.take() {
            let duration = session.start_time.elapsed().as_micros() as u64;
            self.total_anomaly_time_us += duration;
        }
    }

    // ── Per-feature error breakdown ──────────────────────────

    /// Log top-3 features contributing to reconstruction error.
    pub fn log_top_feature_errors(
        &self,
        input: &[f32; INPUT_DIM],
        reconstruction: &[f32; INPUT_DIM],
    ) {
        let mut errors: [(usize, f32); INPUT_DIM] = [(0, 0.0f32); INPUT_DIM];
        for i in 0..INPUT_DIM {
            let d = input[i] - reconstruction[i];
            errors[i] = (i, d * d);
        }
        // Partial sort: find top 3 by bubble-up
        let mut top3 = [errors[0], errors[1], errors[2]];
        for &e in errors.iter().skip(3) {
            let mut min_idx = 0;
            let mut min_val = top3[0].1;
            for j in 1..3 {
                if top3[j].1 < min_val {
                    min_val = top3[j].1;
                    min_idx = j;
                }
            }
            if e.1 > min_val {
                top3[min_idx] = e;
            }
        }
        // Sort top3 descending
        if top3[0].1 < top3[1].1 { top3.swap(0, 1); }
        if top3[1].1 < top3[2].1 { top3.swap(1, 2); }
        if top3[0].1 < top3[1].1 { top3.swap(0, 1); }

        log::info!(
            "[DEBUG] Top feature errors: f[{}]={:.6}  f[{}]={:.6}  f[{}]={:.6}",
            top3[0].0, top3[0].1,
            top3[1].0, top3[1].1,
            top3[2].0, top3[2].1,
        );
    }

    // ── State change logging ─────────────────────────────────

    pub fn log_window_result(&mut self, window_num: usize, mse: f32, threshold: f32, is_anomaly: bool, was_anomaly: bool) {
        self.window_count += 1;

        if is_anomaly != was_anomaly {
            let state = if is_anomaly { "!!! ANOMALY !!!" } else { "NORMAL returned" };
            log::info!(
                "[W#{:<6}] {} | MSE={:.6} > thresh={:.6} | LED {}",
                window_num, state, mse, threshold,
                if is_anomaly { "ON" } else { "OFF" },
            );
        }
    }

    // ── Error logging ────────────────────────────────────────

    pub fn log_i2c_error(&mut self, consecutive: u32) {
        self.error_counts.i2c_errors += 1;
        log::error!(
            "[E001] I2C_TIMEOUT #{} (total #{}) | Check SDA/SCL wiring & pull-ups",
            consecutive, self.error_counts.i2c_errors,
        );
    }

    pub fn log_i2c_recovery_ok(&mut self) {
        log::warn!("[I2C] Recovery OK — bus nominal");
    }

    pub fn log_i2c_recovery_fail(&mut self, err: &dyn core::fmt::Debug) {
        log::error!(
            "[E009] I2C_RECOVERY_FAIL: {:?} | Power-cycle sensor, check EMI shielding",
            err
        );
    }

    pub fn log_sensor_frozen(&mut self, var_x: f32, var_y: f32, var_z: f32) {
        self.error_counts.sensor_failures += 1;
        log::error!(
            "[E003] SENSOR_FROZEN (total #{}) | var_x={:.10} var_y={:.10} var_z={:.10} | Tap sensor, check wiring",
            self.error_counts.sensor_failures, var_x, var_y, var_z,
        );
    }

    pub fn log_saturation(&mut self) {
        self.error_counts.saturations += 1;
        if self.error_counts.saturations % 10 == 1 {
            log::warn!(
                "[E010] SATURATION (#{}) | Increase G-range (8G→16G)",
                self.error_counts.saturations,
            );
        }
    }

    pub fn log_feature_nan(&mut self, index: usize, value: f32, window: usize) {
        self.error_counts.feature_nans += 1;
        log::error!(
            "[E007] FEATURE_NAN (total #{}) | feature[{}]={} | window #{} | Input signal anomaly",
            self.error_counts.feature_nans, index, value, window,
        );
    }

    pub fn log_inference_nan(&mut self, mse: f32, window: usize) {
        self.error_counts.inference_nans += 1;
        log::error!(
            "[E008] INFERENCE_NAN (total #{}) | MSE={} | window #{} | Corrupted weights or extreme input?",
            self.error_counts.inference_nans, mse, window,
        );
    }

    // ── Health report ────────────────────────────────────────

    pub fn log_health_report(&mut self, heap_free: usize) {
        let uptime = self.boot_time.elapsed().as_secs();
        let windows_since_last = self.window_count - self.last_report_windows;

        let avg_timing = if self.timing.count > 0 {
            Some((self.timing.avg_us(), self.timing.min_us, self.timing.max_us))
        } else {
            None
        };

        let mse_avg = self.mse_stats.mean();
        let mse_std = self.mse_stats.std();
        let mse_min = self.mse_stats.min;
        let mse_max = self.mse_stats.max;

        let anomaly_pct = if windows_since_last > 0 {
            (self.mse_stats.count as f64 / windows_since_last as f64) * 100.0
        } else {
            0.0
        };

        log::info!("══════════════════ HEALTH REPORT ══════════════════");
        log::info!("  Uptime:        {}s", uptime);
        log::info!("  Windows:       {} ({} since last)", self.window_count, windows_since_last);
        log::info!("  Heap:          {}KB free", heap_free / 1024);

        if let Some((avg, min, max)) = avg_timing {
            log::info!("  Window timing: avg={}us min={}us max={}us", avg, min, max);
        }

        log::info!("  MSE stats:     avg={:.6} std={:.6} min={:.6} max={:.6}", mse_avg, mse_std, mse_min, mse_max);
        log::info!("  Anomaly rate:  {:.1}% ({} windows above threshold)", anomaly_pct, self.mse_stats.count);

        if self.anomaly_sessions > 0 {
            let avg_anomaly_us = self.total_anomaly_time_us / self.anomaly_sessions;
            log::info!("  Anomaly sessions: {} (avg {}us each)", self.anomaly_sessions, avg_anomaly_us);
        }

        log::info!("  Errors:        i2c={} sensor={} sat={} nan={}",
            self.error_counts.i2c_errors,
            self.error_counts.sensor_failures,
            self.error_counts.saturations,
            self.error_counts.inference_nans + self.error_counts.feature_nans,
        );
        log::info!("════════════════════════════════════════════════════");

        // Check heap
        if heap_free < 50 * 1024 {
            log::error!(
                "[E005] HEAP_LOW: {}KB free (< 50KB) | Possible memory leak",
                heap_free / 1024,
            );
        }

        // Reset per-period stats
        self.timing.reset();
        self.mse_stats = RollingStats::new();
        self.anomaly_sessions = 0;
        self.total_anomaly_time_us = 0;
        self.last_report_windows = self.window_count;
        self.last_health_report = Instant::now();
    }

    pub fn should_health_report(&self) -> bool {
        self.last_health_report.elapsed().as_secs() >= 60
    }

    pub fn uptime_secs(&self) -> u64 {
        self.boot_time.elapsed().as_secs()
    }
}
