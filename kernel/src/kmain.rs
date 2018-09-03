#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(i128_type)]
#![feature(never_type)]
#![feature(unique)]
#![feature(pointer_methods)]
#![feature(naked_functions)]
#![feature(fn_must_use)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(pointer_methods)]
#![feature(i128_type)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate fat32;
extern crate pi;
extern crate stack_vec;

pub mod aarch64;
pub mod allocator;
pub mod console;
pub mod fs;
pub mod lang_items;
pub mod led;
pub mod mutex;
pub mod process;
pub mod shell;
pub mod traps;
pub mod vm;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;
use process::GlobalScheduler;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub use fs::wait_micros;

use led::LED;

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    let mut led = LED::new(16);
    for _ in 0..3 {
        led.blink_for(300);
    }
    for _ in 0..unsafe { aarch64::current_el() } {
        led.blink_for(1000);
    }
    ALLOCATOR.initialize();
    FILE_SYSTEM.initialize();
    unsafe { asm!("brk 2" :::: "volatile"); }
    loop {
        shell::shell("> ");
    }
}
