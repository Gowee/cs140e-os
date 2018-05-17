#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

use pi::gpio;

struct LED {
    pin: gpio::Gpio<gpio::Output>,
}

impl LED {
    fn new(pin: u8) -> LED {
        LED { pin: gpio::Gpio::new(pin).into_output() }
    }

    fn on(&mut self) {
        self.pin.set()
    }

    fn off(&mut self) {
        self.pin.clear()
    }

    fn blink_for(&mut self, duration: u64) {
        self.on();
        pi::timer::spin_sleep_ms(duration);
        self.off();
        pi::timer::spin_sleep_ms(duration);
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    let mut led = LED::new(16);
    for _ in 0..3 {
        led.blink_for(300);
    }
    shell::shell("> ");
    // for _ in 0.. {
    //     led.blink_for(500);
    // }
}
