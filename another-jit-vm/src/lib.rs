#![feature(asm_const)]
#![feature(trait_alias)]
#![feature(core_intrinsics)]
// methodid to code id mapping is handled seperately
// exit handling has registered handling but actual handling is seperate -
// have another layer above this which gets rid of native points and does everytthing in terms of IR
// have java layer above that

use std::arch::asm;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::intrinsics::copy_nonoverlapping;
use std::marker::PhantomData;
use std::ops::Range;
use std::ptr::{NonNull, null_mut};
use std::sync::RwLock;

use iced_x86::code_asm::{al, AsmRegister16, AsmRegister32, AsmRegister64, AsmRegister8, AsmRegisterMm, AsmRegisterXmm, bl, bx, cl, CodeAssembler, CodeLabel, cx, dl, dx, eax, ebx, ecx, edx, mm0, mm1, mm2, mm3, mm4, mm5, mm6, mm7, qword_ptr, r10, r10b, r10d, r10w, r11, r11b, r11d, r11w, r12, r12b, r12d, r12w, r13, r13b, r13d, r13w, r14, r14b, r14d, r14w, r15, r8, r8b, r8d, r8w, r9, r9b, r9d, r9w, rax, rbp, rbx, rcx, rdx, rsp, xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7};
use libc::{MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
use memoffset::offset_of;
use rangemap::RangeMap;

use crate::code_modification::CodeModificationHandle;
use crate::intrinsic_helpers::IntrinsicHelpers;
use crate::saved_registers_utils::{SavedRegistersWithIP, SavedRegistersWithIPDiff, SavedRegistersWithoutIP};
use crate::stack::OwnedNativeStack;

// todo this should really go elsewhere
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct IRMethodID(pub usize);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FramePointerOffset(pub usize);

pub const MAGIC_1_EXPECTED: u64 = 0xDEADBEEFDEADBEAF;
pub const MAGIC_2_EXPECTED: u64 = 0xDEADCAFEDEADDEAD;


pub mod stack;
pub mod saved_registers_utils;
pub mod code_modification;
pub mod intrinsic_helpers;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Register(pub u8);


impl Register {
    pub fn guest_offset_const(&self) -> usize {
        match self.0 {
            0 => RAX_GUEST_OFFSET_CONST,
            1 => RBX_GUEST_OFFSET_CONST,
            2 => RCX_GUEST_OFFSET_CONST,
            3 => RDX_GUEST_OFFSET_CONST,
            4 => R8_GUEST_OFFSET_CONST,
            5 => R9_GUEST_OFFSET_CONST,
            6 => R10_GUEST_OFFSET_CONST,
            7 => R11_GUEST_OFFSET_CONST,
            8 => R12_GUEST_OFFSET_CONST,
            9 => R13_GUEST_OFFSET_CONST,
            10 => R14_GUEST_OFFSET_CONST,
            _ => todo!(),
        }
    }

    pub fn to_native_64(&self) -> AsmRegister64 {
        match self.0 {
            0 => rax,
            1 => rbx,
            2 => rcx,
            3 => rdx,
            4 => r8,
            5 => r9,
            6 => r10,
            7 => r11,
            8 => r12,
            9 => r13,
            10 => r14,
            _ => todo!(),
        }
    }

    pub fn to_native_32(&self) -> AsmRegister32 {
        match self.0 {
            0 => eax,
            1 => ebx,
            2 => ecx,
            3 => edx,
            4 => r8d,
            5 => r9d,
            6 => r10d,
            7 => r11d,
            8 => r12d,
            9 => r13d,
            10 => r14d,
            _ => todo!(),
        }
    }
    pub fn to_native_16(&self) -> AsmRegister16 {
        match self.0 {
            0 => panic!(),
            1 => bx,
            2 => cx,
            3 => dx,
            4 => r8w,
            5 => r9w,
            6 => r10w,
            7 => r11w,
            8 => r12w,
            9 => r13w,
            10 => r14w,
            _ => todo!(),
        }
    }
    pub fn to_native_8(&self) -> AsmRegister8 {
        match self.0 {
            0 => al,
            1 => bl,
            2 => cl,
            3 => dl,
            4 => r8b,
            5 => r9b,
            6 => r10b,
            7 => r11b,
            8 => r12b,
            9 => r13b,
            10 => r14b,
            _ => todo!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FloatRegister(pub u8);

impl FloatRegister {
    pub fn to_xmm(&self) -> AsmRegisterXmm {
        match self.0 {
            0 => xmm0,
            1 => xmm1,
            2 => xmm2,
            3 => xmm3,
            4 => xmm4,
            5 => xmm5,
            6 => xmm6,
            7 => xmm7,
            _ => todo!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DoubleRegister(pub u8);

impl DoubleRegister {
    pub fn to_xmm(&self) -> AsmRegisterXmm {
        match self.0 {
            0 => xmm0,
            1 => xmm1,
            2 => xmm2,
            3 => xmm3,
            4 => xmm4,
            5 => xmm5,
            6 => xmm6,
            7 => xmm7,
            _ => todo!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MMRegister(pub u8);

impl MMRegister {
    pub fn to_mm(&self) -> AsmRegisterMm {
        match self.0 {
            0 => mm0,
            1 => mm1,
            2 => mm2,
            3 => mm3,
            4 => mm4,
            5 => mm5,
            6 => mm6,
            7 => mm7,
            _ => todo!()
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MethodImplementationID(usize);

pub struct MethodOffset(usize);

pub struct VMStateInner<'vm, T: Sized>/*<'vm_state_life, T: Sized, ExtraData: 'vm_state_life>*/ {
    method_id_max: MethodImplementationID,
    code_regions: HashMap<MethodImplementationID, Range<*const c_void>>,
    code_regions_to_method: RangeMap<*const c_void, MethodImplementationID>,
    max_ptr: *mut c_void,
    phantom_nightly_compiler_bug_workaround: PhantomData<&'vm ()>,
    phantom_2: PhantomData<&'vm T>,
}

pub struct VMState<'vm, T: Sized> {
    inner: RwLock<VMStateInner<'vm, T>>,
    //should be per thread
    mmaped_code_region_base: *mut c_void,
    mmaped_code_size: usize,
}

impl<'vm, T> VMState<'vm, T> {
    pub fn lookup_method_addresses(&self, method_implementation_id: MethodImplementationID) -> Range<*const c_void> {
        self.inner.read().unwrap().code_regions.get(&method_implementation_id).unwrap().clone()
    }

    pub fn lookup_ip(&self, ip: *const c_void) -> MethodImplementationID {
        *self.inner.read().unwrap().code_regions_to_method.get(&ip).unwrap()
    }
}

impl<T> Drop for VMState<'_, T> {
    fn drop(&mut self) {
        let res = unsafe { libc::munmap(self.mmaped_code_region_base, self.mmaped_code_size) };
        if res != 0 {
            panic!();
        }
    }
}

#[derive(Clone)]
pub struct VMExitEvent {
    // pub method: MethodImplementationID,
    // pub method_base_address: *const c_void,
    pub saved_guest_registers: SavedRegistersWithIP,
    correctly_exited: bool,
}

impl Drop for VMExitEvent {
    fn drop(&mut self) {
        if !self.correctly_exited {
            panic!("Did not handle the vm exit")
        }
    }
}

impl VMExitEvent {
    pub fn indicate_okay_to_drop(&mut self) {
        self.correctly_exited = true;
    }
}

pub enum VMExitAction<T: Sized> {
    ExitVMCompletely { return_data: T },
    ReturnTo { return_register_state: SavedRegistersWithIPDiff },
}

#[repr(C)]
struct JITContext {
    guest_registers: SavedRegistersWithIP,
    vm_native_saved_registers: SavedRegistersWithIP,
    intrinsic_helpers: IntrinsicHelpers,
}

trait ExitHandlerType<'vm, ExtraData, T> = Fn(&VMExitEvent, &mut OwnedNativeStack, &mut ExtraData) -> VMExitAction<T> + 'vm;

pub struct LaunchedVM<'vm, 'l, T> {
    vm_state: &'l VMState<'vm, T>,
    jit_context: JITContext,
    stack_top: NonNull<c_void>,
    stack_bottom: NonNull<c_void>,
    pending_exit: bool,
}

impl<'vm, 'extra_data_life, T> Iterator for LaunchedVM<'vm, '_, T> {
    type Item = VMExitEvent;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(!self.pending_exit);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rbp as *mut c_void);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rsp as *mut c_void);
        let vm_exit_event = self.vm_state.run_method_impl(&mut self.jit_context);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rbp as *mut c_void);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rsp as *mut c_void);
        self.pending_exit = true;
        Some(vm_exit_event)
    }
}

impl<'vm, 'extra_data_life, T> LaunchedVM<'vm, '_, T> {
    pub fn return_to(&mut self, mut event: VMExitEvent, return_register_state: SavedRegistersWithIPDiff) {
        assert!(self.pending_exit);
        self.pending_exit = false;
        event.correctly_exited = true;
        self.jit_context.guest_registers.apply_diff(return_register_state);
    }

    fn validate_stack_ptr(&self, ptr: *mut c_void) {
        let ptr = NonNull::new(ptr).unwrap();
        assert!((self.stack_top >= ptr && self.stack_bottom <= ptr));
    }
}

impl<'vm, T> VMState<'vm, T> {
    //don't store exit type in here, that can go in register or derive from ip, include base method address in  event
    pub fn new() -> Self {
        const DEFAULT_CODE_SIZE: usize = 1024 * 1024 * 1024;
        unsafe {
            let mmaped_code_region_base = libc::mmap(null_mut(), DEFAULT_CODE_SIZE, PROT_READ | PROT_WRITE | PROT_EXEC, MAP_ANONYMOUS | MAP_PRIVATE | MAP_NORESERVE, -1, 0) as *mut c_void;
            VMState {
                inner: RwLock::new(VMStateInner {
                    method_id_max: MethodImplementationID(0),
                    code_regions: Default::default(),
                    code_regions_to_method: Default::default(),
                    max_ptr: mmaped_code_region_base,
                    phantom_nightly_compiler_bug_workaround: Default::default(),
                    phantom_2: Default::default(),
                }),
                mmaped_code_region_base,
                mmaped_code_size: DEFAULT_CODE_SIZE,
            }
        }
    }

    pub fn launch_vm<'l, 'stack_life, 'extra_data>(&'l self, stack: &'stack_life OwnedNativeStack, method_id: MethodImplementationID, initial_registers: SavedRegistersWithoutIP) -> LaunchedVM<'vm, 'l, T> {
        let inner_guard = self.inner.read().unwrap();
        let code_region: Range<*const c_void> = inner_guard.code_regions.get(&method_id).unwrap().clone();
        let branch_to = code_region.start;
        drop(inner_guard);
        let rip_guest_offset = offset_of!(SavedRegistersWithIP, rip) + offset_of!(JITContext, guest_registers);
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
        let rip_native_offset = offset_of!(SavedRegistersWithIP, rip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rip_native_offset, RIP_NATIVE_OFFSET_CONST);
        let rax_native_offset = offset_of!(SavedRegistersWithoutIP, rax) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rax_native_offset, RAX_NATIVE_OFFSET_CONST);
        let rbx_native_offset = offset_of!(SavedRegistersWithoutIP, rbx) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rbx_native_offset, RBX_NATIVE_OFFSET_CONST);
        let rcx_native_offset = offset_of!(SavedRegistersWithoutIP, rcx) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rcx_native_offset, RCX_NATIVE_OFFSET_CONST);
        let rdx_native_offset = offset_of!(SavedRegistersWithoutIP, rdx) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rdx_native_offset, RDX_NATIVE_OFFSET_CONST);
        let rsi_native_offset = offset_of!(SavedRegistersWithoutIP, rsi) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rsi_native_offset, RSI_NATIVE_OFFSET_CONST);
        let rdi_native_offset = offset_of!(SavedRegistersWithoutIP, rdi) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rdi_native_offset, RDI_NATIVE_OFFSET_CONST);
        let rbp_native_offset = offset_of!(SavedRegistersWithoutIP, rbp) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rbp_native_offset, RBP_NATIVE_OFFSET_CONST);
        let rsp_native_offset = offset_of!(SavedRegistersWithoutIP, rsp) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(rsp_native_offset, RSP_NATIVE_OFFSET_CONST);
        let r8_native_offset = offset_of!(SavedRegistersWithoutIP, r8) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r8_native_offset, R8_NATIVE_OFFSET_CONST);
        let r9_native_offset = offset_of!(SavedRegistersWithoutIP, r9) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r9_native_offset, R9_NATIVE_OFFSET_CONST);
        let r10_native_offset = offset_of!(SavedRegistersWithoutIP, r10) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r10_native_offset, R10_NATIVE_OFFSET_CONST);
        let r11_native_offset = offset_of!(SavedRegistersWithoutIP, r11) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r11_native_offset, R11_NATIVE_OFFSET_CONST);
        let r12_native_offset = offset_of!(SavedRegistersWithoutIP, r12) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r12_native_offset, R12_NATIVE_OFFSET_CONST);
        let r13_native_offset = offset_of!(SavedRegistersWithoutIP, r13) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r13_native_offset, R13_NATIVE_OFFSET_CONST);
        let r14_native_offset = offset_of!(SavedRegistersWithoutIP, r14) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(r14_native_offset, R14_NATIVE_OFFSET_CONST);
        let xsave_area_native_offset = offset_of!(SavedRegistersWithoutIP, xsave_area) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
        assert_eq!(xsave_area_native_offset, XSAVE_AREA_NATIVE_OFFSET_CONST);
        let jit_context = JITContext {
            guest_registers: SavedRegistersWithIP { rip: branch_to as *mut c_void, saved_registers_without_ip: initial_registers },
            vm_native_saved_registers: SavedRegistersWithIP {
                rip: null_mut(),
                saved_registers_without_ip: SavedRegistersWithoutIP {
                    rax: 0,
                    rbx: 0,
                    rcx: 0,
                    rdx: 0,
                    rsi: 0,
                    rdi: 0,
                    rbp: 0,
                    rsp: 0,
                    r8: 0,
                    r9: 0,
                    r10: 0,
                    r11: 0,
                    r12: 0,
                    r13: 0,
                    r14: 0,
                    xsave_area: [0; 64],
                },
            },
            intrinsic_helpers: IntrinsicHelpers::new(),
        };
        let self_: &'l VMState<'vm, T> = self;
        let iterator: LaunchedVM<'vm, 'l, T> = LaunchedVM { vm_state: self_, jit_context, stack_top: stack.mmaped_top, stack_bottom: stack.mmaped_bottom, pending_exit: false };
        // eprintln!("==== VM Start ====");
        iterator
    }
}

