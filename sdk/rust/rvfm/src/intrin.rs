#![allow(unused)]

use core::arch::asm;

pub fn wfi() {
    unsafe {
        asm!(
            "wfi"
        );
    }
}

pub(crate) unsafe fn hart_id_raw() -> u32 {
    let mut id: u32;
    asm!("csrr {}, mhartid", out(reg) id);
    id
}

pub fn hart_disable_interrupts() {
    unsafe {
        asm!("csrrci zero, mstatus, 0b1000");
    }
}

pub fn hart_enable_interrupts() {
    unsafe {
        asm!("csrrsi zero, mstatus, 0b1000");
    }
}

pub fn hart_set_vectored_interrupt_table(table: extern "C" fn() -> ()) {
    unsafe {
        let vector = core::mem::transmute::<_, u32>(table) | 1;
        asm!("csrw mtvec, {}", in(reg) vector);
    }
}

pub fn hart_disable_external_interrupt() {
    let external_interrupt_bit = 1 << 11;
    unsafe {
        asm!("csrrc zero, mie, {}", in(reg) external_interrupt_bit);
    }
}

pub fn hart_enable_external_interrupt() {
    let external_interrupt_bit = 1 << 11;
    unsafe {
        asm!("csrrs zero, mie, {}", in(reg) external_interrupt_bit);
    }
}

pub fn hart_disable_software_interrupt() {
    let software_interrupt_bit = 1 << 3;
    unsafe {
        asm!("csrrc zero, mie, {}", in(reg) software_interrupt_bit);
    }
}

pub fn hart_enable_software_interrupt() {
    let software_interrupt_bit = 1 << 3;
    unsafe {
        asm!("csrrs zero, mie, {}", in(reg) software_interrupt_bit);
    }
}


