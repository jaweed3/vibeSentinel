use esp_idf_hal::i2c::I2cDriver;

const LSM6DS3_ADDR: u8 = 0x6A;
const OUTX_L_A:     u8 = 0x28;
const CTRL1_XL:     u8 = 0x10;

/// Accelerometer full-scale range.
/// Industrial motors may exceed ±2G in normal operation.
/// Default to ±8G to avoid clipping/saturation.
#[derive(Clone, Copy)]
pub enum AccelRange {
    #[allow(dead_code)]
    G2  = 0x00,   // ±2g,  scale 0.000061 g/LSB — pedometer use only
    #[allow(dead_code)]
    G4  = 0x02,   // ±4g,  scale 0.000122 g/LSB
    G8  = 0x03,   // ±8g,  scale 0.000244 g/LSB
    #[allow(dead_code)]
    G16 = 0x04,   // ±16g, scale 0.000488 g/LSB
}

impl AccelRange {
    pub const fn scale(self) -> f32 {
        match self {
            AccelRange::G2  => 0.000061,
            AccelRange::G4  => 0.000122,
            AccelRange::G8  => 0.000244,
            AccelRange::G16 => 0.000488,
        }
    }

    /// Raw i16 value where the sensor saturates for this range.
    const fn saturation_raw(self) -> i16 {
        match self {
            AccelRange::G2  => i16::MAX,
            AccelRange::G4  => i16::MAX,
            AccelRange::G8  => i16::MAX,
            AccelRange::G16 => i16::MAX,
        }
    }
}

const ODR_416HZ: u8 = 0x60; // 416 Hz ODR, range bits OR'd in

/// Max consecutive I2C errors before attempted bus recovery.
const MAX_I2C_ERRORS: u32 = 5;

pub struct ImuDriver<'d> {
    i2c: I2cDriver<'d>,
    range: AccelRange,
    error_count: u32,
}

impl<'d> ImuDriver<'d> {
    pub fn new(mut i2c: I2cDriver<'d>, range: AccelRange) -> anyhow::Result<Self> {
        let ctrl = ODR_416HZ | (range as u8) << 2;
        i2c.write(LSM6DS3_ADDR, &[CTRL1_XL, ctrl], 100)?;
        Ok(Self { i2c, range, error_count: 0 })
    }

    /// Read one triaxial accelerometer sample.
    /// Returns (x, y, z) in m/s² (converts from g to m/s²: × 9.80665).
    pub fn read_accel(&mut self) -> anyhow::Result<(f32, f32, f32)> {
        let mut buf = [0u8; 6];
        self.i2c.write_read(LSM6DS3_ADDR, &[OUTX_L_A], &mut buf, 100)?;

        let raw_x = i16::from_le_bytes([buf[0], buf[1]]);
        let raw_y = i16::from_le_bytes([buf[2], buf[3]]);
        let raw_z = i16::from_le_bytes([buf[4], buf[5]]);

        self.error_count = 0; // successful read resets error counter

        const G_TO_MS2: f32 = 9.80665;
        let scale = self.range.scale() * G_TO_MS2;
        Ok((
            raw_x as f32 * scale,
            raw_y as f32 * scale,
            raw_z as f32 * scale,
        ))
    }

    /// Returns true if any axis is at the saturation limit.
    /// Indicates the signal is being clipped — the vibration amplitude
    /// exceeds the current range. Consider upgrading to a higher range.
    pub fn is_saturated(&self, raw_x: i16, raw_y: i16, raw_z: i16) -> bool {
        let sat = self.range.saturation_raw();
        raw_x.abs() >= sat - 10 || raw_y.abs() >= sat - 10 || raw_z.abs() >= sat - 10
    }

    /// Read raw accelerometer values (for saturation check).
    pub fn read_accel_raw(&mut self) -> anyhow::Result<(i16, i16, i16)> {
        let mut buf = [0u8; 6];
        self.i2c.write_read(LSM6DS3_ADDR, &[OUTX_L_A], &mut buf, 100)?;

        let raw_x = i16::from_le_bytes([buf[0], buf[1]]);
        let raw_y = i16::from_le_bytes([buf[2], buf[3]]);
        let raw_z = i16::from_le_bytes([buf[4], buf[5]]);

        self.error_count = 0;
        Ok((raw_x, raw_y, raw_z))
    }

    /// Call on I2C read failure. Tracks consecutive errors and attempts
    /// bus recovery after MAX_I2C_ERRORS consecutive failures.
    pub fn report_error(&mut self) -> u32 {
        self.error_count += 1;
        self.error_count
    }

    /// Attempt I2C peripheral reset after persistent errors.
    /// Returns Ok if reinit succeeds.
    pub fn recover(&mut self) -> anyhow::Result<()> {
        log::warn!("Attempting I2C bus recovery ({} consecutive errors)", self.error_count);
        // Re-send IMU config in case registers were corrupted
        let range_bits = (self.range as u8) << 2;
        self.i2c.write(LSM6DS3_ADDR, &[CTRL1_XL, ODR_416HZ | range_bits], 100)?;
        self.error_count = 0;
        Ok(())
    }

    pub fn error_count(&self) -> u32 {
        self.error_count
    }
}
