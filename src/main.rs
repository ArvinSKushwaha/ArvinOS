#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

mod intrinsics;

global_asm! {r#"
    .set ALIGN, 1<<0
    .set MEMINFO, 1<<1
    .set FLAGS, ALIGN | MEMINFO
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
pub fn panic(_info: &PanicInfo) -> ! {
    intrinsics::halt_loop();
}

/// # Safety
/// This method is meant to be loaded by GRUB, not for use to attempt
/// to use.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::asm!("mov $stack_top, esp");

    // Everything else

    intrinsics::halt_loop();
}

global_asm! {
    ".size _start, . - start"
}
