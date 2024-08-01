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

use crate as rvfm;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    debug::flush();
    let hart = hart::Hart::current().to_u32();
    if let Some(location) = &panic_info.location() {
        println!("hart {} panic!\n@ {}:{}:{}", hart, location.file(), location.line(), location.column());
    } else {
        println!("hart {} panic!", hart);
    }
    rvfm::debug::flush();
    loop {
        rvfm::intrin::wfi();
    }
}
