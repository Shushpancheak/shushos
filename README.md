# shushos
An implementation of a real-time operating system.

## How to build
In order to build the project, you have to have Rust toolchain installed on the system. The following commands will produce a bare binary targeted at x86-64 platform.

```shell
rustup component add rust-src
cargo build --target shushos-target.json
```
