# shushos
An implementation of a real-time operating system.

## How to build
In order to build the project, you have to have Rust toolchain installed on the system. The following commands will produce a bare binary targeted at x86-64 platform.

```shell
cargo install bootimage
rustup component add rust-src
rustup component add llvm-tools-preview
cargo run
```
