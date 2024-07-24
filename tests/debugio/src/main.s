.global start

.section .text
start:
    li x1, 0x80000000
    la x2, hello
    sw x2, 0(x1)
    li x2, 14
    sw x2, 4(x1)
    sw x0, 12(x1)
    sw x0, 16(x1)
    wfi

.section .rodata
hello:
    .asciz "hello, world!\n\0"