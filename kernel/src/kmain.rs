#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(pointer_methods)]
#![feature(i128_type)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod fs;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

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
#[cfg(not(test))]
pub extern "C" fn kmain() {
    let mut led = LED::new(16);
    for _ in 0..3 {
        led.blink_for(300);
    }
    ALLOCATOR.initialize();

    shell::shell("> ");
}
