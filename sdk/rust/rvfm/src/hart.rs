use crate::intrin::hart_id_raw;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hart {
    Hart0,
    Hart1,
    Hart2,
    Hart3,
}

const HART_START_TRIGGER_BASE: u32 = 0x8002_0000;

impl Hart {
    pub fn current() -> Self {
        let id_raw = unsafe { hart_id_raw() };
        match id_raw {
            0 => Self::Hart0,
            1 => Self::Hart1,
            2 => Self::Hart2,
            3 => Self::Hart3,
            _ => unreachable!()
        }
    }

    pub fn to_u32(self) -> u32 {
        match self {
            Self::Hart0 => 0,
            Self::Hart1 => 1,
            Self::Hart2 => 2,
            Self::Hart3 => 3,
        }
    }

    pub unsafe fn start_raw(self, start_address: *const ()) {
        let address = HART_START_TRIGGER_BASE + (self.to_u32() << 2);
        unsafe { (address as usize as *mut u32).write_volatile(start_address as usize as u32); }
    }
}
