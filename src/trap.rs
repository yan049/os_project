//! trap handler
use core::arch::global_asm;
use riscv::interrupt::{Trap, Interrupt, Exception};
use riscv::register::scause;
use riscv::register::sstatus::*;
use riscv::register::*;
use riscv::register::mtvec::TrapMode;
use core::arch::asm;
use crate::syscall::syscall_handler;
use crate::thread::*;
use crate::timer::set_next_trigger;
use crate::mem::{TRAMPOLINE};
global_asm!(include_str!("trap.asm"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

/// enable timer interrupt in sie CSR
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[repr(C)]

/// trap frame
pub struct Frame {
    /// general regs[0..31].
    pub x: [usize; 32],
    /// CSR sstatus. (mainly use spp, which give privilege level before trap)
    pub sstatus: Sstatus,
    /// CSR sepc. (which is the address that trigger trap)
    pub sepc: usize,
    /// Addr of Page Table
    pub kernel_satp: usize,
    /// kernel stack
    pub kernel_sp: usize,
    /// Addr of trap_handler function
    pub trap_handler: usize,
}
impl Frame {
    /// set stack pointer to x_2 reg (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// init app trap frame
    pub fn app_init_frame(app_addr: usize, sp: usize, kernel_satp: usize,
        kernel_sp: usize, trap_handler: usize,) -> Self {
	// create sstatus with spp=user
        let sstatus_current = sstatus::read(); // save current spp
	unsafe {set_spp(SPP::User)};
	let sstatus = sstatus::read();
	unsafe {set_spp(sstatus_current.spp())};
	
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: app_addr,
	    kernel_satp,  // addr of page table
            kernel_sp,    // kernel stack
            trap_handler, // addr of trap_handler function
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}

#[unsafe(no_mangle)]
/// handle the trap by the scause
pub fn trap_handler() {
    set_kernel_trap_entry();
    let mut frame = current_trap_frame();
    // get trap cause
    let raw_scause: Trap<usize, usize> = scause::read().cause();
    let scause: Trap<Interrupt, Exception> = raw_scause.try_into().unwrap();
    
    let stval = stval::read(); // get extra value
    match scause {
        Trap::Exception(Exception::UserEnvCall) => {
            let id = frame.x[17];
            let args = [frame.x[10], frame.x[11], frame.x[12]];
            // ecall trigger trap, so we increase sepc by 1 to skip ecall when return
            frame.sepc += 4;
	    
            let result = syscall_handler(id, args) as usize;
	    frame = current_trap_frame();
	    frame.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause,
                stval,
                current_trap_frame().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("\n[kernel] IllegalInstruction in application, kernel killed it.");
             // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
	Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}",
                scause,
                stval
            );
        }
    }
    trap_return();
}

#[unsafe(no_mangle)]
/// set the return addrs for trap return and jump to return function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_frame_ptr = current_trap_frame_user_va();
    let user_satp = current_user_token();
    unsafe extern "C" {
        unsafe fn trap_entry_u();
        unsafe fn trap_exit_u();
    }
    let restore_va = trap_exit_u as usize - trap_entry_u as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") trap_frame_ptr,      // a0 = virt addr of Trap Context
            in("a1") user_satp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel(){
    let raw_scause: Trap<usize, usize> = scause::read().cause();
    let scause: Trap<Interrupt, Exception> = raw_scause.try_into().unwrap();
    
    let stval = stval::read();
    match scause {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            // do not schedule now
        }
        _ => {
            panic!(
                "Unsupported trap from kernel: {:?}, stval = {:#x}!",
                scause,
                stval
            );
        }
    }
}
