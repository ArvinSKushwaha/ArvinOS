#![no_std]

pub mod output;

#[panic_handler]
#[no_mangle]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        prepare_background();

        if let Some(msg) = info.location() {
            let mut text = [b' '; 80 * 25];
            let file = msg.line().to_be_bytes();
            text[0..file.len()].copy_from_slice(&file);

            for (i, b) in text.iter().enumerate() {
                (0xB8000 as *mut u8)
                    .offset((i as isize) << 1)
                    .write_volatile(*b);
            }
        } else {
            (0xB8000 as *mut u8).write_volatile(b'a')
        }
    }

    halt_loop();
}

/// This method acts as a safe wrapper to the commonly used assembly sequence:
/// ```asm
/// cli
/// 1: hlt
/// jmp 1,
/// ```
#[no_mangle]
pub fn halt_loop() -> ! {
    // SAFETY: This is safe as it freezes the system.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
        loop {
            core::arch::asm!("hlt", options(nomem, nostack, noreturn));
        }
    }
}

pub unsafe fn prepare_background() {
    let x = 0xB8000 as *mut u8;

    for row in 0..25 {
        for col in 0..80 {
            x.offset((row * 80 + col) << 1 + 1).write_volatile(0x0f);
            x.offset((row * 80 + col) << 1).write_volatile(b' ');
        }
    }
}
