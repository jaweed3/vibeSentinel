use crate::fft::WINDOW_SIZE;

pub struct SlidingWindow {
    pub buf_x: [f32; WINDOW_SIZE],
    pub buf_y: [f32; WINDOW_SIZE],
    pub buf_z: [f32; WINDOW_SIZE],
    pub index: usize,
}

impl SlidingWindow {
    pub const fn new() -> Self {
        Self {
            buf_x: [0.0; WINDOW_SIZE],
            buf_y: [0.0; WINDOW_SIZE],
            buf_z: [0.0; WINDOW_SIZE],
            index: 0,
        }
    }

    pub fn push(&mut self, x: f32, y: f32, z: f32) -> bool {
        self.buf_x[self.index] = x;
        self.buf_y[self.index] = y;
        self.buf_z[self.index] = z;
        self.index += 1;
        if self.index >= WINDOW_SIZE {
            self.index = 0;
            true
        } else {
            false
        }
    }
}
