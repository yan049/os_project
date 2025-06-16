use core::arch::asm;

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

fn syscall_handler(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}
pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall_handler(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len()])
}
pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall_handler(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall_handler(SYSCALL_EXIT, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall_handler(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall_handler(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall_handler(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall_handler(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    syscall_handler(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall_handler(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}
pub fn sys_thread_create(entry: usize, arg: usize) -> isize {
    syscall_handler(SYSCALL_THREAD_CREATE, [entry, arg, 0])
}

pub fn sys_gettid() -> isize {
    syscall_handler(SYSCALL_GETTID, [0; 3])
}

pub fn sys_waittid(tid: usize) -> isize {
    syscall_handler(SYSCALL_WAITTID, [tid, 0, 0])
}
