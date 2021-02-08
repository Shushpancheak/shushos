#![no_std]
#![no_main]

use core::panic::PanicInfo;


// The entry point of the boot.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}


// The function called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}