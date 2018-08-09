use pi::{self, gpio};

pub struct LED {
    pin: gpio::Gpio<gpio::Output>,
}

impl LED {
    pub fn new(pin: u8) -> LED {
        LED {
            pin: gpio::Gpio::new(pin).into_output(),
        }
    }

    pub fn on(&mut self) {
        self.pin.set()
    }

    pub fn off(&mut self) {
        self.pin.clear()
    }

    pub fn blink_for(&mut self, duration: u64) {
        self.on();
        pi::timer::spin_sleep_ms(duration);
        self.off();
        pi::timer::spin_sleep_ms(duration);
    }
}
