#pragma once

#include "types.h"

inline static void wfi() {
    __asm__ __volatile__ ("wfi");
}

#define INTERRUPT_HANDLER_RETURN __asm__ __volatile__ ("mret")

inline static u32 hart_id() {
    volatile u32 id = 0;
    __asm__ __volatile__ ("csrr %0, mhartid" : "=r"(id));
    return id;
}

inline static void hart_disable_interrupts() {
    __asm__ __volatile__ ("csrrci zero, mstatus, 0b1000");
}

inline static void hart_enable_interrupts() {
    __asm__ __volatile__ ("csrrsi zero, mstatus, 0b1000");
}

inline static void hart_set_vectored_interrupt_table(const void (* table)()) {
    u32 table_addr = 1 | (u32) table;
    __asm__ __volatile__ ("csrw mtvec, %0" : "=r" (table_addr));
}

inline static void hart_disable_external_interrupt() {
    u32 external_interrupt_bit = 1 << 11;
    __asm__ __volatile__ ("csrrc zero, mie, %0" : "=r" (external_interrupt_bit));
}

inline static void hart_enable_external_interrupt() {
    u32 external_interrupt_bit = 1 << 11;
    __asm__ __volatile__ ("csrrs zero, mie, %0" : "=r" (external_interrupt_bit));
}

inline static void hart_disable_software_interrupt() {
    u32 software_interrupt_bit = 1 << 3;
    __asm__ __volatile__ ("csrrc zero, mie, %0" : "=r" (software_interrupt_bit));
}

inline static void hart_enable_software_interrupt() {
    u32 software_interrupt_bit = 1 << 3;
    __asm__ __volatile__ ("csrrs zero, mie, %0" : "=r" (software_interrupt_bit));
}
