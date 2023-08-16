use std::sync::{Arc, atomic::{AtomicBool, self, AtomicUsize, AtomicU32}};
use parking_lot::{Condvar, Mutex};
use static_init::dynamic;

use crate::machine::{WriteResult, ReadResult};

const HART_CYCLES_PER_FRAME: usize = 500000;
pub const HART_CYCLE_UNIT: usize = 500;

#[derive(Clone)]
pub struct HartClockMaster {
    state: Arc<HartClockMasterState>
}

#[derive(Clone, Copy, Debug)]
enum RunState {
    Stopped,
    WaitForInterrupt,
    Run,
    BusError,
}

pub enum ClockEvent {
    Reset(u32),
    Cycles(usize)
}

struct HartClockMasterState {
    pub frame: AtomicUsize,
    pub event_cv_lock: Mutex<()>,
    pub event_cv: Condvar,
    pub interrupts: [AtomicBool; 4],
    pub start_flags: [AtomicBool; 4],
    pub start_address: [AtomicU32; 4],
}

#[dynamic]
pub static HART_CLOCK_MASTER: HartClockMaster = HartClockMaster::new();

impl HartClockMaster {
    pub fn new() -> Self {
        Self {
            state: Arc::new(HartClockMasterState {
                frame: AtomicUsize::new(0),
                event_cv: Condvar::new(),
                event_cv_lock: Mutex::new(()),
                interrupts: [(); 4].map(|_| AtomicBool::new(false)),
                start_flags: [(); 4].map(|_| AtomicBool::new(false)),
                start_address: [(); 4].map(|_| AtomicU32::new(0)),
            })
        }
    }

    pub fn next_frame(&self) {
        self.state.frame.fetch_add(1, atomic::Ordering::AcqRel);
        self.state.event_cv.notify_all();
    }

    pub fn frame(&self) -> u64 {
        self.state.frame.load(atomic::Ordering::Acquire) as u64
    }

    pub fn start_hart(&self, hart: usize, start_address: u32) {
        self.state.start_address[hart].store(start_address, atomic::Ordering::Release);
        self.state.start_flags[hart].store(true, atomic::Ordering::Release);
        self.state.event_cv.notify_all();
    }

    pub fn interrupt_hart(&self, hart: usize) {
        self.state.interrupts[hart].store(true, atomic::Ordering::Release);
        self.state.event_cv.notify_all();
    }
}

pub struct HartClock {
    master: HartClockMaster,
    current_frame: usize,
    elapsed_cycles: usize,
    state: RunState,
    hart: usize,
}

impl HartClock {
    pub fn new(hart: usize) -> Self {
        Self {
            master: HART_CLOCK_MASTER.clone(),
            current_frame: 0,
            elapsed_cycles: 0,
            state: RunState::Stopped,
            hart
        }
    }
    
    pub fn register_cycles(&mut self, cycles: usize) {
        self.elapsed_cycles += cycles;
    }

    pub fn wfi(&mut self) {
        self.state = RunState::WaitForInterrupt;
    }

    pub fn error(&mut self) {
        self.state = RunState::BusError;
    }

    pub fn wait_for_event(&mut self) -> ClockEvent {
        match self.state {
            RunState::Stopped => {
                {
                    let mut event_lock = self.master.state.event_cv_lock.lock();
                    while !self.master.state.start_flags[self.hart].fetch_and(false, atomic::Ordering::AcqRel) {
                        self.master.state.event_cv.wait(&mut event_lock);
                    }
                    self.state = RunState::Run;
                }
                ClockEvent::Reset(self.master.state.start_address[self.hart].load(atomic::Ordering::Acquire))
            }
            RunState::Run => {
                let frame = self.master.state.frame.load(atomic::Ordering::Acquire);
                if self.current_frame != frame {
                    self.current_frame = frame;
                    self.elapsed_cycles = 0;
                }
                if self.elapsed_cycles == HART_CYCLES_PER_FRAME {
                    let mut lock_gaurd = self.master.state.event_cv_lock.lock();
                    let mut frame = self.master.state.frame.load(atomic::Ordering::Acquire);
                    while self.current_frame == frame {
                        self.master.state.event_cv.wait(&mut lock_gaurd);
                        frame = self.master.state.frame.load(atomic::Ordering::Acquire);
                    }
                    self.elapsed_cycles = 0;
                    self.current_frame = frame;
                    ClockEvent::Cycles(HART_CYCLES_PER_FRAME)
                } else {
                    ClockEvent::Cycles(HART_CYCLES_PER_FRAME - self.elapsed_cycles)
                }
            },
            RunState::WaitForInterrupt => {
                {
                    let mut lock_gaurd = self.master.state.event_cv_lock.lock();
                    while !self.master.state.interrupts[self.hart].fetch_and(false, atomic::Ordering::AcqRel) {
                        self.master.state.event_cv.wait(&mut lock_gaurd);
                    }
                }
                self.state = RunState::Run;
                return self.wait_for_event();
            },
            RunState::BusError => {
                loop {
                    std::thread::park();
                }
            }
        }
    }
}

pub fn clock_write_u32(offset: u32, value: u32) -> WriteResult {
    match offset {
        0 | 4 | 8 | 12 => {
            HART_CLOCK_MASTER.start_hart((offset >> 2) as usize, value);
            WriteResult::Ok
        },
        _ => WriteResult::InvalidAddress,
    }
}


pub fn clock_write_u16(offset: u32, value: u16) -> WriteResult {
    clock_write_u32(offset, value as u32)
}

pub fn clock_write_u8(offset: u32, value: u8) -> WriteResult {
    clock_write_u32(offset, value as u32)
}

pub fn clock_read_u32(offset: u32) -> ReadResult<u32> {
    match offset {
        0 | 4 | 8 | 12 => ReadResult::Ok(0),
        _ => ReadResult::InvalidAddress
    }
}

pub fn clock_read_u16(offset: u32) -> ReadResult<u16> {
    clock_read_u32(offset).map(|x| x as u16)
}

pub fn clock_read_u8(offset: u32) -> ReadResult<u8> {
    clock_read_u32(offset).map(|x: u32| x as u8)
}
