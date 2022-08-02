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
    .set MAGIC, 0xE85250D6
    .set ARCHITECTURE, 0
    .set HEADERLENGTH, 16
    .set CHECKSUM, 0x17ADAF1A
    .set FLAGS, 0

    .section .multiboot
    .align 8
    .long MAGIC
    .long ARCHITECTURE
    .long HEADERLENGTH
    .long CHECKSUM
    .long FLAGS

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
pub fn kernel_main() {
}

/// # Safety
/// This method is meant to be loaded by GRUB, not for use to attempt
/// to use.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Check if bootloader is multiboot compliant
    let mut eax: u32;
    let mut _ebx: *const Multiboot;
    core::arch::asm!("mov {x}, eax", x = out(reg) eax);
    assert_eq!(eax, 0x36d76289);


    // Set up stack
    core::arch::asm!("mov $stack_top, esp");

    kernel_main();

    intrinsics::halt_loop();
}

global_asm! {
    ".size _start, . - start"
}
