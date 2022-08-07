/// This method acts as a safe wrapper to the commonly used assembly sequence:
/// ```asm
/// cli
/// 1: hlt
/// jmp 1,
/// ```
use core::arch::asm;

#[no_mangle]
pub fn halt_loop() -> ! {
    unsafe {
        asm! {
            "cli",
            "2: hlt",
            "jmp 2b",
            options(nomem, noreturn)
        }
    }
}
