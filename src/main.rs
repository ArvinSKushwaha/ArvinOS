#![no_std]
#![no_main]
// #![feature(const_mut_refs)]
#![allow(dead_code)]

use core::{arch::global_asm, panic::PanicInfo};

use crate::multiboot::Multiboot;

mod intrinsics;
mod output;
mod multiboot;

global_asm! {r#"
    .set ALIGN, 1<<0
    .set MEMINFO, 1<<1
    .set VIDINFO, 1<<2
    .set FLAGS, ALIGN | MEMINFO | VIDINFO
    .set MAGIC, 0x1BADB002
    .set CHECKSUM, -(MAGIC + FLAGS)

    .section .multiboot
    .align 4
    .long MAGIC
    .long FLAGS
    .long CHECKSUM

    .section .bss
    .align 16
    stack_bottom:
    .skip 1048576 # 1MiB
    stack_top:
"#}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    intrinsics::halt_loop();
}

/// This method is the portal through which our operating system is executed.
/// It gets called by [`_start`], which sets up our stack and halt loop.
pub fn kernel_main(multiboot_data: Multiboot) {
}

/// # Safety
/// This method is meant to be loaded by GRUB, not for use to attempt
/// to use.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Check if bootloader is multiboot compliant
    let mut eax: u32;
    let mut _ebx: *const Multiboot;
    core::arch::asm!("mov {x}, eax", "mov {y}, ebx", x = out(reg) eax, y = out(reg) _ebx);
    assert_eq!(eax, 0x2BADB002);

    let multiboot_data = *_ebx;

    // Set up stack
    core::arch::asm!("mov $stack_top, esp");

    kernel_main(multiboot_data);

    intrinsics::halt_loop();
}

global_asm! {
    ".size _start, . - start"
}
