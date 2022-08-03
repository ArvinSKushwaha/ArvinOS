#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(used_with_arg)]
#![feature(const_size_of_val)]

use core::{arch::global_asm, panic::PanicInfo};

use multiboot2::BootInformation;

mod intrinsics;
mod multiboot;
mod output;

global_asm! {r#"
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
pub fn kernel_main(boot_info: BootInformation) {
    let framebuffer_info = boot_info.framebuffer_tag();

    if let Some(framebuffer_info) = framebuffer_info {
        output::setup_visuals(&framebuffer_info);
        println!("Hello, world!");
    } else {
        output::setup_headless();
    }
    println!("Hello, world!");
}

/// # Safety
/// This method is meant to be loaded by GRUB, not for use to attempt
/// to use.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Check if bootloader is multiboot compliant
    let mut eax: u32;
    let mut ebx: u32;

    core::arch::asm!(
        "mov {x}, eax",
        "mov {y}, ebx",
        x = out(reg) eax,
        y = out(reg) ebx
    );

    assert_eq!(eax, multiboot2::MULTIBOOT2_BOOTLOADER_MAGIC);
    let boot_info = multiboot2::load(ebx as usize).unwrap();

    // Set up stack
    core::arch::asm!("mov $stack_top, esp");

    kernel_main(boot_info);

    intrinsics::halt_loop();
}
