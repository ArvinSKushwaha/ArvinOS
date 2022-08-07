#![no_std]
#![no_main]
#![feature(used_with_arg)]
#![feature(arbitrary_enum_discriminant)]

use core::{arch::global_asm, panic::PanicInfo};

mod intrinsics;
mod multiboot;
mod output;

global_asm! {r#"
    .pushsection .bss
    .align 16
    stack_bottom:
    .skip 1048576 # 1MiB
    stack_top:
    .popsection

    .pushsection .text
    .global _start
    .type _start, @function
    _start:
        mov stack_top, esp
        cmp eax, 0x2badb002
        jne .inf_loop
        # prepare arguments
        sub esp, 12 # maintains 16 byte alignment for stack before calling
        push ebx
        call kernel_main
        pop ebx
        add esp, 12
    .inf_loop:
        cli
    .l00p:
        hlt
        jmp .l00p
        ret
    .popsection
"#}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        let mut ptr: *mut u16 = 0xb8000 as *mut u16;
        let str = "Panic!!!";
        for c in str.chars() {
            ptr.write_volatile(0x0B00 | c as u16);
            ptr = ptr.offset(1);
        }
    }

    intrinsics::halt_loop();
}

#[no_mangle]
extern "C" fn kernel_main(multiboot_info: *const multiboot::Info) -> ! {
    let _multiboot_info = unsafe { multiboot_info.read() };

    intrinsics::halt_loop();
}
