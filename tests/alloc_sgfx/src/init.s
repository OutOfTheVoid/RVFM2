.extern __stack_ptr
.extern __global_pointer
.extern main

.section .init_text
.global _start
_start:
    
    .cfi_startproc
    
    .cfi_undefined ra
    .cfi_undefined gp

    addi x1, zero, 0
    addi x2, zero, 0
    addi x3, zero, 0
    addi x4, zero, 0
    addi x5, zero, 0
    addi x6, zero, 0
    addi x7, zero, 0
    addi x8, zero, 0
    addi x9, zero, 0
    addi x10, zero, 0
    addi x11, zero, 0
    addi x12, zero, 0
    addi x13, zero, 0
    addi x14, zero, 0
    addi x15, zero, 0
    addi x16, zero, 0
    addi x17, zero, 0
    addi x18, zero, 0
    addi x19, zero, 0
    addi x20, zero, 0
    addi x21, zero, 0
    addi x22, zero, 0
    addi x23, zero, 0
    addi x24, zero, 0
    addi x25, zero, 0
    addi x26, zero, 0
    addi x27, zero, 0
    addi x28, zero, 0
    addi x29, zero, 0
    addi x30, zero, 0
    addi x31, zero, 0

    .option push
    .option norelax
    
    lui gp, %hi(__global_pointer)
    addi gp, gp, %lo(__global_pointer)

    lui sp, %hi(__stack_ptr)
    addi sp, sp, %lo(__stack_ptr)

    .option pop

    lui x1, %hi(main)
    addi x1, x1, %lo(main)

    jalr x1, x1

wfi_loop:
    wfi
    j wfi_loop
    
    .cfi_endproc
    