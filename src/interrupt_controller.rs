use std::{sync::atomic::{AtomicU32, AtomicBool, self}, future::Pending};

use parking_lot::RwLock;
use static_init::dynamic;

use crate::{machine::{WriteResult, ReadResult}, hart_clock::HART_CLOCK_MASTER, hart::csrs::InterruptBits};

struct Interrupt {
    enabled: bool,
    flag   : bool,
    hart   : u32,
}

impl Interrupt {
    pub fn new() -> Self {
        Self {
            enabled: false,
            flag   : false,
            hart   : 0,
        }
    }
}

struct InterHartInterrupt {
    enabled: bool,
    flag: bool,
}

impl InterHartInterrupt {
    pub fn new() -> Self {
        Self {
            enabled: false,
            flag: false,
        }
    }
}

pub enum InterruptType {
    Gpu,
    Present,
    VSync,
    Spu,
}

pub enum PendingInterrupt {
    External,
    InterHart,
    Timer,
}

pub struct InterruptController {
    gpu_interrupt: RwLock<Interrupt>,
    present_interrupt: RwLock<Interrupt>,
    vsync_interrupt: RwLock<Interrupt>,
    spu_interrupt: RwLock<Interrupt>,
    ihis: [RwLock<InterHartInterrupt>; 4],
    mips: [AtomicU32; 4],
}

#[dynamic]
pub static INTERRUPT_CONTROLLER: InterruptController = InterruptController::new();

impl InterruptController {
    pub fn new() -> Self {
        Self {
            gpu_interrupt: RwLock::new(Interrupt::new()),
            present_interrupt: RwLock::new(Interrupt::new()),
            vsync_interrupt: RwLock::new(Interrupt::new()),
            spu_interrupt: RwLock::new(Interrupt::new()),
            ihis: [(); 4].map(|_| RwLock::new(InterHartInterrupt::new())),
            mips: [(); 4].map(|_| AtomicU32::new(0))
        }
    }

    fn get_interrupt(&self, interrupt: u32) -> Option<&RwLock<Interrupt>> {
        match interrupt {
            0 => Some(&self.gpu_interrupt),
            1 => Some(&self.present_interrupt),
            2 => Some(&self.vsync_interrupt),
            3 => Some(&self.spu_interrupt),
            _ => None
        }
    }

    pub fn set_interrupt_enabled(&self, interrupt: u32, enabled: bool) {
        println!("set_interrupt_enabled(int: {interrupt}, enabled: {enabled})");
        if let Some(interrupt) = self.get_interrupt(interrupt) {
            self.update_and_propogate_interrupt(interrupt, |interrupt| interrupt.enabled = enabled)
        }
    }

    pub fn get_interrupt_enabled(&self, interrupt: u32) -> bool {
        if let Some(interrupt) = self.get_interrupt(interrupt) {
            interrupt.read().enabled
        } else {
            false
        }
    }

    pub fn set_interrupt_hart(&self, interrupt: u32, hart: u32) {
        println!("set_interrupt_hart(int: {interrupt}, hart: {hart})");
        if let Some(interrupt) = self.get_interrupt(interrupt) {
            self.update_and_propogate_interrupt(interrupt, |interrupt| interrupt.hart = hart)
        }
    }

    pub fn get_interrupt_hart(&self, interrupt: u32) -> u32 {
        if let Some(interrupt) = self.get_interrupt(interrupt) {
            interrupt.read().hart
        } else {
            0
        }
    }

    pub fn clear_interrupt(&self, interrupt: u32) {
        if let Some(interrupt) = self.get_interrupt(interrupt) {
            self.update_and_propogate_interrupt(interrupt, |interrupt| interrupt.flag = false)
        }
    }

    pub fn trigger_interrupt(&self, interrupt: InterruptType) {
        match interrupt {
            InterruptType::Gpu => self.update_and_propogate_interrupt(&self.gpu_interrupt, |interrupt| interrupt.flag = true),
            InterruptType::Present => self.update_and_propogate_interrupt(&self.present_interrupt, |interrupt| interrupt.flag = true),
            InterruptType::VSync => self.update_and_propogate_interrupt(&self.vsync_interrupt, |interrupt| interrupt.flag = true),
            InterruptType::Spu => self.update_and_propogate_interrupt(&self.spu_interrupt, |interrupt| interrupt.flag = true),
            _ => {}
        }
    }

