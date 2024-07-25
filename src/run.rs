use std::sync::Arc;

use crate::{hart::{StepState, Hart, csrs::SharedCSRs}, machine::Machine, hart_clock::{HartClockMaster, HartClock, ClockEvent, HART_CLOCK_MASTER}, ui::main_window::{self, MainWindow}, interrupt_controller::{INTERRUPT_CONTROLLER, PendingInterrupt}};

pub const ROM_START_ADDRESS: u32 = 0xF800_0000;

pub fn run(machine: &Arc<Machine>) {
    let shared_csrs = SharedCSRs::new();
    let hart0 = Hart::new(0xF800_0000, 0, &shared_csrs, &machine);
    let hart1 = Hart::new(0xF800_0000, 1, &shared_csrs, &machine);
    let hart2 = Hart::new(0xF800_0000, 2, &shared_csrs, &machine);
    let hart3 = Hart::new(0xF800_0000, 3, &shared_csrs, &machine);

    HART_CLOCK_MASTER.start_hart(0, ROM_START_ADDRESS);

    std::thread::spawn(move || run_hart_clocked(1, hart1));
    std::thread::spawn(move || run_hart_clocked(2, hart2));
    std::thread::spawn(move || run_hart_clocked(3, hart3));
    run_hart_clocked(0, hart0);
}

fn run_hart_clocked(hart_id: usize, mut hart: Hart) {
    let mut clock = HartClock::new(hart_id);
    loop {
        let event = clock.wait_for_event();
        match event {
            ClockEvent::Reset(reset_addr) => {
                hart.reset(reset_addr);
            },
            ClockEvent::Cycles(cycles) => {
                let mut elapsed_cycles = 0;
                for i in 0..cycles {
                    match hart.single_step::<false>() {
                        StepState::Run => {
                            elapsed_cycles += 1;
                        },
                        StepState::WaitForInterrupt => {
                            clock.wfi();
                            elapsed_cycles += 1;
                            break;
                        }
                        StepState::InstructionError | StepState::BusError => {
                            clock.error();
                            break;
                        }
                    }
                }
                clock.register_cycles(elapsed_cycles);
            }
        }
    }
}
