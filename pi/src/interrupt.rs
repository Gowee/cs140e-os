use common::IO_BASE;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile};

const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IRQ_basic_pending: ReadVolatile<u32>,
    IRQ_pending: [ReadVolatile<u32>; 2],
    FIQ_control: Volatile<u32>,
    enable_IRQs: [Volatile<u32>; 2],
    enalbe_basic_IRQs: Volatile<u32>,
    disable_IRQs: [Volatile<u32>; 2],
    disalbe_basic_IRQs: Volatile<u32>,
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Controller {
        Controller {
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let int = int as usize;
        let n = int / 32;
        let i = int % 32;
        self.registers.enable_IRQs[n].or_mask(0b1 << i);
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let int = int as usize;
        let n = int / 32;
        let i = int % 32;
        self.registers.disable_IRQs[n].or_mask(0b1 << i);
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        let int = int as usize;
        let n = int / 32;
        let i = int % 32;
        self.registers.IRQ_pending[n].has_mask(0b1 << i)
    }
}
