use std::ffi::c_void;

pub enum Event {
    InvokeVirtual {},
}

/*#[repr(align = "4096")]
#[repr(packed)]
pub struct XsaveArea {
    data : [u64;64]
}
*/
/*  #[naked]
unsafe fn trace(){
asm!(
"push r0",
"push r1",
"push r2",
"push r3",
"push r4",
"push r5",
"push r6",
"push r7",
"push r8",
"push r9",
"push r10",
"push r11",
"push r12",
"push r13",
"push r14",
"push r15",
"push cs",
"push ss",
"push ds",
"push es",
"push fs",
"push gs",
"push gs",
"push esp",
"mov rax, rsp",
"sub rsp, 4096",
"xsave rax",
"push rsp"
);



/*asm!(
"pop esp",
"add esp, 16", "movdqu dqword [esp], ymm31",
"add esp, 16", "movdqu dqword [esp], ymm30",
"add esp, 16", "movdqu dqword [esp], ymm29",
"add esp, 16", "movdqu dqword [esp], ymm28",
"add esp, 16", "movdqu dqword [esp], ymm27",
"add esp, 16", "movdqu dqword [esp], ymm26",
"add esp, 16", "movdqu dqword [esp], ymm25",
"add esp, 16", "movdqu dqword [esp], ymm24",
"add esp, 16", "movdqu dqword [esp], ymm23",
"add esp, 16", "movdqu dqword [esp], ymm22",
"add esp, 16", "movdqu dqword [esp], ymm21",
"add esp, 16", "movdqu dqword [esp], ymm20",
"add esp, 16", "movdqu dqword [esp], ymm19",
"add esp, 16", "movdqu dqword [esp], ymm18",
"add esp, 16", "movdqu dqword [esp], ymm17",
"add esp, 16", "movdqu dqword [esp], ymm16",
"add esp, 16", "movdqu dqword [esp], ymm15",
"add esp, 16", "movdqu dqword [esp], ymm14",
"add esp, 16", "movdqu dqword [esp], ymm13",
"add esp, 16", "movdqu dqword [esp], ymm12",
"add esp, 16", "movdqu dqword [esp], ymm11",
"add esp, 16", "movdqu dqword [esp], ymm10",
"add esp, 16", "movdqu dqword [esp], ymm9",
"add esp, 16", "movdqu dqword [esp], ymm8",
"add esp, 16", "movdqu dqword [esp], ymm7",
"add esp, 16", "movdqu dqword [esp], ymm6",
"add esp, 16", "movdqu dqword [esp], ymm5",
"add esp, 16", "movdqu dqword [esp], ymm4",
"add esp, 16", "movdqu dqword [esp], ymm3",
"add esp, 16", "movdqu dqword [esp], ymm2",
"add esp, 16", "movdqu dqword [esp], ymm1",
"add esp, 16", "movdqu dqword [esp], ymm0",
"add esp, 16", "movdqu dqword [esp], xmm7",
"add esp, 16", "movdqu dqword [esp], xmm6",
"add esp, 16", "movdqu dqword [esp], xmm5",
"add esp, 16", "movdqu dqword [esp], xmm4",
"add esp, 16", "movdqu dqword [esp], xmm3",
"add esp, 16", "movdqu dqword [esp], xmm2",
"add esp, 16", "movdqu dqword [esp], xmm1",
"add esp, 16", "movdqu dqword [esp], xmm0",
//todo should use xsave
"pop gs",
"pop gs",
"pop fs",
"pop es",
"pop ds",
"pop ss",
"pop cs",
"pop r15",
"pop r14",
"pop r13",
"pop r12",
"pop r11",
"pop r10",
"pop r9",
"pop r8",
"pop r7",
"pop r6",
"pop r5",
"pop r4",
"pop r3",
"pop r2",
"pop r1",
"pop r0",
)*/
}*/