impl<'vm, T> VMState<'vm, T> {
    #[allow(named_asm_labels)]
    #[inline(never)]
    fn run_method_impl(&self, jit_context: &mut JITContext) -> VMExitEvent {
        // unsafe {
        //     if GOING_IN_COUNT == 5 {
        //         eprintln!("here")
        //     }
        //     GOING_IN_COUNT += 1;
        // }
        // eprintln!("GOING IN AT: rbp:{:?} rsp:{:?} rip:{:?}",
        //           jit_context.guest_registers.saved_registers_without_ip.rbp,
        //           jit_context.guest_registers.saved_registers_without_ip.rsp, jit_context.guest_registers.rip);
        let jit_context_pointer = jit_context as *mut JITContext as *mut c_void;
        unsafe {
            asm!(
            "push r15",
            //save all registers to avoid breaking stuff
            "mov r15, {0}",
            // "mov [r15 + {__rust_jvm_rax_native_offset_const}], rax",
            "mov [r15 + {__rust_jvm_rbx_native_offset_const}], rbx",// llvm doesn't like out rb for some reason
            // "mov [r15 + {__rust_jvm_rcx_native_offset_const}], rcx",
            // "mov [r15 + {__rust_jvm_rdx_native_offset_const}], rdx",
            // "mov [r15 + {__rust_jvm_rsi_native_offset_const}], rsi",
            // "mov [r15 + {__rust_jvm_rdi_native_offset_const}], rdi",
            "mov [r15 + {__rust_jvm_rbp_native_offset_const}], rbp",
            "mov [r15 + {__rust_jvm_rsp_native_offset_const}], rsp",
            // "mov [r15 + {__rust_jvm_r8_native_offset_const}], r8",
            // "mov [r15 + {__rust_jvm_r9_native_offset_const}], r9",
            // "mov [r15 + {__rust_jvm_r10_native_offset_const}], r10",
            // "mov [r15 + {__rust_jvm_r11_native_offset_const}], r11",
            // "mov [r15 + {__rust_jvm_r12_native_offset_const}], r12",
            // "mov [r15 + {__rust_jvm_r13_native_offset_const}], r13",
            // "mov [r15 + {__rust_jvm_r14_native_offset_const}], r14",
            // "xsave [r15 + {__rust_jvm_xsave_area_native_offset_const}]",
            "lea rax, [rip+__rust_jvm_internal_after_enter]",
            "mov [r15 + {__rust_jvm_rip_native_offset_const}], rax",
            //load expected register values
            "mov rax,[r15 + {__rust_jvm_rax_guest_offset_const}]",
            "mov rbx,[r15 + {__rust_jvm_rbx_guest_offset_const}]",
            "mov rcx,[r15 + {__rust_jvm_rcx_guest_offset_const}]",
            "mov rdx,[r15 + {__rust_jvm_rdx_guest_offset_const}]",
            "mov rsi,[r15 + {__rust_jvm_rsi_guest_offset_const}]",
            "mov rdi,[r15 + {__rust_jvm_rdi_guest_offset_const}]",
            "mov rbp,[r15 + {__rust_jvm_rbp_guest_offset_const}]",
            "mov rsp,[r15 + {__rust_jvm_rsp_guest_offset_const}]",
            "mov r8,[r15 + {__rust_jvm_r8_guest_offset_const}]",
            "mov r9,[r15 + {__rust_jvm_r9_guest_offset_const}]",
            "mov r10,[r15 + {__rust_jvm_r10_guest_offset_const}]",
            "mov r11,[r15 + {__rust_jvm_r11_guest_offset_const}]",
            "mov r12,[r15 + {__rust_jvm_r12_guest_offset_const}]",
            "mov r13,[r15 + {__rust_jvm_r13_guest_offset_const}]",
            "mov r14,[r15 + {__rust_jvm_r14_guest_offset_const}]",
            // "xrstor [r15 + {__rust_jvm_xsave_area_guest_offset_const}]",
            "jmp qword ptr [r15 + {__rust_jvm_rip_guest_offset_const}]",
            "__rust_jvm_internal_after_enter:",
            // "mov rax, [r15 + {__rust_jvm_rax_native_offset_const}]",
            "mov rbx, [r15 + {__rust_jvm_rbx_native_offset_const}]",
            // "mov rcx, [r15 + {__rust_jvm_rcx_native_offset_const}]",
            // "mov rdx, [r15 + {__rust_jvm_rdx_native_offset_const}]",
            // "mov rsi, [r15 + {__rust_jvm_rsi_native_offset_const}]",
            // "mov rdi, [r15 + {__rust_jvm_rdi_native_offset_const}]",
            "mov rbp, [r15 + {__rust_jvm_rbp_native_offset_const}]",
            "mov rsp, [r15 + {__rust_jvm_rsp_native_offset_const}]",
            // "mov r8, [r15 + {__rust_jvm_r8_native_offset_const}]",
            // "mov r9, [r15 + {__rust_jvm_r9_native_offset_const}]",
            // "mov r10, [r15 + {__rust_jvm_r10_native_offset_const}]",
            // "mov r11, [r15 + {__rust_jvm_r11_native_offset_const}]",
            // "mov r12, [r15 + {__rust_jvm_r12_native_offset_const}]",
            // "mov r13, [r15 + {__rust_jvm_r13_native_offset_const}]",
            // "mov r14, [r15 + {__rust_jvm_r14_native_offset_const}]",
            "pop r15",
            in(reg) jit_context_pointer,
            __rust_jvm_rip_guest_offset_const = const RIP_GUEST_OFFSET_CONST,
            __rust_jvm_rax_guest_offset_const = const RAX_GUEST_OFFSET_CONST,
            __rust_jvm_rbx_guest_offset_const = const RBX_GUEST_OFFSET_CONST,
            __rust_jvm_rcx_guest_offset_const = const RCX_GUEST_OFFSET_CONST,
            __rust_jvm_rdx_guest_offset_const = const RDX_GUEST_OFFSET_CONST,
            __rust_jvm_rsi_guest_offset_const = const RSI_GUEST_OFFSET_CONST,
            __rust_jvm_rdi_guest_offset_const = const RDI_GUEST_OFFSET_CONST,
            __rust_jvm_rbp_guest_offset_const = const RBP_GUEST_OFFSET_CONST,
            __rust_jvm_rsp_guest_offset_const = const RSP_GUEST_OFFSET_CONST,
            __rust_jvm_r8_guest_offset_const = const R8_GUEST_OFFSET_CONST,
            __rust_jvm_r9_guest_offset_const = const R9_GUEST_OFFSET_CONST,
            __rust_jvm_r10_guest_offset_const = const R10_GUEST_OFFSET_CONST,
            __rust_jvm_r11_guest_offset_const = const R11_GUEST_OFFSET_CONST,
            __rust_jvm_r12_guest_offset_const = const R12_GUEST_OFFSET_CONST,
            __rust_jvm_r13_guest_offset_const = const R13_GUEST_OFFSET_CONST,
            __rust_jvm_r14_guest_offset_const = const R14_GUEST_OFFSET_CONST,
            // __rust_jvm_xsave_area_guest_offset_const = const XSAVE_AREA_GUEST_OFFSET_CONST,
            __rust_jvm_rip_native_offset_const = const RIP_NATIVE_OFFSET_CONST,
            // __rust_jvm_rax_native_offset_const = const RAX_NATIVE_OFFSET_CONST,
            __rust_jvm_rbx_native_offset_const = const RBX_NATIVE_OFFSET_CONST,
            // __rust_jvm_rcx_native_offset_const = const RCX_NATIVE_OFFSET_CONST,
            // __rust_jvm_rdx_native_offset_const = const RDX_NATIVE_OFFSET_CONST,
            // __rust_jvm_rsi_native_offset_const = const RSI_NATIVE_OFFSET_CONST,
            // __rust_jvm_rdi_native_offset_const = const RDI_NATIVE_OFFSET_CONST,
            __rust_jvm_rbp_native_offset_const = const RBP_NATIVE_OFFSET_CONST,
            __rust_jvm_rsp_native_offset_const = const RSP_NATIVE_OFFSET_CONST,
            // __rust_jvm_r8_native_offset_const = const R8_NATIVE_OFFSET_CONST,
            // __rust_jvm_r9_native_offset_const = const R9_NATIVE_OFFSET_CONST,
            // __rust_jvm_r10_native_offset_const = const R10_NATIVE_OFFSET_CONST,
            // __rust_jvm_r11_native_offset_const = const R11_NATIVE_OFFSET_CONST,
            // __rust_jvm_r12_native_offset_const = const R12_NATIVE_OFFSET_CONST,
            // __rust_jvm_r13_native_offset_const = const R13_NATIVE_OFFSET_CONST,
            // __rust_jvm_r14_native_offset_const = const R14_NATIVE_OFFSET_CONST,
            // __rust_jvm_xsave_area_native_offset_const = const XSAVE_AREA_NATIVE_OFFSET_CONST,
            out("ymm0") _,
            out("ymm1") _,
            out("ymm2") _,
            out("ymm3") _,
            out("ymm4") _,
            out("ymm5") _,
            out("ymm6") _,
            out("ymm8") _,
            out("ymm9") _,
            out("ymm10") _,
            out("ymm11") _,
            out("ymm12") _,
            out("ymm13") _,
            out("ymm14") _,
            out("ymm15") _,
            out("rax") _,
            // out("rbx") _,
            out("rcx") _,
            out("rdx") _,
            out("rsi") _,
            out("rdi") _,
            out("r8") _,
            out("r9") _,
            out("r10") _,
            out("r11") _,
            out("r12") _,
            out("r13") _,
            out("r14") _,
            // out("ymm14") _,
            // out("ymm15") _,
            )
        }
        // eprintln!("GOING OUT AT: rbp:{:?} rsp:{:?} rip:{:?}", jit_context.guest_registers.saved_registers_without_ip.rbp, jit_context.guest_registers.saved_registers_without_ip.rsp, jit_context.guest_registers.rip);
        self.generate_exit_event(jit_context.guest_registers/*, extra*/)
    }

