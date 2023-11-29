#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(used_with_arg, exclusive_wrapper, lazy_cell)]

use core::arch::{global_asm, asm};

pub use intrinsics::panic_handler;

use multiboot::MultibootInfo;

global_asm! {r#"
    .section .bss
    .align 16
    stack_bottom:
    .skip 1048576 # 1MiB
    stack_top:
"#}

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let eax: u32;
    let ebx: u32;
    asm! {
        "mov eax, {eax:e}",
        "mov ebx, {ebx:e}",
        eax = out(reg) eax,
        ebx = out(reg) ebx,
        options(readonly),
    };

    // assert_eq!(eax, 0x2BADB002);
    // let boot_info = MultibootInfo::load(ebx);

    // Set up stack
    core::arch::asm!("mov $stack_top, esp");
    let _ = intrinsics::output::EGA_WRITER.write_str("hello, world!");

    intrinsics::halt_loop();
}
