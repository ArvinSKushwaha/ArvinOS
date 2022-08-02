/// This method acts as a safe wrapper to the commonly used assembly sequence:
/// ```asm
/// cli
/// 1: hlt
/// jmp 1,
/// ```
pub fn halt_loop() -> ! {
    // SAFETY: This is safe as it freezes the system.
    unsafe {
        core::arch::asm!("cli");
        loop {
            core::arch::asm!("hlt");
        }
    }
}
