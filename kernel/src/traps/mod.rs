mod irq;
mod syndrome;
mod syscall;
mod trap_frame;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

use self::irq::handle_irq;
use self::syndrome::Syndrome;
use self::syscall::handle_syscall;
use console::kprintln;
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
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    match info.kind {
        Kind::Synchronous => {
            let mut led = LED::new(16);
            for _ in 0..1 {
                led.blink_for(100);
            }
            kprintln!("Exception:\n\tInfo: {:?}", info);
            let syndrome = Syndrome::from(esr);
            kprintln!("\tESR: {:?}", syndrome);
            match syndrome {
                Syndrome::Brk(_) => {
                    // kprintln!("Source PC: 0x{:X}", tf.pc);
                    tf.pc += 4;
                    shell("E> ");
                    kprintln!("Exiting the debug shell.");
                }
                Syndrome::Svc(n) => {
                    handle_syscall(n, tf);
                }
                _ => (),
            }
        }
        Kind::Irq => {
            use self::Interrupt::*;
            //  kprintln!("Exception:\n\tInfo: {:?}", info);

            let intctl = Controller::new();
            for int in [Timer1, Timer3, Usb, Gpio0, Gpio1, Gpio2, Gpio3, Uart].iter() {
                if intctl.is_pending(*int) {
                    // kprintln!("\tInterrupt: {:?}", int);
                    handle_irq(*int, tf);
                }
            }
        }
        _ => (),
    }
}
