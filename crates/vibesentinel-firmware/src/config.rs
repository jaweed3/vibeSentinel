//! Board configuration & pin definitions.
//!
//! ## Supported Boards
//!
//! | Board | IMU | I2C SDA | I2C SCL | LED | UART TX | UART RX |
//! |-------|-----|---------|---------|-----|---------|---------|
//! | XIAO ESP32S3 Sense | External | GPIO6 | GPIO7 | GPIO21 | GPIO43 | GPIO44 |
//! | ESP32-S3 DevKitC   | External | GPIO8 | GPIO9 | GPIO2  | GPIO43 | GPIO44 |
//!
//! The XIAO ESP32S3 Sense does NOT have an onboard IMU.
//! Connect an external LSM6DS3 (address 0x6A) or MPU-6050 (address 0x68) via I2C.

// ── Board Selection ──────────────────────────────────────────────
// Uncomment the board you're using:

/// Seeed Studio XIAO ESP32S3 Sense
pub const BOARD_NAME: &str = "XIAO ESP32S3 Sense";
pub const PIN_I2C_SDA: u32 = 6;
pub const PIN_I2C_SCL: u32 = 7;
pub const PIN_LED: u32 = 21;
pub const I2C_BAUDRATE_HZ: u32 = 400_000;

// /// Generic ESP32-S3 DevKitC
// pub const BOARD_NAME: &str = "ESP32-S3 DevKitC";
// pub const PIN_I2C_SDA: u32 = 8;
// pub const PIN_I2C_SCL: u32 = 9;
// pub const PIN_LED: u32 = 2;
// pub const I2C_BAUDRATE_HZ: u32 = 400_000;

// ── IMU Configuration ────────────────────────────────────────────
pub const IMU_I2C_ADDR: u8 = 0x6A;  // LSM6DS3 default (MPU-6050 = 0x68)
pub const IMU_ODR_HZ: u16 = 416;     // Accelerometer output data rate
pub const IMU_DEFAULT_RANGE: &str = "8G";

// ── Sampling ─────────────────────────────────────────────────────
pub const SAMPLE_RATE_HZ: u32 = 200;
pub const SAMPLE_INTERVAL_MS: u32 = 1000 / SAMPLE_RATE_HZ;

// ── Watchdog ─────────────────────────────────────────────────────
pub const WDT_TIMEOUT_SECS: u8 = 5;
pub const WDT_FEED_INTERVAL_MS: u64 = 100;

// ── Sensor Health ────────────────────────────────────────────────
pub const MIN_SIGNAL_VARIANCE: f32 = 0.0001;

// ── I2C Recovery ─────────────────────────────────────────────────
pub const MAX_I2C_ERRORS: u32 = 5;

// ── Health Reports ───────────────────────────────────────────────
pub const HEALTH_REPORT_INTERVAL_SECS: u64 = 60;
pub const MIN_HEAP_FREE_BYTES: usize = 50 * 1024;  // 50KB minimum
