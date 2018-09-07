mod irq;
mod trap_frame;
mod syndrome;
mod syscall;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

use console::kprintln;
use self::syndrome::Syndrome;
use self::irq::handle_irq;
use self::syscall::handle_syscall;
use shell::shell;
use LED;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    let mut led = LED::new(16);
    for _ in 0..1 {
        led.blink_for(100);
    }
    kprintln!("Exception:\n\tInfo: {:?}", info);
    if info.kind == Kind::Synchronous {
        let syndrome = Syndrome::from(esr);
        kprintln!("\tESR: {:?}", syndrome);
        if let Syndrome::Brk(_) = syndrome {
            // kprintln!("Source PC: 0x{:X}", tf.pc);
            tf.pc += 4;
            shell("E> ");
            kprintln!("Exiting the debug shell.");
        }
    }
}
