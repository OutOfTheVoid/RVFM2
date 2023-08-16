#pragma once

#include "types.h"
#include "hart.h"

#define INTERRUPT_HANDLER __attribute__((interrupt))

typedef enum {
    GpuInterrupt = 0,
    PresentInterrupt = 1,
    VSyncInterrupt = 2,
} Interrupt;

static volatile u32 * const pending_interrupts = (volatile u32 * const) 0x80030FC0;
static volatile u32 * const ihi_enables = (volatile u32 * const) 0x80030FD0;
static volatile u32 * const ihi_clears = (volatile u32 * const) 0x80030FE0;
static volatile u32 * const ihi_triggers = (volatile u32 * const) 0x80030FF0;

inline static void set_interrupt_target(Interrupt interrupt, Hart hart) {
    u32 address = 0x80030000 + (interrupt << 4) + 4;
    *((volatile u32 *) address) = hart;
}

inline static bool poll_interrupt(Interrupt interrupt) {
    u32 address = 0x80030000 + (interrupt << 4) + 8;
    return *((volatile u32 *) address) != 0;
}

inline static void clear_interrupt(Interrupt interrupt) {
    u32 address = 0x80030000 + (interrupt << 4) + 8;
    *((volatile u32 *) address) = 1;
}

inline static void enable_interrupt(Interrupt interrupt) {
    u32 address = 0x80030000 + (interrupt << 4) + 0;
    *((volatile u32 *) address) = 1;
}

inline static void disable_interrupt(Interrupt interrupt) {
    u32 address = 0x80030000 + (interrupt << 4) + 0;
    *((volatile u32 *) address) = 0;
}

inline static void enable_ihi(Hart hart) {
    ihi_enables[hart] = 1;
}

inline static void disable_ihi(Hart hart) {
    ihi_enables[hart] = 0;
}

inline static void send_ihi(Hart hart) {
    ihi_triggers[hart] = 1;
}

inline static void clear_ihi(Hart hart) {
    ihi_clears[hart] = 1;
}

inline static bool poll_ihi(Hart hart) {
    return ihi_clears[hart] != 0;
}

inline static bool get_pending_interrupt(Hart hart, Interrupt * interrupt) {
    u32 pending = pending_interrupts[hart];
    if (pending == 0xFFFFFFFF) {
        return false;
    }
    *interrupt = (Interrupt) pending;
    return true;
}

#define DECLARE_VECTORED_INTERRUPT_TABLE(handler_table_name, software_interrupt, timer_interrupt, external_interrupt) \
__attribute__((naked)) void handler_table_name() { \
    __asm__ volatile ( \
        "    mret\n" \
        "    mret\n" \
        "    mret\n" \
        "    j %0\n" \
        "    mret\n" \
        "    mret\n" \
        "    mret\n" \
        "    j %1\n" \
        "    mret\n" \
        "    mret\n" \
        "    mret\n" \
        "    j %2\n" :: \
        "i"(software_interrupt), \
        "i"(timer_interrupt   ), \
        "i"(external_interrupt) \
    ); \
}
