# shushos
An implementation of a real-time operating system.

## How to build
In order to build the project, you have to have Rust toolchain installed on the system. The following commands will produce a bare binary targeted at x86-64 platform.

```shell
sudo apt-get update
sudo apt-get install qemu qemu-system qemu-kvm
cargo install bootimage
rustup component add rust-src
rustup component add llvm-tools-preview
cargo run
```

## Run tests

To run tests, simply execute the following:
```shell
cargo test
```

## Features

### print! and println! macros
These macros are defined in the crate `vga_buffer.rs`, which also holds the VgaBuffer, Buffer, and Writer structs with safe interfaces. Printing is done by using a `"static mut" WRITER` object, which, upon writing into the Buffer and VgaBuffer, locks a spinlock mutex and prevents cpu from sending interrupts to prevent undefined behaviour caused by race conditions. Since the Rust's implementation of `static mut` struct is flawed, we emulate this behaviour by making `WRITER` a `lazy_static` object, and by locking a mutex when accessing it.

### Scrolling
Using arrow keys (up/down) you can scroll the contents of the buffer. The implementation is inside `interrupts.rs` and `vga_buffer.rs`. During a keyboard interrupt, we execute a function that changes the position of the VgaBuffer inside Buffer and updates the screen view.

### Panic
Panic situations are handled by the function defined in `main.rs`, `panic`.

### Memory mapping
Using `bootimage`'s initialized memory map, we introduce a `FrameAllocator` which, using the physical memory offset, returns usable frames specified in the memory map. The implementation is in `memory.rs`, we test virtual memory mapping in function `basic_tests` inside `main.rs`.

### Keyboard
ps/2 keyboard interrupts are handled by functions inside `interrupts.rs`. The scan codes are transformed into characters via crate `pc-keyboard`'s functions.
