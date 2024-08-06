#![no_std]
#![no_main]

use core::arch::global_asm;

global_asm!(include_str!("init.s"));

use rvfm_platform::intrin::*;
use rvfm_platform::hart::*;
use rvfm_platform::multihart::*;
use rvfm_platform::debug;

static mut AP_STACK: [u32; 0x400] = [0u32; 0x400];

#[no_mangle]
extern "C" fn main() {

    debug::write_str("starting hart 1...\n");
    debug::flush();

    start_hart(Hart::Hart1, unsafe { &mut AP_STACK[..] }, ap_main);

    debug::write_str("hart 1 started\n");
    debug::flush();
    
    loop {
        wfi();
    }
}

extern "C" fn ap_main() {
    debug::write_str("ap_main()\n");
    debug::flush();

    loop {
        wfi();
    }
}

