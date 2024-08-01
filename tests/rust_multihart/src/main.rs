#![no_std]
#![no_main]

use core::arch::global_asm;

global_asm!(include_str!("init.s"));

use rvfm::intrin::*;
use rvfm::hart::*;
use rvfm::multihart::*;

static mut AP_STACK: [u32; 0x400] = [0u32; 0x400];

#[no_mangle]
extern "C" fn main() {

    rvfm::debug::write_str("starting hart 1...\n");
    rvfm::debug::flush();

    start_hart(Hart::Hart1, unsafe { &mut AP_STACK[..] }, ap_main);

    rvfm::debug::write_str("hart 1 started\n");
    rvfm::debug::flush();
    
    loop {
        wfi();
    }
}

extern "C" fn ap_main() {
    rvfm::debug::write_str("ap_main()\n");
    rvfm::debug::flush();

    loop {
        wfi();
    }
}

