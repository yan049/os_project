#![no_std] //  unlink the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(alloc_error_handler)]
use core::arch::global_asm;

extern crate alloc;

extern crate bitflags;


#[macro_use]
mod console;
mod sbi;
mod mem;
pub mod trap;
pub mod loader;
pub mod syscall;
pub mod timer;
pub mod thread;

global_asm!(include_str!("boot.asm"));
global_asm!(include_str!("link_app.asm"));

fn clear_bss() {
    unsafe extern "C" { // this is from linker.ld
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}
#[unsafe(no_mangle)] // tell compiler not to modify this function name, so we can call it from outside(asm)
pub fn main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    mem::init();
    trap::init();
    println!("[kernel] load app!");
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    loader::list_apps();
    println!("[kernel] initproc!");
    thread::add_initproc();
    println!("[kernel] runtasks!");
    thread::run_tasks();
    panic!("Unreachable in rust_main!");
}


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panicked: {}", info);
    sbi::legacy::shutdown()
}
