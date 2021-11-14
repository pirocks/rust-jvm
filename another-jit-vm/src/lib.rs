#![feature(asm)]
// save all registers when entering and exiting vm
// methodid to code id mapping is handled seperately
// exit handling has registered handling but actual handling is seperate
// have another layer above this which gets rid of native points and does everytthing in terms of IR
// have java layer above that

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Range;
use std::ptr::null_mut;
use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;

use iced_x86::code_asm::{CodeAssembler, r15};
use memoffset::offset_of;

pub struct MethodImplementationID(usize);

pub struct MethodOffset(usize);

pub struct VMState<T: Sized> {
    method_id_max: AtomicUsize,
    exit_handlers: RwLock<HashMap<MethodImplementationID, HashMap<MethodOffset, Box<dyn FnMut(&VMExitEvent) -> VMExitAction<T>>>>>,
    code_regions: RwLock<HashMap<MethodImplementationID, Range<*mut c_void>>>,
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
    rip: *mut c_void,
    saved_registers_without_ip: SavedRegistersWithoutIP,
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
    xsave_area: [u64; 64],
}

pub struct VMExitEvent {
    saved_guest_registers: SavedRegistersWithIP,
}

pub enum VMExitAction<T: Sized> {
    ExitVMCompletely { return_data: T },
    ReturnTo { return_register_state: SavedRegistersWithIP },
}

#[repr(C)]
struct JITContext {
    guest_registers: SavedRegistersWithIP,
    vm_native_saved_registers: SavedRegistersWithIP,
}

pub const RIP_GUEST_OFFSET_CONST: usize = 0;
pub const RAX_GUEST_OFFSET_CONST: usize = 0 + 8;
pub const RBX_GUEST_OFFSET_CONST: usize = 8 + 8;
pub const RCX_GUEST_OFFSET_CONST: usize = 16 + 8;
pub const RDX_GUEST_OFFSET_CONST: usize = 24 + 8;
pub const RSI_GUEST_OFFSET_CONST: usize = 32 + 8;
pub const RDI_GUEST_OFFSET_CONST: usize = 40 + 8;
pub const RBP_GUEST_OFFSET_CONST: usize = 48 + 8;
pub const RSP_GUEST_OFFSET_CONST: usize = 56 + 8;
pub const R8_GUEST_OFFSET_CONST: usize = 64 + 8;
pub const R9_GUEST_OFFSET_CONST: usize = 72 + 8;
pub const R10_GUEST_OFFSET_CONST: usize = 80 + 8;
pub const R11_GUEST_OFFSET_CONST: usize = 88 + 8;
pub const R12_GUEST_OFFSET_CONST: usize = 96 + 8;
pub const R13_GUEST_OFFSET_CONST: usize = 104 + 8;
pub const R14_GUEST_OFFSET_CONST: usize = 112 + 8;
pub const XSAVE_AREA_GUEST_OFFSET_CONST: usize = 120 + 8;

pub const RAX_NATIVE_OFFSET_CONST: usize = 0 + 120 + 4096;
pub const RBX_NATIVE_OFFSET_CONST: usize = 8 + 120 + 4096;
pub const RCX_NATIVE_OFFSET_CONST: usize = 16 + 120 + 4096;
pub const RDX_NATIVE_OFFSET_CONST: usize = 24 + 120 + 4096;
pub const RSI_NATIVE_OFFSET_CONST: usize = 32 + 120 + 4096;
pub const RDI_NATIVE_OFFSET_CONST: usize = 40 + 120 + 4096;
pub const RBP_NATIVE_OFFSET_CONST: usize = 48 + 120 + 4096;
pub const RSP_NATIVE_OFFSET_CONST: usize = 56 + 120 + 4096;
pub const R8_NATIVE_OFFSET_CONST: usize = 64 + 120 + 4096;
pub const R9_NATIVE_OFFSET_CONST: usize = 72 + 120 + 4096;
pub const R10_NATIVE_OFFSET_CONST: usize = 80 + 120 + 4096;
pub const R11_NATIVE_OFFSET_CONST: usize = 88 + 120 + 4096;
pub const R12_NATIVE_OFFSET_CONST: usize = 96 + 120 + 4096;
pub const R13_NATIVE_OFFSET_CONST: usize = 104 + 120 + 4096;
pub const R14_NATIVE_OFFSET_CONST: usize = 112 + 120 + 4096;
pub const XSAVE_AREA_NATIVE_OFFSET_CONST: usize = 120 + 120 + 4096;

