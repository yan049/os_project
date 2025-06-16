//! PCB structure
use super::ThreadControlBlock;
use super::id::IdAllocator;
use super::add_task;
use super::{PidHandle, pid_alloc};
use crate::mem::{KERNEL_SPACE, MemorySet};
use spin::Mutex;
use crate::trap::{Frame, trap_handler};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

pub struct ProcessControlBlock {
    // immutable
    pub pid: PidHandle,
    // mutable
    pub inner: Mutex<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub is_zombie: bool,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,
    pub exit_code: i32,
    pub threads: Vec<Option<Arc<ThreadControlBlock>>>,
    pub thread_res_allocator: IdAllocator,
}

impl ProcessControlBlockInner {
    pub fn alloc_tid(&mut self) -> usize {
        self.thread_res_allocator.alloc()
    }

    pub fn dealloc_tid(&mut self, tid: usize) {
        self.thread_res_allocator.dealloc(tid)
    }

    pub fn thread_count(&self) -> usize {
        self.threads.len()
    }

    pub fn get_task(&self, tid: usize) -> Arc<ThreadControlBlock> {
        self.threads[tid].as_ref().unwrap().clone()
    }
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        // alloc a pid
        let pid_handle = pid_alloc();
	let process = Arc::new(Self {
            pid: pid_handle,
            inner: Mutex::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    threads: Vec::new(),
                    thread_res_allocator: IdAllocator::new(),
            }),
        });
	// create a main thread, we should allocate ustack and trap_cx here
        let task = Arc::new(ThreadControlBlock::new(
            Arc::clone(&process),
            ustack_base,
            true,
        ));
        // prepare trap_cx of main thread
        let task_inner = task.inner.lock();
        let trap_frame = task_inner.get_trap_frame();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = task.kstack.get_top();
        drop(task_inner);
        *trap_frame = Frame::app_init_frame(
            entry_point,
            ustack_top,
            KERNEL_SPACE.lock().token(),
            kstack_top,
            trap_handler as usize,
        );
        // add main thread to the process
        let mut process_inner = process.inner.lock();
        process_inner.threads.push(Some(Arc::clone(&task)));
        drop(process_inner);
        // add main thread to scheduler
        add_task(task);
        process
    }
    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        // substitute memory_set
        self.inner.lock().memory_set = memory_set;
        // then we alloc user resource for main thread again
        // since memory_set has been changed
        let task = self.inner.lock().get_task(0);
        let mut task_inner = task.inner.lock();
        task_inner.res.as_mut().unwrap().ustack_base = ustack_base;
        task_inner.res.as_mut().unwrap().alloc_user_res();
        task_inner.trap_frame_ppn = task_inner.res.as_mut().unwrap().trap_frame_ppn();
	let user_sp = task_inner.res.as_mut().unwrap().ustack_top();
        // initialize trap_cx
        let trap_frame = task_inner.get_trap_frame();
        *trap_frame = Frame::app_init_frame(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            task.kstack.get_top(),
            trap_handler as usize,
        );
        // **** release inner automatically
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent = self.inner.lock();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent.memory_set);
        // alloc a pid and a kernel stack in kernel space
        let pid = pid_alloc();
	// create child process pcb
        let child = Arc::new(Self {
            pid,
            inner: Mutex::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    threads: Vec::new(),
                    thread_res_allocator: IdAllocator::new(),
            }),
        });
        // add child
        parent.children.push(Arc::clone(&child));
	// create main thread of child process
        let task = Arc::new(ThreadControlBlock::new(
            Arc::clone(&child),
            parent
                .get_task(0)
                .inner
		.lock()
                .res
                .as_ref()
                .unwrap()
                .ustack_base(),
            // here we do not allocate trap_cx or ustack again
            // but mention that we allocate a new kstack here
            false,
        ));
	// attach task to child process
        let mut child_inner = child.inner.lock();
        child_inner.threads.push(Some(Arc::clone(&task)));
        drop(child_inner);
	// modify kstack_top in trap_cx of this thread
        let task_inner = task.inner.lock();
        let trap_frame = task_inner.get_trap_frame();
        trap_frame.kernel_sp = task.kstack.get_top();
        drop(task_inner);
        // add this thread to scheduler
        add_task(task);
        child
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}
