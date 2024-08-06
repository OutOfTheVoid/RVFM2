MEMORY
{
    RAM (rwx) : ORIGIN = 0x00000000, LENGTH = 0x08000000
    MMIO (rw) : ORIGIN = 0x80000000, LENGTH = 0x40000000
    ROM (rx)  : ORIGIN = 0xF8000000, LENGTH = 0x08000000
}

ENTRY(_start)

SECTIONS
{
    .text :
    {
        _text = .;
        KEEP(*(.init_text))
        *(.text*)
        *(.rodata*)
        *(.srodata*)
        _etext = .;
    } > ROM

    .data : 
    {
        _data = .;
        *(.data*)
        _edata = .;
    } > RAM AT >ROM

    .bss :
    {
        _bss = .;
        *(.bss*)
        *(COMMON)       
        _ebss = .;
    } > RAM

    PROVIDE( __stack_ptr      = ORIGIN(RAM) + LENGTH(RAM) );
    PROVIDE( __global_pointer = 0x800                     );
}
