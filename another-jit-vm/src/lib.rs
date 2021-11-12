#![feature(asm)]
// save all registers when entering and exiting vm
// methodid to code id mapping is handled seperately
// exit handling has registered handling but actual handling is seperate
// have another layer above this which gets rid of native points and does everytthing in terms of IR
// have java layer above that

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Range;

use memoffset::offset_of;

pub struct MethodImplementationID(usize);

pub struct MethodOffset(usize);

pub struct VMState<T: Sized> {
    method_id_max: MethodImplementationID,
    exit_handlers: HashMap<MethodImplementationID, HashMap<MethodOffset, Box<dyn FnMut(&VMExitEvent) -> VMExitAction<T>>>>,
    code_regions: HashMap<MethodImplementationID, Range<*mut c_void>>,
    mmaped_code_region_base: *mut c_void,
}

impl<T> Drop for VMState<T> {
    fn drop(&mut self) {
        todo!()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SavedRegistersWithIP {
    rax: *mut c_void,
    rbx: *mut c_void,
    rcx: *mut c_void,
    rdx: *mut c_void,
    rsi: *mut c_void,
    rdi: *mut c_void,
    rbp: *mut c_void,
    rsp: *mut c_void,
    r8: *mut c_void,
    r9: *mut c_void,
    r10: *mut c_void,
    r11: *mut c_void,
    r12: *mut c_void,
    r13: *mut c_void,
    r14: *mut c_void,
    xsave_area: [u64; 4096],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SavedRegistersWithoutIP {
    rax: *mut c_void,
    rbx: *mut c_void,
    rcx: *mut c_void,
    rdx: *mut c_void,
    rsi: *mut c_void,
    rdi: *mut c_void,
    rbp: *mut c_void,
    rsp: *mut c_void,
    r8: *mut c_void,
    r9: *mut c_void,
    r10: *mut c_void,
    r11: *mut c_void,
    r12: *mut c_void,
    r13: *mut c_void,
    r14: *mut c_void,
    xsave_area: [u64; 4096],
}

pub struct VMExitEvent {
    saved_guest_registers: SavedRegistersWithIP,
}

pub enum VMExitAction<T: Sized> {
    ExitVMCompletely {
        return_data: T
    },
    ReturnTo {
        return_register_state: SavedRegistersWithIP
    },
}

#[repr(C)]
struct JITContext {
    registers_to_copy_in: SavedRegistersWithoutIP,
}

pub const RAX_OFFSET_CONST: usize = 0;
pub const RBX_OFFSET_CONST: usize = 8;
pub const RCX_OFFSET_CONST: usize = 16;
pub const RDX_OFFSET_CONST: usize = 24;
pub const RSI_OFFSET_CONST: usize = 32;
pub const RDI_OFFSET_CONST: usize = 40;
pub const RBP_OFFSET_CONST: usize = 48;
pub const RSP_OFFSET_CONST: usize = 56;
pub const R8_OFFSET_CONST: usize = 64;
pub const R9_OFFSET_CONST: usize = 72;
pub const R10_OFFSET_CONST: usize = 80;
pub const R11_OFFSET_CONST: usize = 88;
pub const R12_OFFSET_CONST: usize = 96;
pub const R13_OFFSET_CONST: usize = 104;
pub const R14_OFFSET_CONST: usize = 112;
pub const XSAVE_AREA_OFFSET_CONST: usize = 120;

impl<T> VMState<T> {
    pub fn launch_vm(&self, method_id: MethodImplementationID, initial_registers: SavedRegistersWithoutIP) -> T {
        let code_region = self.code_regions.get(&method_id).unwrap();
        let branch_to = code_region.start;
        let rax_offset = offset_of!(SavedRegistersWithoutIP,rax) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rax_offset, RAX_OFFSET_CONST);
        let rbx_offset = offset_of!(SavedRegistersWithoutIP,rbx) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rbx_offset, RBX_OFFSET_CONST);
        let rcx_offset = offset_of!(SavedRegistersWithoutIP,rcx) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rcx_offset, RCX_OFFSET_CONST);
        let rdx_offset = offset_of!(SavedRegistersWithoutIP,rdx) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rdx_offset, RDX_OFFSET_CONST);
        let rsi_offset = offset_of!(SavedRegistersWithoutIP,rsi) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rsi_offset, RSI_OFFSET_CONST);
        let rdi_offset = offset_of!(SavedRegistersWithoutIP,rdi) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rdi_offset, RDI_OFFSET_CONST);
        let rbp_offset = offset_of!(SavedRegistersWithoutIP,rbp) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rbp_offset, RBP_OFFSET_CONST);
        let rsp_offset = offset_of!(SavedRegistersWithoutIP,rsp) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(rsp_offset, RSP_OFFSET_CONST);
        let r8_offset = offset_of!(SavedRegistersWithoutIP,r8) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r8_offset, R8_OFFSET_CONST);
        let r9_offset = offset_of!(SavedRegistersWithoutIP,r9) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r9_offset, R9_OFFSET_CONST);
        let r10_offset = offset_of!(SavedRegistersWithoutIP,r10) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r10_offset, R10_OFFSET_CONST);
        let r11_offset = offset_of!(SavedRegistersWithoutIP,r11) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r11_offset, R11_OFFSET_CONST);
        let r12_offset = offset_of!(SavedRegistersWithoutIP,r12) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r12_offset, R12_OFFSET_CONST);
        let r13_offset = offset_of!(SavedRegistersWithoutIP,r13) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r13_offset, R13_OFFSET_CONST);
        let r14_offset = offset_of!(SavedRegistersWithoutIP,r14) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(r14_offset, R14_OFFSET_CONST);
        let xsave_area_offset = offset_of!(SavedRegistersWithoutIP,xsave_area) + offset_of!(JITContext,registers_to_copy_in);
        assert_eq!(xsave_area_offset, XSAVE_AREA_OFFSET_CONST);
        let mut jit_context = JITContext {
            registers_to_copy_in: initial_registers
        };
        let jit_context_pointer = (&mut jit_context) as *mut c_void;
        unsafe {
            asm!(
            //todo push all registers to avoid breaking stuff
            "mov r15, {0}",
            "mov rax,[r15 + rax_offset_const]",
            "mov rbx,[r15 + rbx_offset_const]",
            "mov rcx,[r15 + rcx_offset_const]",
            "mov rdx,[r15 + rdx_offset_const]",
            "mov rsi,[r15 + rsi_offset_const]",
            "mov rbp,[r15 + rbp_offset_const]",
            "mov rsp,[r15 + rsp_offset_const]",
            "mov r8,[r15 + r8_offset_const]",
            "mov r9,[r15 + r9_offset_const]",
            "mov r10,[r15 + r10_offset_const]",
            "mov r11,[r15 + r11_offset_const]",
            "mov r12,[r15 + r12_offset_const]",
            "mov r13,[r15 + r13_offset_const]",
            "mov r14,[r15 + r14_offset_const]",
            "xrstor [r15 + xsave_area_offset_const]",
            "call qword [r15 + rdi_offset_const]"
            in(reg) jit_context_pointer,
            rax_offset_const = const RAX_OFFSET_CONST,
            rbx_offset_const = const RBX_OFFSET_CONST,
            rcx_offset_const = const RCX_OFFSET_CONST,
            rdx_offset_const = const RDX_OFFSET_CONST,
            rsi_offset_const = const RSI_OFFSET_CONST,
            rdi_offset_const = const RDI_OFFSET_CONST,
            rbp_offset_const = const RBP_OFFSET_CONST,
            rsp_offset_const = const RSP_OFFSET_CONST,
            r8_offset_const = const R8_OFFSET_CONST,
            r9_offset_const = const R9_OFFSET_CONST,
            r10_offset_const = const R10_OFFSET_CONST,
            r11_offset_const = const R11_OFFSET_CONST,
            r12_offset_const = const R12_OFFSET_CONST,
            r13_offset_const = const R13_OFFSET_CONST,
            r14_offset_const = const R14_OFFSET_CONST,
            xsave_area_offset_const = const XSAVE_AREA_OFFSET_CONST,
            );
        }
    }
}

pub struct NativeStack {
    mmaped_stack_base: *mut c_void,
}

impl Drop for NativeStack {
    fn drop(&mut self) {
        todo!()
    }
}