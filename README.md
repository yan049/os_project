## Prerequistes
### Rust
we use nightly Rust, and some tools
``` shell
rustup install nightly
rustup default nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add llvm-tools-preview
rustup component add rust-src
```
### QEMU
see https://pdos.csail.mit.edu/6.1810/2024/tools.html
## My enviroment
- Operating System: archlinux
- CPU Architecture: x86_64
- Toolchain:
  - Rust: 1.89.0-nightly 
  - Cargo: 1.89.0-nightly
- Emulator: QEMU 9.2.0
## Run

``` shell
cd user/
./build.sh
cd ..
cargo build --release
./boot.sh
```
## clean

``` shell
cargo clean
cd user/
cargo clean
```
## code structure
```
.
├── boot.sh
├── build.rs (build link_app.asm)
├── Cargo.lock
├── Cargo.toml
├── debug.sh
├── fw_jump.bin (opensbi firmware, maybe compile new one if needed https://github.com/riscv-software-src/opensbi)
├── gdb.sh
├── README.md
├── src
│   ├── boot.asm (set kernel enviroment)
│   ├── console.rs (print)
│   ├── link_app.asm (app memory location, built by build.rs)
│   ├── linker.ld (kernel memory layout)
│   ├── loader.rs (load app)
│   ├── main.rs
│   ├── mem
│   │   ├── address.rs (implement address space)
│   │   ├── enrty.rs (pagetable entry)
│   │   ├── malloc.rs (memory allocator)
│   │   ├── mod.rs
│   │   ├── pagetable.rs (page table structure)
│   │   └── utils.rs (various data structure about pagetable)
│   ├── sbi.rs (opensbi interface)
│   ├── syscall.rs (syscall implementation and handler)
│   ├── thread
│   │   ├── context.rs (thread context)
│   │   ├── id.rs (id allocator)
│   │   ├── manager.rs (TCB manager)
│   │   ├── mod.rs
│   │   ├── processor.rs (scheduler)
│   │   ├── process.rs (PCB structure)
│   │   ├── switch.asm
│   │   ├── switch.rs (context switch)
│   │   └── thread.rs (TCB structure)
│   ├── timer.rs (timer for interupt)
│   ├── trap.asm
│   └── trap.rs  (trap frame and handler)
└── user
    ├── Cargo.lock
    ├── Cargo.toml
    ├── Makefile
    └── src
        ├── bin (shell & initproc & test apps)
        │   ├── forkexec.rs
        │   ├── hello_world.rs
        │   ├── initproc.rs
        │   ├── simple_fork_test.rs
        │   ├── threads.rs
        │   └── user_shell.rs
        ├── console.rs
        ├── lib.rs
        ├── linker.ld
        └── syscall.rs
```
