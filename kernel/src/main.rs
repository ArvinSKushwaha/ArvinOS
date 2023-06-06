#![no_std]
#![no_main]

#[panic_handler]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
