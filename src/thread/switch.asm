	.section .text
        .globl switch
switch:
	# a0 is current task context, we save registers on it
	# a1 is next task context, we load it to registers
	# see task.rs::Context
        sd ra, 0x0(a0)
        ld ra, 0x0(a1)
        sd sp, 0x8(a0)
        ld sp, 0x8(a1)
        sd  s0, 0x10(a0)
        ld  s0, 0x10(a1)
        sd  s1, 0x18(a0)
        ld  s1, 0x18(a1)
        sd  s2, 0x20(a0)
        ld  s2, 0x20(a1)
        sd  s3, 0x28(a0)
        ld  s3, 0x28(a1)
        sd  s4, 0x30(a0)
        ld  s4, 0x30(a1)
        sd  s5, 0x38(a0)
        ld  s5, 0x38(a1)
        sd  s6, 0x40(a0)
        ld  s6, 0x40(a1)
        sd  s7, 0x48(a0)
        ld  s7, 0x48(a1)
        sd  s8, 0x50(a0)
        ld  s8, 0x50(a1)
        sd  s9, 0x58(a0)
        ld  s9, 0x58(a1)
        sd s10, 0x60(a0)
        ld s10, 0x60(a1)
        sd s11, 0x68(a0)
        ld s11, 0x68(a1)

        ret
