use esp_idf_hal::i2c::I2cDriver;
use crate::config;

const LSM6DS3_ADDR: u8 = config::IMU_I2C_ADDR;
const OUTX_L_A:     u8 = 0x28;
const CTRL1_XL:     u8 = 0x10;

/// Accelerometer full-scale range.
#[derive(Clone, Copy, PartialEq)]
pub enum AccelRange {
    G2  = 0x00,   // ±2g,  scale 0.000061 g/LSB — pedometer only, NOT for industrial
    G4  = 0x02,   // ±4g,  scale 0.000122 g/LSB
    G8  = 0x03,   // ±8g,  scale 0.000244 g/LSB — default for motor monitoring
    G16 = 0x04,   // ±16g, scale 0.000488 g/LSB — heavy machinery
}

impl AccelRange {
    pub const fn scale_g_per_lsb(self) -> f32 {
        match self {
            AccelRange::G2  => 0.000061,
            AccelRange::G4  => 0.000122,
            AccelRange::G8  => 0.000244,
            AccelRange::G16 => 0.000488,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            AccelRange::G2  => "2G",
            AccelRange::G4  => "4G",
            AccelRange::G8  => "8G",
            AccelRange::G16 => "16G",
        }
    }

    pub const fn max_raw(self) -> i16 {
        i16::MAX // All ranges saturate at ±32767 in raw units
    }
}

fn range_ctrl_bits(range: AccelRange) -> u8 {
    let odr_bits: u8 = 0x60; // 416Hz ODR
    let fs_bits = (range as u8) << 2; // FS[1:0] at bits 3:2
    odr_bits | fs_bits
}

/// Number of LSb considered "near saturation" for warning threshold.
const SATURATION_MARGIN: i16 = 50;

pub struct ImuDriver<'d> {
    i2c: I2cDriver<'d>,
    range: AccelRange,
    error_count: u32,
    addr: u8,
}

impl<'d> ImuDriver<'d> {
    /// Initialize IMU with the given I2C driver and range.
    /// Returns Ok if the device responds at the configured address.
    pub fn new(mut i2c: I2cDriver<'d>, range: AccelRange) -> anyhow::Result<Self> {
        let addr = LSM6DS3_ADDR;
        let ctrl = range_ctrl_bits(range);

        // Attempt to configure the IMU
        i2c.write(addr, &[CTRL1_XL, ctrl], 100)
            .map_err(|e| anyhow::anyhow!(
                "[E002] IMU_INIT_FAIL at 0x{:02X}: {}. Check: (1) 3.3V power (2) SDA/SCL wiring (3) pull-up resistors (4) correct I2C address",
                addr, e
            ))?;

        log::info!(
            "[INIT] IMU: addr=0x{:02X} range={} ODR={}Hz | OK",
            addr, range.name(), config::IMU_ODR_HZ
        );

        Ok(Self { i2c, range, error_count: 0, addr })
    }

    /// Read triaxial accelerometer in m/s² (g × 9.80665).
    pub fn read_accel(&mut self) -> anyhow::Result<(f32, f32, f32)> {
        let mut buf = [0u8; 6];
        self.i2c.write_read(self.addr, &[OUTX_L_A], &mut buf, 100)
            .map_err(|e| {
                self.error_count += 1;
                anyhow::anyhow!(
                    "[E001] I2C_TIMEOUT (consecutive #{}, addr=0x{:02X}): {}",
                    self.error_count, self.addr, e
                )
            })?;

        let raw_x = i16::from_le_bytes([buf[0], buf[1]]);
        let raw_y = i16::from_le_bytes([buf[2], buf[3]]);
        let raw_z = i16::from_le_bytes([buf[4], buf[5]]);

        self.error_count = 0; // success resets counter

        const G_TO_MS2: f32 = 9.80665;
        let scale = self.range.scale_g_per_lsb() * G_TO_MS2;

        Ok((
            raw_x as f32 * scale,
            raw_y as f32 * scale,
            raw_z as f32 * scale,
        ))
    }

    /// Check if any axis is near the saturation limit.
    /// Returns a warning string if saturated, None otherwise.
    pub fn check_saturation(&self, x: f32, y: f32, z: f32) -> Option<&'static str> {
        let max_g = self.range.max_raw() as f32 * self.range.scale_g_per_lsb();
        let threshold = max_g * 0.95; // warn at 95% of range
        if x.abs() > threshold || y.abs() > threshold || z.abs() > threshold {
            Some("[E010] SATURATION: signal approaching range limit. Increase G-range")
        } else {
            None
        }
    }

    /// Attempt I2C bus recovery after persistent errors.
    /// Re-sends the IMU configuration to re-establish communication.
    pub fn recover(&mut self) -> anyhow::Result<()> {
        log::warn!(
            "[I2C] Attempting recovery after {} consecutive errors...",
            self.error_count
        );
        let ctrl = range_ctrl_bits(self.range);
        self.i2c.write(self.addr, &[CTRL1_XL, ctrl], 100)
            .map_err(|e| anyhow::anyhow!(
                "[E009] I2C_RECOVERY_FAIL ({} errors, addr=0x{:02X}): {}. Power-cycle sensor?",
                self.error_count, self.addr, e
            ))?;
        self.error_count = 0;
        log::warn!("[I2C] Recovery OK — bus nominal");
        Ok(())
    }

    pub fn error_count(&self) -> u32 { self.error_count }
    pub fn range(&self) -> AccelRange { self.range }
}
