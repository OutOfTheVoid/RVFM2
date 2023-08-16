.extern __stack_ptr
.extern __global_pointer
.extern main

.section .init_text
.global _start
_start:
    .cfi_startproc
    .cfi_undefined ra
    .option push
    .option norelax
    la gp, __global_pointer
    la sp, __stack_ptr
    jal x1, main
_wait_looop:
    wfi
    j _wait_looop
    .option pop
    .cfi_endproc
