.section .text.trampoline
        .globl trap_entry_u
        .globl trap_exit_u
	
    .align 2  # address of trap handlers must be 4-byte aligned
    trap_entry_u:

	# assume sscratch stores the kernel stack, sp stores user stack
	# swap kernel and user stack
        csrrw sp, sscratch, sp

	# save general purpose regs except sp and x0
        # sd x0, 0*8(sp)
        sd x1,   1*8(sp)
        # sd sp, 2*8(sp)
        sd x3,   3*8(sp)
        sd x4,   4*8(sp)
        sd x5,   5*8(sp)
        sd x6,   6*8(sp)
        sd x7,   7*8(sp)
        sd x8,   8*8(sp)
        sd x9,   9*8(sp)
        sd x10, 10*8(sp)
        sd x11, 11*8(sp)
        sd x12, 12*8(sp)
        sd x13, 13*8(sp)
        sd x14, 14*8(sp)
        sd x15, 15*8(sp)
        sd x16, 16*8(sp)
        sd x17, 17*8(sp)
        sd x18, 18*8(sp)
        sd x19, 19*8(sp)
        sd x20, 20*8(sp)
        sd x21, 21*8(sp)
        sd x22, 22*8(sp)
        sd x23, 23*8(sp)
        sd x24, 24*8(sp)
        sd x25, 25*8(sp)
        sd x26, 26*8(sp)
        sd x27, 27*8(sp)
        sd x28, 28*8(sp)
        sd x29, 29*8(sp)
        sd x30, 30*8(sp)
        sd x31, 31*8(sp)

	# now we can use registers
	# save CSRs
        csrr t0, sstatus
        csrr t1, sepc
	# save user stack.
        csrr t2, sscratch

        sd t0, 32*8(sp)
        sd t1, 33*8(sp)
        sd t2,  2*8(sp)

	# load kernel_satp into t0
	ld t0, 34*8(sp)
	# load trap_handler into t1
	ld t1, 36*8(sp)
	# move to kernel_sp
	ld sp, 35*8(sp)
	# switch to kernel space
	csrw satp, t0
	sfence.vma
	# jump to trap_handler
	jr t1

    trap_exit_u:

	# a0: *TrapContext in user space(Constant); a1: user space token
	# switch to user space
	csrw satp, a1
	sfence.vma
	csrw sscratch, a0
	mv sp, a0
	# now sp points to TrapContext in user space, start restoring based on it
	# restore sstatus/sepc
	ld t0, 32*8(sp)
	ld t1, 33*8(sp)
	csrw sstatus, t0
	csrw sepc, t1	# restore CSR.
	
	# restore general-purpose regs.
        # ld x0, 0*8(sp)
        ld x1,   1*8(sp)
        # ld sp, 2*8(sp)
        ld x3,   3*8(sp)
        ld x4,   4*8(sp)
        ld x5,   5*8(sp)
        ld x6,   6*8(sp)
        ld x7,   7*8(sp)
        ld x8,   8*8(sp)
        ld x9,   9*8(sp)
        ld x10, 10*8(sp)
        ld x11, 11*8(sp)
        ld x12, 12*8(sp)
        ld x13, 13*8(sp)
        ld x14, 14*8(sp)
        ld x15, 15*8(sp)
        ld x16, 16*8(sp)
        ld x17, 17*8(sp)
        ld x18, 18*8(sp)
        ld x19, 19*8(sp)
        ld x20, 20*8(sp)
        ld x21, 21*8(sp)
        ld x22, 22*8(sp)
        ld x23, 23*8(sp)
        ld x24, 24*8(sp)
        ld x25, 25*8(sp)
        ld x26, 26*8(sp)
        ld x27, 27*8(sp)
        ld x28, 28*8(sp)
        ld x29, 29*8(sp)
        ld x30, 30*8(sp)
        ld x31, 31*8(sp)

	# back to user stack
	ld sp, 2*8(sp)
	sret
