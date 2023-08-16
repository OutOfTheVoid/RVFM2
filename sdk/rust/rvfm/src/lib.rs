#![no_std]

pub mod gpu;
pub mod intrin;
pub mod interrupt;
pub mod hart;

#[macro_use]
pub mod debug;

use core::panic::PanicInfo;

use debug::DebugWriter;

#[panic_handler]
fn panic_handler(panic_info: &PanicInfo) -> ! {
    loop {
        debug::flush();
        let hart = hart::Hart::current().to_u32();
        let mut writer = DebugWriter;
        use core::fmt::Write;
        if let Some(location) = &panic_info.location() {
            let _ = write!(&mut writer, "hart {} panic!\n@ {}:{}:{}", hart, location.file(), location.line(), location.column());
        } else {
            let _ = write!(&mut writer, "hart {} panic!", hart);
        }
        debug::flush();
        intrin::wfi();
    }
}
