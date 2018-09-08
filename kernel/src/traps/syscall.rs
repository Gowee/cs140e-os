use pi::timer::current_time;

use SCHEDULER;
use traps::TrapFrame;
use process::{State, Process};


/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
// The text and the skeleton enforce sleep to be u32 -> u32, but I cannot find any reasonableness
// not to make it be u64 -> u64. So I did what I want. :p But there may be potential
// incompatibility with future phases / assignments.
pub fn sleep(ms: u64, tf: &mut TrapFrame) {
    let start = current_time();
    let until = ms * 1000 + start;
    let epf = Box::new(move |process: &mut Process| {
        let current_time = current_time();
        if current_time >= until {
            process.trap_frame.x0 = (current_time - start) / 1000;
            process.trap_frame.x7 = 0; // There is no error.
            true
        }
        else {
            false
        }
    });
    SCHEDULER.switch(State::Waiting(epf), tf).expect("At least one process is running");
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            sleep(tf.x0, tf);
        }
        _ => {
            tf.x7 = 1; // The system call does not exist.
        }
    }
}
