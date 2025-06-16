//! implement system call
use crate::thread::{current_process, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, ThreadControlBlock, add_task
};
use crate::trap::{Frame, trap_handler};
use crate::mem::{translated_byte_buffer, translated_refmut, translated_str, KERNEL_SPACE};
use crate::loader::get_app_data_by_name;
use crate::timer::get_time_ms;
use crate::sbi::legacy::console_getchar;
use alloc::sync::Arc;
const SYSCALL_READ: usize = 1;
const SYSCALL_WRITE: usize = 2;
const SYSCALL_EXIT: usize = 3;
const SYSCALL_YIELD: usize = 4;
const SYSCALL_GET_TIME: usize = 5;
const SYSCALL_GETPID: usize = 6;
const SYSCALL_FORK: usize = 7;
const SYSCALL_EXEC: usize = 8;
const SYSCALL_WAITPID: usize = 9;
const SYSCALL_THREAD_CREATE: usize = 10;
const SYSCALL_GETTID: usize = 11;
const SYSCALL_WAITTID: usize = 12;

pub fn syscall_handler(syscall_id: usize, args: [usize; 3]) -> isize {  
    match syscall_id {
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
	SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

/// (now support stdout only)
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            let mut c: usize;
            loop {
                c = console_getchar();
                if c == usize::MAX {
                    suspend_current_and_run_next();
                    continue;
                } else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().process.upgrade().unwrap().getpid() as isize
}

pub fn sys_fork() -> isize {
    let current_process = current_process();
    let new_process = current_process.fork();
    let new_pid = new_process.getpid();
    // modify trap context of new_task, because it returns immediately after switching
    let new_process_inner = new_process.inner.lock();
    let task = new_process_inner.threads[0].as_ref().unwrap();
    let trap_frame = task.inner.lock().get_trap_frame();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_frame.x[10] = 0;
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let process = current_process();
        process.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let process = current_process();
    // find a child process

    let mut inner = process.inner.lock();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        p.inner.lock().is_zombie && (pid == -1 || pid as usize == p.getpid())
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        let exit_code = child.inner.lock().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    //  release current PCB automatically
}
pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    // create a new thread
    let new_task = Arc::new(ThreadControlBlock::new(
        Arc::clone(&process),
        task.inner
	    .lock()
            .res
            .as_ref()
            .unwrap()
            .ustack_base,
        true,
    ));
    // add new task to scheduler
    add_task(Arc::clone(&new_task));
    let new_task_inner = new_task.inner.lock();
    let new_task_res = new_task_inner.res.as_ref().unwrap();
    let new_task_tid = new_task_res.tid;
    let mut process_inner = process.inner.lock();
    // add new thread to current process
    let tasks = &mut process_inner.threads;
    while tasks.len() < new_task_tid + 1 {
        tasks.push(None);
    }
    tasks[new_task_tid] = Some(Arc::clone(&new_task));
    let kernel_token = KERNEL_SPACE.lock().token();
    let new_task_trap_frame = new_task_inner.get_trap_frame();
    *new_task_trap_frame = Frame::app_init_frame(
        entry,
        new_task_res.ustack_top(),
        kernel_token,
        new_task.kstack.get_top(),
        trap_handler as usize,
    );
    (*new_task_trap_frame).x[10] = arg;
    new_task_tid as isize
}

pub fn sys_gettid() -> isize {
    current_task()
        .unwrap()
        .inner
        .lock()
        .res
        .as_ref()
        .unwrap()
        .tid as isize
}

/// thread does not exist, return -1
/// thread has not exited yet, return -2
/// otherwise, return thread's exit code
pub fn sys_waittid(tid: usize) -> i32 {
    let task = current_task().unwrap();
    let process = task.process.upgrade().unwrap();
    let task_inner = task.inner.lock();
    let mut process_inner = process.inner.lock();
    // a thread cannot wait for itself
    if task_inner.res.as_ref().unwrap().tid == tid {
        return -1;
    }
    let mut exit_code: Option<i32> = None;
    let waited_task = process_inner.threads[tid].as_ref();
    if let Some(waited_task) = waited_task {
        if let Some(waited_exit_code) = waited_task.inner.lock().exit_code {
            exit_code = Some(waited_exit_code);
        }
    } else {
        // waited thread does not exist
        return -1;
    }
    if let Some(exit_code) = exit_code {
        // dealloc the exited thread
        process_inner.threads[tid] = None;
        exit_code
    } else {
        // waited thread has not exited
        -2
    }
}
