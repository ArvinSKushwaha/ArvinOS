ENTRY(_start)

SECTIONS {
    . = 1M;
    .text ALIGN(4096) : {
        *(.multiboot)
        *(.text*)
    }
    .rodata ALIGN(4096) : {
        *(.rodata*)
    }
    .data ALIGN(4096) : {
        *(.data*)
    }
    .bss ALIGN(4096) : {
        *(COMMON)
        *(.bss*)
    }

    /* Add other sections here */
}
