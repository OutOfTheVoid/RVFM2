/*
static volatile const u8** const debug_message_ptr   = (volatile const u8** const) 0x80000000;
static volatile       u32* const debug_length        = (volatile       u32* const) 0x80000004;
static volatile       u32* const debug_status        = (volatile       u32* const) 0x80000008;
static volatile       u32* const debug_print_trigger = (volatile       u32* const) 0x8000000C;
*/

const DEBUG_MESSAGE_PTR    : *mut *const u8  = 0x8000_0000_u32 as _;
const DEBUG_MESSAGE_LENGTH : *mut        u32 = 0x8000_0004_u32 as _;
const DEBUG_STATUS         : *mut        u32 = 0x8000_0008_u32 as _;
const DEBUG_WRITE_TRIGGER  : *mut        u32 = 0x8000_000C_u32 as _;
const DEBUG_FLUSH_TRIGGER  : *mut        u32 = 0x8000_0010_u32 as _;

use core::fmt::Write;

pub fn write_str(s: &str) {
    unsafe {
        DEBUG_MESSAGE_LENGTH.write_volatile(s.len() as u32);
        DEBUG_MESSAGE_PTR.write_volatile(s.as_ptr());
        DEBUG_WRITE_TRIGGER.write_volatile(1);
    }
}

pub fn flush() {
    unsafe {
        DEBUG_FLUSH_TRIGGER.write_volatile(1);
    }
}

pub enum DebugError {
    MemoryAccess,
    BadUtf,
    Unknown,
}

pub fn status() -> Result<(), DebugError> {
    unsafe {
        match DEBUG_STATUS.read_volatile() {
            0 => Ok(()),
            1 => Err(DebugError::MemoryAccess),
            2 => Err(DebugError::BadUtf),
            _ => Err(DebugError::Unknown),
        }
    }
}

pub fn clear_status() {
    unsafe {
        DEBUG_STATUS.write_volatile(0);
    }
}

pub struct DebugWriter;

impl DebugWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write_str(s);
        match status() {
            Ok(()) => Ok(()),
            Err(_) => Err(core::fmt::Error)
        }
    }
}

#[macro_export]
macro_rules! __println__ {
    () => {
        {
            use core::fmt::Write;
            use rvfm_platform;
            let _ = rvfm_platform::debug::write_str("\n");
            rvfm_platform::debug::flush();
        }
    };
    ($($arg:tt)*) => {
        {
            use core::fmt::Write;
            use rvfm_platform;
            let mut writer = rvfm_platform::debug::DebugWriter;
            let _ = write!(&mut writer, $($arg)*);
            rvfm_platform::debug::flush();
        }
    };
}

pub use crate::__println__ as println;