    pub fn pending_interrupt(&self, hart: u32) -> Option<PendingInterrupt> {
        if
            Self::check_interrupt(&self.gpu_interrupt.read(), hart) ||
            Self::check_interrupt(&self.vsync_interrupt.read(), hart) ||
            Self::check_interrupt(&self.present_interrupt.read(), hart) ||
            Self::check_interrupt(&self.spu_interrupt.read(), hart) {
            Some(PendingInterrupt::External)
        } else if self.check_ihi(hart) {
            Some(PendingInterrupt::InterHart)
        } else {
            None
        }
    }

    pub fn pending_interrupt_number(&self, hart: u32) -> Option<u32> {
        if Self::check_interrupt(&self.gpu_interrupt.read(), hart) { return Some(0) };
        if Self::check_interrupt(&self.vsync_interrupt.read(), hart) { return Some(1) };
        if Self::check_interrupt(&self.present_interrupt.read(), hart) { return Some(2) };
        if Self::check_interrupt(&self.spu_interrupt.read(), hart) { return Some(3) };
        if Self::check_ihi(&self, hart) { return Some(0xFF) }
        None
    }

    fn check_interrupt(interrupt: &Interrupt, hart: u32) -> bool {
        interrupt.hart == hart && interrupt.enabled && interrupt.flag
    }

    fn check_ihi(&self, hart: u32) -> bool {
        let ihi = self.ihis[hart as usize].read();
        ihi.enabled && ihi.flag
    }

    fn update_and_propogate_interrupt(&self, interrupt: &RwLock<Interrupt>, update_fn: impl Fn(&mut Interrupt)) {
        let mut interrupt = interrupt.write();
        let previous_state = interrupt.enabled && interrupt.flag;
        (update_fn)(&mut interrupt);
        let state = interrupt.enabled && interrupt.flag;
        if state {
            self.mips[interrupt.hart as usize].fetch_or(InterruptBits::MEI, atomic::Ordering::AcqRel);
        } else {
            self.mips[interrupt.hart as usize].fetch_and(!InterruptBits::MEI, atomic::Ordering::AcqRel);
        }
        if state && !previous_state {
            HART_CLOCK_MASTER.interrupt_hart(interrupt.hart as usize);
        }
    }

    pub fn get_interrupt_flag(&self, interrupt: u32) -> bool {
        if let Some(interrupt) = self.get_interrupt(interrupt) {
            interrupt.read().flag
        } else {
            false
        }
    }

    fn update_and_propogate_ihi(&self, ihi: &RwLock<InterHartInterrupt>, hart: u32, update_fn: impl Fn(&mut InterHartInterrupt)) {
        let mut ihi = ihi.write();
        let previous_state = ihi.enabled && ihi.flag;
        (update_fn)(&mut ihi);
        let state = ihi.enabled && ihi.flag;
        if state {
            self.mips[hart as usize].fetch_or(InterruptBits::MSI, atomic::Ordering::AcqRel);
        } else {
            self.mips[hart as usize].fetch_and(!InterruptBits::MSI, atomic::Ordering::AcqRel);
        }
        if state && !previous_state {
            HART_CLOCK_MASTER.interrupt_hart(hart as usize);
        }
    }

    pub fn set_ihi_enabled(&self, hart: u32, enabled: bool) {
        println!("set_ihi_enabled {}: {}", hart, enabled);
        if (0..4).contains(&hart) {
            self.update_and_propogate_ihi(&self.ihis[hart as usize], hart, |ihi| ihi.enabled = enabled);
        }
    }

    pub fn get_ihi_enabled(&self, hart: u32) -> bool {
        if (0..4).contains(&hart) {
            self.ihis[hart as usize].read().enabled
        } else {
            false
        }
    }

    pub fn get_ihi_flag(&self, hart: u32) -> bool {
        if (0..4).contains(&hart) {
            self.ihis[hart as usize].read().flag
        } else {
            false
        }
    }

