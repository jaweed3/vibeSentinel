/// Target accelerometer sample rate in Hz.
/// IMU runs at 416Hz ODR; firmware decimates to 200Hz.
pub const SAMPLE_RATE_HZ: u32 = 200;

/// Target interval between samples in milliseconds.
pub const SAMPLE_INTERVAL_MS: u32 = 1000 / SAMPLE_RATE_HZ;

/// Number of consecutive I2C errors before bus recovery attempt.
pub const MAX_I2C_ERRORS: u32 = 5;
