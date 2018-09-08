use pi::interrupt::Interrupt;
use pi::timer::tick_in;

use traps::TrapFrame;
use process::{TICK, State};
use SCHEDULER;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    match interrupt {
        Interrupt::Timer1 => {
            // After this, process switching happens automatically due to ELR_EL1 changing in tf.
            SCHEDULER.switch(State::Ready, tf).expect("There are one process running");

            /*
            Acknowledgement is also done inside (i.e. writing into CS register).
            But: Is tick_in really needed?

            According to https://web.stanford.edu/class/cs140e/docs/BCM2837-ARM-Peripherals.pdf
            Chapter 12 System Timer:
            When the two values match, the system timer peripheral generates a signal to indicate a
            match for the appropriate channel. The match signal is then fed into the interrupt
            controller. The interrupt service routine then reads the output compare register and
            adds the appropriate offset for the next timer tick.

            So it seems that what is needed is only ACK (no need to rewrite the COMPARE register).
            */
            tick_in(TICK);
        }
        _ => unimplemented!("Unable to handle IRQ interrupts other than Timer1.")
    }
}
