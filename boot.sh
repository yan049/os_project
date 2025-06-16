qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios fw_jump.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/release/os
