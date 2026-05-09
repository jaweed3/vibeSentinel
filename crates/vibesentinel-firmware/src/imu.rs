use esp_idf_hal::i2c::I2cDriver;

const LSM6DS3_ADDR: u8 = 0x6A;
const OUTX_L_A:     u8 = 0x28;
const CTRL1_XL:     u8 = 0x10;
const ODR_416HZ_2G: u8 = 0x68;

pub struct ImuDriver<'d> {
    i2c: I2cDriver<'d>,
}

impl<'d> ImuDriver<'d> {
    pub fn new(mut i2c: I2cDriver<'d>) -> anyhow::Result<Self> {
        i2c.write(LSM6DS3_ADDR, &[CTRL1_XL, ODR_416HZ_2G], 100)?;
        Ok(Self { i2c })
    }

    pub fn read_accel(&mut self) -> anyhow::Result<(f32, f32, f32)> {
        let mut buf = [0u8; 6];
        self.i2c.write_read(LSM6DS3_ADDR, &[OUTX_L_A], &mut buf, 100)?;

        let raw_x = i16::from_le_bytes([buf[0], buf[1]]);
        let raw_y = i16::from_le_bytes([buf[2], buf[3]]);
        let raw_z = i16::from_le_bytes([buf[4], buf[5]]);

        const SCALE: f32 = 0.000598;
        Ok((raw_x as f32 * SCALE, raw_y as f32 * SCALE, raw_z as f32 * SCALE))
    }
}
