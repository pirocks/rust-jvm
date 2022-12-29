use iced_x86::code_asm::CodeAssembler;
use another_jit_vm::{DoubleRegister, FloatRegister, Register};

pub(crate) fn double_to_integer(assembler: &mut CodeAssembler, from: &DoubleRegister, temp_1: &Register, temp_2: &Register, temp_3: &DoubleRegister, temp_4: &DoubleRegister, to: &Register) {
//.LCPI1_0:
    //         .quad   0xc1e0000000000000
    // .LCPI1_1:
    //         .quad   0x41dfffffffc00000
    // example::doubletoint:
    //         xor     eax, eax
    //         ucomisd xmm0, xmm0
    //         maxsd   xmm0, qword ptr [rip + .LCPI1_0]
    //         minsd   xmm0, qword ptr [rip + .LCPI1_1]
    //         cvttsd2si       ecx, xmm0
    //         cmovnp  eax, ecx
    //         ret
    assembler.xor(to.to_native_64(), to.to_native_64()).unwrap();
    assembler.ucomisd(from.to_xmm(), from.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_64(), 0xc1e0000000000000u64).unwrap();
    assembler.movq(temp_3.to_xmm(), temp_1.to_native_64()).unwrap();
    assembler.mov(temp_2.to_native_64(), 0x41dfffffffc00000i64).unwrap();
    assembler.movq(temp_4.to_xmm(), temp_2.to_native_64()).unwrap();
    assembler.maxsd(from.to_xmm(), temp_3.to_xmm()).unwrap();
    assembler.minsd(from.to_xmm(), temp_4.to_xmm()).unwrap();
    assembler.cvttsd2si(temp_2.to_native_32(), from.to_xmm()).unwrap();
    assembler.cmovnp(to.to_native_32(), temp_2.to_native_32()).unwrap();
}

pub(crate) fn double_to_long(assembler: &mut CodeAssembler, from: &DoubleRegister, temp_1: &Register, temp_3: &DoubleRegister, to: &Register) {
//.LCPI0_0:
    //         .quad   0x43dfffffffffffff
    // example::doubletolong:
    //         cvttsd2si       rax, xmm0
    //         ucomisd xmm0, qword ptr [rip + .LCPI0_0]
    //         movabs  rcx, 9223372036854775807
    //         cmovbe  rcx, rax
    //         xor     eax, eax
    //         ucomisd xmm0, xmm0
    //         cmovnp  rax, rcx
    //         ret
    assembler.cvttsd2si(to.to_native_64(), from.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_64(), 0x43dfffffffffffffi64).unwrap();
    assembler.movq(temp_3.to_xmm(), temp_1.to_native_64()).unwrap();
    assembler.ucomisd(from.to_xmm(), temp_3.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_64(), 9223372036854775807i64).unwrap();
    assembler.cmovbe(temp_1.to_native_64(), to.to_native_64()).unwrap();
    assembler.xor(to.to_native_32(), to.to_native_32()).unwrap();
    assembler.ucomisd(from.to_xmm(), from.to_xmm()).unwrap();
    assembler.cmovnp(to.to_native_64(), temp_1.to_native_64()).unwrap();
}

pub(crate) fn float_to_long(assembler: &mut CodeAssembler, from: &FloatRegister, temp_1: &Register, temp_3: &FloatRegister, to: &Register) {
// .LCPI3_0:
    // .long   0x5effffff
    // example::floattolong:
    //     cvttss2si       rax, xmm0
    // ucomiss xmm0, dword ptr [rip + .LCPI3_0]
    // movabs  rcx, 9223372036854775807
    // cmovbe  rcx, rax
    // xor     eax, eax
    // ucomiss xmm0, xmm0
    // cmovnp  rax, rcx
    // ret
    assembler.cvttss2si(to.to_native_64(), from.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_32(), 0x5effffff).unwrap();
    assembler.movd(temp_3.to_xmm(), temp_1.to_native_32()).unwrap();
    assembler.ucomisd(from.to_xmm(), temp_3.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_64(), 9223372036854775807i64).unwrap();
    assembler.cmovbe(temp_1.to_native_64(), to.to_native_64()).unwrap();
    assembler.xor(to.to_native_64(), to.to_native_64()).unwrap();
    assembler.ucomiss(from.to_xmm(), from.to_xmm()).unwrap();
    assembler.cmovnp(to.to_native_64(), temp_1.to_native_64()).unwrap();
}

pub(crate) fn float_to_integer(assembler: &mut CodeAssembler, from: &FloatRegister, temp_1: &Register, temp_3: &FloatRegister, to: &Register) {
// .LCPI2_0:
    // .long   0x4effffff
    // example::floattoint:
    // cvttss2si eax, xmm0
    // ucomiss xmm0, dword ptr [rip + .LCPI2_0]
    // mov     ecx, 2147483647
    // cmovbe  ecx, eax
    // xor     eax, eax
    // ucomiss xmm0, xmm0
    // cmovnp  eax, ecx
    // ret
    assembler.cvtss2si(to.to_native_32(), from.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_32(), 0x4effffff).unwrap();
    assembler.movd(temp_3.to_xmm(), temp_1.to_native_32()).unwrap();
    assembler.ucomiss(from.to_xmm(), temp_3.to_xmm()).unwrap();
    assembler.mov(temp_1.to_native_32(), 2147483647).unwrap();
    assembler.cmovbe(temp_1.to_native_32(), to.to_native_32()).unwrap();
    assembler.xor(to.to_native_32(), to.to_native_32()).unwrap();
    assembler.ucomiss(from.to_xmm(), from.to_xmm()).unwrap();
    assembler.cmovnp(to.to_native_32(), temp_1.to_native_32()).unwrap();
}

