mod malloc;
mod pagetable;
mod utils;
mod enrty;
mod address;

pub use address::{KERNEL_SPACE, MapPermission, MemorySet, TRAMPOLINE, KERNEL_STACK_SIZE, USER_STACK_SIZE};
pub use utils::{PhysPageNum, VirtAddr, PG_SIZE};
pub use pagetable::{translated_byte_buffer, translated_refmut, translated_str};


pub fn init() {
    malloc::init_heap();
    malloc::init_frame_allocator();
    address::KERNEL_SPACE.lock().activate();
}
