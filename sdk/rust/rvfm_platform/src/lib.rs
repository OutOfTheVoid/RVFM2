#![no_std]

pub mod gpu;
pub mod intrin;
pub mod interrupt;
pub mod hart;
pub mod command_list;
pub mod input;
pub mod spu;
#[cfg(feature = "multihart")]
pub mod multihart;

#[macro_use]
pub mod debug;
use debug::*;

use core::panic::PanicInfo;

use crate as rvfm_platform;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    let loc = panic_info.location().unwrap();
    println!("PANIC: {}:{}:{}:", loc.file(), loc.line(), loc.column());
    println!("{:?}", panic_info.message());
    
    loop {
        rvfm_platform::intrin::wfi();
    }
}
