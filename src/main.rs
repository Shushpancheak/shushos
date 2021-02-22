#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(shushos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use shushos::println;
use shushos::print;
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("shushos initialization and self-testing starting...");
    shushos::init();

    basic_tests(boot_info);

    #[cfg(test)]
    test_main();

    println!("Initialization complete.");
    println!();
    println!("Welcome to shushos!");
    println!();
    println!("Use up/down arrow keys to scroll through the shell.");
    println!("Hit Enter to come back to the cursor.");
    println!();
    print!(">>");

    shushos::hlt_loop(); 
}

fn basic_tests(boot_info: &'static BootInfo) {
    use shushos::memory;
    use x86_64::{structures::paging::Translate, VirtAddr};

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    shushos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    shushos::test_panic_handler(info)
}