    fn generate_exit_event(&self, guest_registers: SavedRegistersWithIP/*, extra: &mut ExtraData*/) -> VMExitEvent {
        // let inner_read_guard = self.inner.read().unwrap();
        // let method_implementation = inner_read_guard.code_regions_to_method.get(&guest_rip);
        // match method_implementation {
        // None => {
        //     todo!()
        // }
        // Some(method_implementation) => {
        // let method_implementation = *method_implementation;
        VMExitEvent {
            // method: method_implementation,
            // method_base_address: inner_read_guard.code_regions.get(&method_implementation).unwrap().start,
            saved_guest_registers: guest_registers,
            correctly_exited: false,
        }
        // }
        // }
    }

    pub fn gen_vm_exit(assembler: &mut CodeAssembler, before_exit_label: &mut CodeLabel, after_exit_label: &mut CodeLabel, registers_to_save: HashSet<Register>) {
        assembler.set_label(before_exit_label).unwrap();
        for register in registers_to_save {
            assembler.mov(r15 + register.guest_offset_const(), register.to_native_64()).unwrap();
        }
        assembler.mov(r15 + RBP_GUEST_OFFSET_CONST, rbp).unwrap();
        assembler.mov(r15 + RSP_GUEST_OFFSET_CONST, rsp).unwrap();
        assembler.lea(r10, qword_ptr(*before_exit_label)).unwrap();//safe to clober r10 b/c it was saved
        assembler.mov(r15 + RIP_GUEST_OFFSET_CONST, r10).unwrap();
        assembler.jmp(qword_ptr(r15 + RIP_NATIVE_OFFSET_CONST)).unwrap();
        assembler.set_label(after_exit_label).unwrap();
    }

