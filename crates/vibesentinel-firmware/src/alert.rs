use esp_idf_hal::gpio::{OutputPin, PinDriver, Output};

pub struct LedAlert<'d, P: OutputPin> {
    pin: PinDriver<'d, P, Output>,
}

impl<'d, P: OutputPin> LedAlert<'d, P> {
    pub fn new(pin: P) -> Self {
        Self { pin: PinDriver::output(pin).expect("LED GPIO init") }
    }
    pub fn set(&mut self, alert: bool) {
        if alert { self.pin.set_high().ok(); }
        else     { self.pin.set_low().ok();  }
    }
}
