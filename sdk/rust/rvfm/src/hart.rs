use crate::intrin::hart_id_raw;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hart {
    Hart0,
    Hart1,
    Hart2,
    Hart3,
}

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

    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Hart0 => 0,
            Self::Hart1 => 1,
            Self::Hart2 => 2,
            Self::Hart3 => 3,
        }
    }
}
