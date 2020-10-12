# shushos
An implementation of a real-time operating system.

## How to build
In order to build the project, you have to have Rust toolchain installed on the system. The following commands will produce a bare binary targeted at x86-64 platform.

```shell
rustup target add thumbv7em-none-eabihf
cargo build --target thumbv7em-none-eabihf
```