    pub fn get_new_base_address(&self) -> BaseAddress {
        BaseAddress(self.inner.read().unwrap().max_ptr)
    }

    pub fn add_method_implementation(&self, code: Vec<u8>, base_address: BaseAddress, code_modification_handle: CodeModificationHandle) -> MethodImplementationID {
        let mut inner_guard = self.inner.write().unwrap();
        let current_method_id = inner_guard.method_id_max;
        inner_guard.method_id_max.0 += 1;
        let new_method_base = inner_guard.max_ptr;
        assert_eq!(base_address.0, new_method_base);
        let code_len = code.len();
        let end_of_new_method = unsafe {
            new_method_base.add(code_len)
        };
        let method_range = (new_method_base as *const c_void)..(end_of_new_method as *const c_void);
        inner_guard.code_regions.insert(current_method_id, method_range.clone());
        inner_guard.code_regions_to_method.insert(method_range, current_method_id);
        inner_guard.max_ptr = end_of_new_method;
        unsafe { copy_nonoverlapping(code.as_ptr() as *const c_void, new_method_base as *mut c_void, code_len); }
        drop(code_modification_handle);
        current_method_id
    }
}

#[must_use]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BaseAddress(pub *const c_void);

pub struct VMExitLabel {
    pub before_exit_label: CodeLabel,
    pub after_exit_label: CodeLabel,
}

