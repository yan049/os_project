//! scheduler
use super::switch;
use super::{Context, ProcessControlBlock, ThreadControlBlock};
use super::{Status, fetch_task};
use spin::Mutex;
use crate::trap::Frame;
use alloc::sync::Arc;
use lazy_static::*;
///Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<ThreadControlBlock>>,
    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: Context,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: Context::zero_init(),
        }
    }
    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut Context {
        &mut self.idle_task_cx as *mut _
    }
    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<ThreadControlBlock>> {
        self.current.take()
    }
    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<ThreadControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: Mutex<Processor> = Mutex::new(Processor::new());
}
///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.lock();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner.lock();
            let next_task_cx_ptr = &task_inner.thread_cx as *const Context;
            task_inner.thread_status = Status::Running;
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            println!("no tasks available in run_tasks");
        }
    }
}
///Take the current task,leaving a None in its place
pub fn take_current_task() -> Option<Arc<ThreadControlBlock>> {
    PROCESSOR.lock().take_current()
}
///Get running task
pub fn current_task() -> Option<Arc<ThreadControlBlock>> {
    PROCESSOR.lock().current()
}

pub fn current_process() -> Arc<ProcessControlBlock> {
    current_task().unwrap().process.upgrade().unwrap()
}

///Get token of the address space of current task
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}
///Get the mutable reference to trap context of current task
pub fn current_trap_frame() -> &'static mut Frame {
    current_task()
        .unwrap()
        .inner
        .lock()
        .get_trap_frame()
}

pub fn current_trap_frame_user_va() -> usize {
    current_task()
        .unwrap()
        .inner
        .lock()
        .res
        .as_ref()
        .unwrap()
        .trap_frame_user_va()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut Context) {
    let mut processor = PROCESSOR.lock();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
