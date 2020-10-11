#![no_std]
#![mp_main]

use core::panic::PanicInfo


/**
 * \brief The entry point of the boot.
 */
#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}


/** 
 * \brief The function called on panic.
 */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
