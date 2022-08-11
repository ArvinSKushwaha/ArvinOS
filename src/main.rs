#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

global_asm! {r#"
    .code32
    .pushsection .bss
    stack_bottom:
        .skip 16384
    stack_top:
    .popsection

    .pushsection .multiboot_header
    header_start:
        .long 0xe85250d6
        .long 0
        .long header_end - header_start
        // checksum
        .long 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))
        
        // optional multiboot tags

        // required end tag
        .word 0
        .word 0
        .long 8
    header_end:
    .popsection

    .pushsection .text
    .global start
    .type start, @function
    start:
        // Set stack
        lea esp, [stack_top]
        call check_multiboot
        call check_cpuid_supported
        call check_has_long_mode
        
        // Prints OK to the screen
        mov dword ptr [0xb8000], 0x2f4b2f4f
        hlt

    error:
        mov dword ptr [0xb8000], 0x4f524f45
        mov dword ptr [0xb8004], 0x4f3a4f52
        mov dword ptr [0xb8008], 0x4f204f20
        mov byte ptr [0xb800a], al
        hlt

    check_multiboot:
        cmp eax, 0x36d76289
        jne .no_multiboot
        ret
    
    .no_multiboot:
        mov al, '0'
        jmp error

    .no_cpuid:
        mov al, '1'
        jmp error

    .no_long_mode:
        mov al, '2'
        jmp error

    check_cpuid_supported:
        pushfd
        pushfd
        xor dword ptr [esp], 1 << 21
        popfd
        pushfd
        pop eax
        pop ecx
        cmp eax, ecx
        je .no_cpuid
        ret

    check_has_long_mode:
        // Check if cpuid supports extended functions
        mov eax, 0x80000000
        cpuid
        cmp eax, 0x80000001
        jb .no_long_mode
        //
        mov eax, 0x80000001
        cpuid
        test edx, 1 << 29
        jz .no_long_mode
    .popsection
"#}

#[panic_handler]
pub extern "C" fn panic(_: &PanicInfo) -> ! {
    loop {}
}