impl<T> VMState<T> {
    pub fn launch_vm(&self, method_id: MethodImplementationID, initial_registers: SavedRegistersWithoutIP) -> T {
        let code_region: &Range<*mut c_void> = self.code_regions.get(&method_id).unwrap();
        let branch_to = code_region.start;
        let rip_guest_offset = offset_of!(SavedRegistersWithIP, rip) + offset_of!(JITContext, registers_to_copy_in) + offset_of!(JITContext, guest_registers);
        assert_eq!(rip_guest_offset, RIP_GUEST_OFFSET_CONST);
        let rax_guest_offset = offset_of!(SavedRegistersWithoutIP, rax) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rax_guest_offset, RAX_GUEST_OFFSET_CONST);
        let rbx_guest_offset = offset_of!(SavedRegistersWithoutIP, rbx) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rbx_guest_offset, RBX_GUEST_OFFSET_CONST);
        let rcx_guest_offset = offset_of!(SavedRegistersWithoutIP, rcx) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rcx_guest_offset, RCX_GUEST_OFFSET_CONST);
        let rdx_guest_offset = offset_of!(SavedRegistersWithoutIP, rdx) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rdx_guest_offset, RDX_GUEST_OFFSET_CONST);
        let rsi_guest_offset = offset_of!(SavedRegistersWithoutIP, rsi) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rsi_guest_offset, RSI_GUEST_OFFSET_CONST);
        let rdi_guest_offset = offset_of!(SavedRegistersWithoutIP, rdi) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rdi_guest_offset, RDI_GUEST_OFFSET_CONST);
        let rbp_guest_offset = offset_of!(SavedRegistersWithoutIP, rbp) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rbp_guest_offset, RBP_GUEST_OFFSET_CONST);
        let rsp_guest_offset = offset_of!(SavedRegistersWithoutIP, rsp) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(rsp_guest_offset, RSP_GUEST_OFFSET_CONST);
        let r8_guest_offset = offset_of!(SavedRegistersWithoutIP, r8) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r8_guest_offset, R8_GUEST_OFFSET_CONST);
        let r9_guest_offset = offset_of!(SavedRegistersWithoutIP, r9) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r9_guest_offset, R9_GUEST_OFFSET_CONST);
        let r10_guest_offset = offset_of!(SavedRegistersWithoutIP, r10) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r10_guest_offset, R10_GUEST_OFFSET_CONST);
        let r11_guest_offset = offset_of!(SavedRegistersWithoutIP, r11) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r11_guest_offset, R11_GUEST_OFFSET_CONST);
        let r12_guest_offset = offset_of!(SavedRegistersWithoutIP, r12) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r12_guest_offset, R12_GUEST_OFFSET_CONST);
        let r13_guest_offset = offset_of!(SavedRegistersWithoutIP, r13) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r13_guest_offset, R13_GUEST_OFFSET_CONST);
        let r14_guest_offset = offset_of!(SavedRegistersWithoutIP, r14) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(r14_guest_offset, R14_GUEST_OFFSET_CONST);
        let xsave_area_guest_offset = offset_of!(SavedRegistersWithoutIP, xsave_area) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, guest_registers);
        assert_eq!(xsave_area_guest_offset, XSAVE_AREA_GUEST_OFFSET_CONST);
        let rax_native_offset = offset_of!(SavedRegistersWithoutIP, rax) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rax_native_offset, RAX_NATIVE_OFFSET_CONST);
        let rbx_native_offset = offset_of!(SavedRegistersWithoutIP, rbx) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rbx_native_offset, RBX_NATIVE_OFFSET_CONST);
        let rcx_native_offset = offset_of!(SavedRegistersWithoutIP, rcx) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rcx_native_offset, RCX_NATIVE_OFFSET_CONST);
        let rdx_native_offset = offset_of!(SavedRegistersWithoutIP, rdx) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rdx_native_offset, RDX_NATIVE_OFFSET_CONST);
        let rsi_native_offset = offset_of!(SavedRegistersWithoutIP, rsi) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rsi_native_offset, RSI_NATIVE_OFFSET_CONST);
        let rdi_native_offset = offset_of!(SavedRegistersWithoutIP, rdi) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rdi_native_offset, RDI_NATIVE_OFFSET_CONST);
        let rbp_native_offset = offset_of!(SavedRegistersWithoutIP, rbp) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rbp_native_offset, RBP_NATIVE_OFFSET_CONST);
        let rsp_native_offset = offset_of!(SavedRegistersWithoutIP, rsp) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rsp_native_offset, RSP_NATIVE_OFFSET_CONST);
        let r8_native_offset = offset_of!(SavedRegistersWithoutIP, r8) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r8_native_offset, R8_NATIVE_OFFSET_CONST);
        let r9_native_offset = offset_of!(SavedRegistersWithoutIP, r9) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r9_native_offset, R9_NATIVE_OFFSET_CONST);
        let r10_native_offset = offset_of!(SavedRegistersWithoutIP, r10) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r10_native_offset, R10_NATIVE_OFFSET_CONST);
        let r11_native_offset = offset_of!(SavedRegistersWithoutIP, r11) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r11_native_offset, R11_NATIVE_OFFSET_CONST);
        let r12_native_offset = offset_of!(SavedRegistersWithoutIP, r12) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r12_native_offset, R12_NATIVE_OFFSET_CONST);
        let r13_native_offset = offset_of!(SavedRegistersWithoutIP, r13) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r13_native_offset, R13_NATIVE_OFFSET_CONST);
        let r14_native_offset = offset_of!(SavedRegistersWithoutIP, r14) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r14_native_offset, R14_NATIVE_OFFSET_CONST);
        let xsave_area_native_offset = offset_of!(SavedRegistersWithoutIP, xsave_area) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(xsave_area_native_offset, XSAVE_AREA_NATIVE_OFFSET_CONST);
        let mut jit_context = JITContext {
            guest_registers: SavedRegistersWithIP { rip: branch_to, saved_registers_without_ip: initial_registers },
            vm_native_saved_registers: SavedRegistersWithIP {
                rip: null_mut(),
                saved_registers_without_ip: SavedRegistersWithoutIP {
                    rax: null_mut(),
                    rbx: null_mut(),
                    rcx: null_mut(),
                    rdx: null_mut(),
                    rsi: null_mut(),
                    rdi: null_mut(),
                    rbp: null_mut(),
                    rsp: null_mut(),
                    r8: null_mut(),
                    r9: null_mut(),
                    r10: null_mut(),
                    r11: null_mut(),
                    r12: null_mut(),
                    r13: null_mut(),
                    r14: null_mut(),
                    xsave_area: [0; 64],
                },
            },
        };
        let jit_context_pointer = (&mut jit_context) as *mut c_void;
        unsafe {
            asm!(
            //save all registers to avoid breaking stuff
            "mov r15, {0}",
            "mov [r15 + rax_native_offset_const], rax",
            "mov [r15 + rbx_native_offset_const], rbx",
            "mov [r15 + rcx_native_offset_const], rcx",
            "mov [r15 + rdx_native_offset_const], rdx",
            "mov [r15 + rsi_native_offset_const], rsi",
            "mov [r15 + rbp_native_offset_const], rbp",
            "mov [r15 + rsp_native_offset_const], rsp",
            "mov [r15 + r8_native_offset_const], r8",
            "mov [r15 + r9_native_offset_const], r9",
            "mov [r15 + r10_native_offset_const], r10",
            "mov [r15 + r11_native_offset_const], r11",
            "mov [r15 + r12_native_offset_const], r12",
            "mov [r15 + r13_native_offset_const], r13",
            "mov [r15 + r14_native_offset_const], r14",
            "xstor [r15 + xsave_area_native_offset_const]",
            "lea rax, [rip+after_enter]",
            "mov [r15 + ], rax",
            //load expected register values
            "mov rax,[r15 + rax_guest_offset_const]",
            "mov rbx,[r15 + rbx_guest_offset_const]",
            "mov rcx,[r15 + rcx_guest_offset_const]",
            "mov rdx,[r15 + rdx_guest_offset_const]",
            "mov rsi,[r15 + rsi_guest_offset_const]",
            "mov rbp,[r15 + rbp_guest_offset_const]",
            "mov rsp,[r15 + rsp_guest_offset_const]",
            "mov r8,[r15 + r8_guest_offset_const]",
            "mov r9,[r15 + r9_guest_offset_const]",
            "mov r10,[r15 + r10_guest_offset_const]",
            "mov r11,[r15 + r11_guest_offset_const]",
            "mov r12,[r15 + r12_guest_offset_const]",
            "mov r13,[r15 + r13_guest_offset_const]",
            "mov r14,[r15 + r14_guest_offset_const]",
            "xrstor [r15 + xsave_area_guest_offset_const]",
            "call qword [r15 + rdi_guest_offset_const]",
            "after_enter:"
            in(reg) jit_context_pointer,
            rax_guest_offset_const = const RAX_GUEST_OFFSET_CONST,
            rbx_guest_offset_const = const RBX_GUEST_OFFSET_CONST,
            rcx_guest_offset_const = const RCX_GUEST_OFFSET_CONST,
            rdx_guest_offset_const = const RDX_GUEST_OFFSET_CONST,
            rsi_guest_offset_const = const RSI_GUEST_OFFSET_CONST,
            rdi_guest_offset_const = const RDI_GUEST_OFFSET_CONST,
            rbp_guest_offset_const = const RBP_GUEST_OFFSET_CONST,
            rsp_guest_offset_const = const RSP_GUEST_OFFSET_CONST,
            r8_guest_offset_const = const R8_GUEST_OFFSET_CONST,
            r9_guest_offset_const = const R9_GUEST_OFFSET_CONST,
            r10_guest_offset_const = const R10_GUEST_OFFSET_CONST,
            r11_guest_offset_const = const R11_GUEST_OFFSET_CONST,
            r12_guest_offset_const = const R12_GUEST_OFFSET_CONST,
            r13_guest_offset_const = const R13_GUEST_OFFSET_CONST,
            r14_guest_offset_const = const R14_GUEST_OFFSET_CONST,
            xsave_area_guest_offset_const = const XSAVE_AREA_GUEST_OFFSET_CONST,
            rax_native_offset_const = const RAX_NATIVE_OFFSET_CONST,
            rbx_native_offset_const = const RBX_NATIVE_OFFSET_CONST,
            rcx_native_offset_const = const RCX_NATIVE_OFFSET_CONST,
            rdx_native_offset_const = const RDX_NATIVE_OFFSET_CONST,
            rsi_native_offset_const = const RSI_NATIVE_OFFSET_CONST,
            rdi_native_offset_const = const RDI_NATIVE_OFFSET_CONST,
            rbp_native_offset_const = const RBP_NATIVE_OFFSET_CONST,
            rsp_native_offset_const = const RSP_NATIVE_OFFSET_CONST,
            r8_native_offset_const = const R8_NATIVE_OFFSET_CONST,
            r9_native_offset_const = const R9_NATIVE_OFFSET_CONST,
            r10_native_offset_const = const R10_NATIVE_OFFSET_CONST,
            r11_native_offset_const = const R11_NATIVE_OFFSET_CONST,
            r12_native_offset_const = const R12_NATIVE_OFFSET_CONST,
            r13_native_offset_const = const R13_NATIVE_OFFSET_CONST,
            r14_native_offset_const = const R14_NATIVE_OFFSET_CONST,
            xsave_area_native_offset_const = const XSAVE_AREA_NATIVE_OFFSET_CONST,
            );
        }
    }
}

pub fn gen_vm_exit(assembler: &mut CodeAssembler) {
    assembler.mov(r15 + RIP_GUEST_OFFSET_CONST, rip).unwrap();
    // assembler.jmp(r15 +)
}

//
// pub struct NativeStack {
//     mmaped_stack_base: *mut c_void,
// }
//
// impl Drop for NativeStack {
//     fn drop(&mut self) {
//         todo!()
//     }
// }