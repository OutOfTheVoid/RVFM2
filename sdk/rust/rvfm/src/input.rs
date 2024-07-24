#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Button {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    A = 4,
    B = 5,
    X = 6,
    Y = 7,
    Start = 8,
    Select = 9,
}

impl Button {
    pub fn get(self) -> bool {
        let address = 0x8005_0000u32 + ((self as u32) << 2);
        unsafe { (address as usize as *mut () as *mut u32).read_volatile() != 0 }
    }
}
