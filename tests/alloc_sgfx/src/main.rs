#![no_std]
#![no_main]

use rvfm_platform::intrin::*;
use rvfm_platform::debug::*;
use rvfm_platform::multihart::spinlock::RawSpinLock;

use core::arch::global_asm;

global_asm!(include_str!("init.s"));

use alloc::boxed::Box;

extern crate alloc;
use talc::*;

static mut ARENA: [u8; 10000] = [0; 10000];

#[global_allocator]
static ALLOCATOR: Talck<RawSpinLock, ErrOnOom> = Talc::new(ErrOnOom).lock();

#[no_mangle]
extern "C" fn main() {
    println!("Hello, world!");

    unsafe { ALLOCATOR.lock().claim(Span::from(&mut ARENA)) };

    let test = Box::new(3);

    println!("test: {:?}", test);

    loop {
        wfi();
    }
}
