ENTRY(start)

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
        *(.bootstrap.stack)
        *(COMMON)
        *(.bss*)
        /* PROVIDE(stack_bottom = .); */
        /* . += 16K; */
        /* PROVIDE(stack_top = .); */
    }
}
