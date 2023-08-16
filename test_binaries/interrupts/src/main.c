#include "dbg.h"
#include "intrin.h"
#include "interrupt.h"
#include "hart.h"

void INTERRUPT_HANDLER dummy_interrupt();
void INTERRUPT_HANDLER software_interrupt();

DECLARE_VECTORED_INTERRUPT_TABLE(interrupt_table, software_interrupt, dummy_interrupt, dummy_interrupt);

Hart self;

void main() {
    self = this_hart();

    hart_set_vectored_interrupt_table(interrupt_table);

    enable_ihi(self);
    hart_enable_software_interrupt();
    hart_enable_interrupts();

    send_ihi(self);

    debug_print("after software interrupt");

    while (1) {
        wfi();
    }
}

void INTERRUPT_HANDLER dummy_interrupt() {}

void INTERRUPT_HANDLER software_interrupt() {
    debug_print("software interrupt");
    clear_ihi(self);
}
