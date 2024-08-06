#![allow(unused)]

use crate::hart::Hart;

const INTERRUPT_PEND_BASE: u32 = 0x80030008;
const INTERRUPT_TARGET_BASE: u32 = 0x80030004;
const INTERRUPT_ENABLE_BASE: u32 = 0x80030000;

const PENDING_INTERRUPT_BASE: u32 = 0x80030FC0;
const IHI_ENABLE_BASE: u32 = 0x80030FD0;
const IHI_CLEAR_BASE: u32 = 0x80030FE0;
const IHI_TRIGGER_BASE: u32 = 0x80030FF0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Interrupt {
    GpuInterrupt = 0,
    PresentInterrupt = 1,
    VSyncInterrupt = 2,
    SpuInterrupt = 3,
}

pub enum PendingInterrupt {
    Peripheral(Interrupt),
    InterHart,
}

impl Interrupt {
    pub fn set_target(self, hart: Hart) {
        let address = INTERRUPT_TARGET_BASE + (self as u32) << 4;
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, hart.to_u32()); }
    }

    pub fn poll(self) -> bool {
        let address = INTERRUPT_PEND_BASE + (self as u32) << 4;
        unsafe { core::ptr::read_volatile::<u32>(address as usize as *const u32) != 0 }
    }

    pub fn clear(self) {
        let address = INTERRUPT_PEND_BASE + (self as u32) << 4;
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 1); }
    }

    pub fn enable(self) {
        let address = INTERRUPT_ENABLE_BASE + (self as u32) << 4;
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 1); }
    }

    pub fn disable(self) {
        let address = INTERRUPT_ENABLE_BASE + (self as u32) << 4;
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 0); }
    }

    pub fn get_pending() -> Option<PendingInterrupt> {
        let hart = Hart::current();
        let address = PENDING_INTERRUPT_BASE + hart.to_u32() << 4;
        let value = unsafe { core::ptr::read_volatile::<u32>(address as usize as *const u32) };
        Some(match value {
            0 => PendingInterrupt::Peripheral(Self::GpuInterrupt),
            1 => PendingInterrupt::Peripheral(Self::PresentInterrupt),
            2 => PendingInterrupt::Peripheral(Self::VSyncInterrupt),
            3 => PendingInterrupt::Peripheral(Self::SpuInterrupt),
            0xFF => PendingInterrupt::InterHart,
            _ => return None
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InterHartInterrupt(pub Hart);

impl InterHartInterrupt {
    fn local() -> Self {
        Self(Hart::current())
    }

    fn enable(self) {
        let address = IHI_ENABLE_BASE + (self.0.to_u32() << 2);
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 1); }
    }

    fn disable(self) {
        let address = IHI_ENABLE_BASE + (self.0.to_u32() << 2);
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 0); }
    }

    fn send(self) {
        let address = IHI_TRIGGER_BASE + (self.0.to_u32() << 2);
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 1); }
    }

    fn clear(self) {
        let address = IHI_CLEAR_BASE + (self.0.to_u32() << 2);
        unsafe { core::ptr::write_volatile(address as usize as *mut u32, 1); }
    }

    fn poll(self) -> bool {
        let address = IHI_CLEAR_BASE + (self.0.to_u32() << 2);
        unsafe { core::ptr::read_volatile::<u32>(address as usize as *const u32) != 0 }
    }
}

