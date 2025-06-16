use super::Context;
use core::arch::global_asm;

global_asm!(include_str!("switch.asm"));

unsafe extern "C" {
    /// Switch to the next context and save the current context
    pub fn switch(current_cx: *mut Context, next_cx: *const Context);
}
