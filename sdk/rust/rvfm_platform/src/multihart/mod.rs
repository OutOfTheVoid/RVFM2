use super::hart::Hart;

use core::arch::global_asm;
use core::ptr::addr_of_mut;

#[cfg(feature = "critical-section")]
pub mod critical_section;

#[cfg(feature = "spinlock")]
pub mod spinlock;

global_asm!(include_str!("trampoline.s"));

extern "C" {
    static mut _mh_trampoline_proc_addr : u32;
    static mut _mh_trampoline_stack_ptr : u32;
    static mut _mh_trampoline_flag      : u32;
    fn _mh_trampoline() -> ();
}

pub fn start_hart(hart: Hart, stack_ptr: &'static mut [u32], proc: extern "C" fn() -> ()) {
    if hart == Hart::current() {
        return;
    }
    unsafe {
        let mh_trampoline_proc_addr_ptr = addr_of_mut!(_mh_trampoline_proc_addr) as *mut u32;
        let mh_trampoline_stack_ptr_ptr = addr_of_mut!(_mh_trampoline_stack_ptr) as *mut u32;
        let mh_trampoline_flag_ptr      = addr_of_mut!(_mh_trampoline_flag     ) as *mut u32;
        mh_trampoline_flag_ptr.write_volatile(0);
        mh_trampoline_proc_addr_ptr.write_volatile(proc as *const () as usize as u32);
        mh_trampoline_stack_ptr_ptr.write_volatile(stack_ptr.as_ptr() as usize as u32);
        hart.start_raw(_mh_trampoline as *const ());
        while mh_trampoline_flag_ptr.read_volatile() == 0 {}
    }
}
