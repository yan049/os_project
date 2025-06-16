//! TCB structure
use super::id::ThreadRes;
use super::{KernelStack, ProcessControlBlock, Context, kstack_alloc};
use crate::trap::Frame;
use crate::{mem::PhysPageNum};
use alloc::sync::{Arc, Weak};
use spin::Mutex;

pub struct ThreadControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: Mutex<ThreadControlBlockInner>,
}

impl ThreadControlBlock {
    pub fn get_user_token(&self) -> usize {
        let process = self.process.upgrade().unwrap();
        let inner = process.inner.lock();
        inner.memory_set.token()
    }
}

pub struct ThreadControlBlockInner {
    pub res: Option<ThreadRes>,
    pub trap_frame_ppn: PhysPageNum,
    pub thread_cx: Context,
    pub thread_status: Status,
    pub exit_code: Option<i32>,
}

impl ThreadControlBlockInner {
    pub fn get_trap_frame(&self) -> &'static mut Frame {
        self.trap_frame_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> Status {
        self.thread_status
    }
}

impl ThreadControlBlock {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = ThreadRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_frame_ppn = res.trap_frame_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: Mutex::new(ThreadControlBlockInner {
                res: Some(res),
                trap_frame_ppn,
                thread_cx: Context::set_cx(kstack_top),
                thread_status: Status::Ready,
                exit_code: None,
            }),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Status {
    Ready,
    Running,
    Blocked,
}
