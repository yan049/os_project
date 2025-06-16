mod context;
mod manager;
mod id;
mod process;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod thread;

use crate::loader::get_app_data_by_name;
use crate::sbi::legacy::shutdown;
use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
pub use manager::{Manager, fetch_task};
use switch::switch;
pub use thread::{ThreadControlBlock, Status};
pub use process::ProcessControlBlock;
pub use context::Context;
pub use manager::add_task;
pub use id::{IDLE_PID, KernelStack, PidHandle, kstack_alloc, pid_alloc, ThreadRes};
pub use processor::{
    Processor, current_process, current_task, current_trap_frame, current_user_token, run_tasks, schedule,
    take_current_task, current_trap_frame_user_va
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner.lock();
    let task_cx_ptr = &mut task_inner.thread_cx as *mut Context;
    // Change status to Ready
    task_inner.thread_status = Status::Ready;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}


/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner.lock();
    let process = task.process.upgrade().unwrap();
    let tid = task_inner.res.as_ref().unwrap().tid;
    // record exit code
    task_inner.exit_code = Some(exit_code);
    task_inner.res = None;
    // here we do not remove the thread since we are still using the kstack
    // it will be deallocated when sys_waittid is called
    drop(task_inner);
    drop(task);
    // however, if this is the main thread of current process
    // the process should terminate at once
    if tid == 0 {
        let pid = process.getpid();
        if pid == IDLE_PID {
            println!(
                "[kernel] Idle process exit with exit_code {} ...",
                exit_code
            );
            shutdown();
        }
        let mut process_inner = process.inner.lock();
        // mark this process as a zombie process
        process_inner.is_zombie = true;
        // record exit code of main process
        process_inner.exit_code = exit_code;

        {
            // move all child processes under init process
            let mut initproc_inner = INITPROC.inner.lock();
            for child in process_inner.children.iter() {
                child.inner.lock().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // deallocate user res (including tid/trap_cx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        let mut recycle_res = Vec::<ThreadRes>::new();
        for task in process_inner.threads.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            let mut task_inner = task.inner.lock();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }
        // dealloc_tid and dealloc_user_res require access to PCB inner, so we
        // need to collect those user res first, then release process_inner
        // for now to avoid deadlock/double borrow problem.
        drop(process_inner);
        recycle_res.clear();

        let mut process_inner = process.inner.lock();
        process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        process_inner.memory_set.recycle_data_pages();
        // Remove all tasks except for the main thread itself.
        // This is because we are still using the kstack under the TCB
        // of the main thread. This TCB, including its kstack, will be
        // deallocated when the process is reaped via waitpid.
        while process_inner.threads.len() > 1 {
            process_inner.threads.pop();
        }
    }
    drop(process);
    // we do not have to save task context
    let mut _unused = Context::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<ProcessControlBlock> = ProcessControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    );
}
///Add init process to the manager
pub fn add_initproc() {
    print!("addinit!\n");
    let _initproc = INITPROC.clone();
}