pub struct Method {
    pub code: Vec<u8>,
}


pub const RIP_GUEST_OFFSET_CONST: usize = 0;
#[allow(clippy::identity_op)]
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

pub const XSAVE_SIZE: usize = 64 * 8;

#[allow(clippy::identity_op)]
pub const RIP_NATIVE_OFFSET_CONST: usize = 0 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 0;
#[allow(clippy::identity_op)]
pub const RAX_NATIVE_OFFSET_CONST: usize = 0 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RBX_NATIVE_OFFSET_CONST: usize = 8 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RCX_NATIVE_OFFSET_CONST: usize = 16 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RDX_NATIVE_OFFSET_CONST: usize = 24 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RSI_NATIVE_OFFSET_CONST: usize = 32 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RDI_NATIVE_OFFSET_CONST: usize = 40 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RBP_NATIVE_OFFSET_CONST: usize = 48 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const RSP_NATIVE_OFFSET_CONST: usize = 56 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R8_NATIVE_OFFSET_CONST: usize = 64 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R9_NATIVE_OFFSET_CONST: usize = 72 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R10_NATIVE_OFFSET_CONST: usize = 80 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R11_NATIVE_OFFSET_CONST: usize = 88 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R12_NATIVE_OFFSET_CONST: usize = 96 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R13_NATIVE_OFFSET_CONST: usize = 104 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const R14_NATIVE_OFFSET_CONST: usize = 112 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 8;
pub const XSAVE_AREA_NATIVE_OFFSET_CONST: usize = XSAVE_AREA_GUEST_OFFSET_CONST + 120 + XSAVE_SIZE + 8;


pub struct NativeInstructionLocation(pub *const c_void);