#![feature(asm, lang_items)]

extern crate xmodem;
extern crate pi;

pub mod lang_items;

use pi::*;
use std::io;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop {
            asm!("nop" :::: "volatile")
        }
    }
}

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
    loop {
        match xmodem::Xmodem::receive(
            {
                let mut mu = uart::MiniUart::new();
                mu.set_read_timeout(750);
                mu
            },
            unsafe { std::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE) },
        ) {
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                for _ in 0..2 {
                    led.blink_for(300);
                }
                continue;
            }
            Ok(_) => jump_to(BINARY_START),
            _ => {
                // ???: triggered when connecting CP2102 to the computer
                // also noticed: shell rings the bell when CP2102 connected to the computer
                // It seems that some unknown characters is sent when CP2102 connected.
                for _ in 0..2 {
                    led.blink_for(100);
                }
            }
        }
    }
}
