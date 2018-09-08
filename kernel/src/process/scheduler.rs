use std::collections::VecDeque;

use pi::{interrupt, timer};

use aarch64;
use mutex::Mutex;
use process::{Id, Process, State};
use traps::TrapFrame;

use run_blinky;
use run_shell;

/// The `tick` time.
pub const TICK: u32 = 10 * 1000;

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.0
            .lock()
            .as_mut()
            .expect("scheduler uninitialized")
            .add(process)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::switch()`.
    #[must_use]
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        self.0
            .lock()
            .as_mut()
            .expect("scheduler uninitialized")
            .switch(new_state, tf)
    }

    /// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) {
        *self.0.lock() = Some(Scheduler::new());

        let mut first_process = Process::new().expect("Create the initial process");
        first_process.init(run_shell as u64);
        let tf = &*first_process.trap_frame as *const TrapFrame;
        self.add(first_process);

        let mut second_process = Process::new().expect("Create the second process");
        second_process.init(run_blinky as u64);
        self.add(second_process);

        interrupt::Controller::new().enable(interrupt::Interrupt::Timer1);
        timer::tick_in(TICK);

        unsafe {
            asm!("mov SP, $0
                  bl context_restore
                  adr x0, _start # Should be ADR instead of LDR (which just does not work) 
                  mov SP, x0
                  mov x0, xzr
                  mov lr, xzr
                  eret"
                :: "r"(tf)
                :: "volatile");
        }
    }
}

#[derive(Debug)]
struct Scheduler {
    processes: VecDeque<Process>,
    current: Option<Id>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler {
            processes: VecDeque::new(),
            current: None,
            last_id: None,
        }
    }

    /// Generate new id by increasing it by one for `Some`, `0` otherwise. If `last_id` hits
    /// `Id::max_value`, return `None`.
    fn inc_id(&mut self) -> Option<Id> {
        match self.last_id {
            None => {
                self.last_id = Some(0);
            }
            Some(ref mut id) => {
                // // WARN: If there are too many processes during the running lifecycle of the OS, it may
                // //       panic!
                // *id.checked_add(1);
                if *id == Id::max_value() {
                    return None;
                } else {
                    *id += 1;
                }
            }
        }
        self.last_id
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// If this is the first process added, it is marked as the current process.
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let id = self.inc_id()?;
        process.trap_frame.tpidr = id;

        self.processes.push_back(process);
        if self.current.is_none() {
            self.current = Some(id);
        }
        Some(id)
    }

    /// Sets the current process's state to `new_state`, finds the next process
    /// to switch to, and performs the context switch on `tf` by saving `tf`
    /// into the current process and restoring the next process's trap frame
    /// into `tf`. If there is no current process, returns `None`. Otherwise,
    /// returns `Some` of the process ID that was context switched into `tf`.
    ///
    /// This method blocks until there is a process to switch to, conserving
    /// energy as much as possible in the interim.
    fn switch(&mut self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        // Not needed! The code inside loop will relaunch the current process if no others 
        // available.
        /* if self.processes.len() == 1 {
            return self.current();
        } */
        let mut current_process = self.processes.pop_front()?;
        current_process.trap_frame = Box::new(*tf); // Save runtime trap_frame
        current_process.state = new_state;
        let current_process_id = current_process.id();
        self.processes.push_back(current_process);

        loop {
            let mut process = self
                .processes
                .pop_front()
                .unwrap_or_else(|| unreachable!("Processes queue can never be empty here."));
            if process.is_ready() {
                // If no others available, the current can also be relaunched.
                *tf = *process.trap_frame;
                process.state = State::Running;
                self.current = Some(process.id());
                self.processes.push_front(process);
                return self.current;
            } else if process.id() == current_process_id {
                // One cycle is ended.

                // <del>
                // Does wfi really work? `switch` is invoked only in `handle_exception` during the
                // execution of which all other interrupts are masked automatically by the CPU.
                // So it seems that there will never be new interrupt raised.
                // </del>
                // See question - wfi: interrupts triggered by events
                aarch64::wfi();
            }
            self.processes.push_back(process);
        }
    }
}
