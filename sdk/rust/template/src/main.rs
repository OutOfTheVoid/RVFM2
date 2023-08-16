#![no_std]
#![no_main]

use core::arch::global_asm;

global_asm!(include_str!("init.s"));

#[no_mangle]
extern "C" fn main() {
    rvfm::debug::println!("hello, world!");
    loop {}
}