    pub fn clear_ihi(&self, hart: u32) {
        if (0..4).contains(&hart) {
            self.update_and_propogate_ihi(&self.ihis[hart as usize], hart, |ihi| ihi.flag = false);
        }
    }

    pub fn trigger_ihi(&self, hart: u32) {
        println!("trigger_ihi {}", hart);
        if (0..4).contains(&hart) {
            self.update_and_propogate_ihi(&self.ihis[hart as usize], hart, |ihi| ihi.flag = true);
        }
    }

    pub fn mip(&self, hart: u32) -> u32 {
        if (0..4).contains(&hart) {
            self.mips[hart as usize].load(atomic::Ordering::Acquire)
        } else {
            0
        }
    }
}

pub fn interrupt_controller_write_u32(offset: u32, value: u32) -> WriteResult {
    let register = offset & 0xF;
    let interrupt = offset >> 4;
    match (interrupt, register) {
        (0xFD, register) => {
            let hart = register >> 2;
            INTERRUPT_CONTROLLER.set_ihi_enabled(hart, value != 0);
            WriteResult::Ok
        },
        (0xFE, register) => {
            let hart = register >> 2;
            if (value & 1) != 0 {
                INTERRUPT_CONTROLLER.clear_ihi(hart);
            }
            WriteResult::Ok
        },
        (0xFF, register) => {
            let hart = register >> 2;
            if (value & 1) != 0 {
                INTERRUPT_CONTROLLER.trigger_ihi(hart);
            }
            WriteResult::Ok
        },
        (interrupt, 0) => {
            INTERRUPT_CONTROLLER.set_interrupt_enabled(interrupt, value != 0);
            WriteResult::Ok
        },
        (interrupt, 4) => {
            INTERRUPT_CONTROLLER.set_interrupt_hart(interrupt, value & 3);
            WriteResult::Ok
        },
        (interrupt, 8) => {
            if (value & 1) != 0 {
                INTERRUPT_CONTROLLER.clear_interrupt(interrupt);
            }
            WriteResult::Ok
        },
        _ => WriteResult::InvalidAddress
    }
}

pub fn interrupt_controller_write_u16(offset: u32, value: u16) -> WriteResult {
    interrupt_controller_write_u32(offset, value as u32)
}

pub fn interrupt_controller_write_u8(offset: u32, value: u8) -> WriteResult {
    interrupt_controller_write_u32(offset, value as u32)
}

pub fn interrupt_controller_read_u32(offset: u32) -> ReadResult<u32> {
    let register = offset & 0x0F;
    let interrupt = offset >> 4;
    match (interrupt, register) {
        (0xFC, register) => {
            let hart = register >> 2;
            ReadResult::Ok(INTERRUPT_CONTROLLER.pending_interrupt_number(hart).unwrap_or(0xFFFF_FFFF))
        }
        (0xFD, register) => {
            let hart = register >> 2;
            ReadResult::Ok(if INTERRUPT_CONTROLLER.get_ihi_enabled(hart) { 1 } else { 0 })
        },
        (0xFE | 0xFF, register) => {
            let hart = register >> 2;
            ReadResult::Ok(if INTERRUPT_CONTROLLER.get_ihi_flag(hart) { 1 } else { 0 })
        },
        (interrupt, 0) => ReadResult::Ok(if INTERRUPT_CONTROLLER.get_interrupt_enabled(interrupt) { 1 } else { 0 }),
        (interrupt, 4) => ReadResult::Ok(INTERRUPT_CONTROLLER.get_interrupt_hart(interrupt)),
        (interrupt, 8) => ReadResult::Ok(if INTERRUPT_CONTROLLER.get_interrupt_flag(interrupt) { 1 } else { 0 }),
        _ => ReadResult::InvalidAddress,
    }
}

pub fn interrupt_controller_read_u16(offset: u32) -> ReadResult<u16> {
    interrupt_controller_read_u32(offset).map(|x| x as u16)
}

pub fn interrupt_controller_read_u8(offset: u32) -> ReadResult<u8> {
    interrupt_controller_read_u32(offset).map(|x| x as u8)
}
