//! thread context
use crate::trap::trap_return;

/// task context
#[repr(C)]
pub struct Context {
    /// return address ( trap_exit_u ) of switch ASM function
    ra: usize,
    /// kernel stack pointer of app
    sp: usize,
    /// saved registers:  s 0..11
    s: [usize; 12],
}

impl Context {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    /// init task context
    pub fn set_cx(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
