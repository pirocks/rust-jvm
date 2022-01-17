#![feature(asm)]
#![feature(asm_const)]
#![feature(backtrace)]
#![feature(trait_alias)]
#![feature(in_band_lifetimes)]
#![feature(generic_associated_types)]
// save all registers when entering and exiting vm -
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
use std::ptr::null_mut;
use std::sync::RwLock;

use iced_x86::code_asm::{AsmRegister32, AsmRegister64, CodeAssembler, CodeLabel, ebx, ecx, edx, qword_ptr, r10, r10d, r11, r11d, r12, r12d, r13, r13d, r14, r14d, r15, r8, r8d, r9, r9d, rax, rbp, rbx, rcx, rdx, rsp};
use libc::{MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
use memoffset::offset_of;
use rangemap::RangeMap;

use crate::stack::OwnedNativeStack;

pub mod stack;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Register(pub u8);


impl Register {
    pub fn guest_offset_const(&self) -> usize {
        match self.0 {
            0 => panic!(),
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
            0 => panic!(),
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
            0 => panic!(),
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
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MethodImplementationID(usize);

pub struct MethodOffset(usize);

pub struct VMStateInner<'vm_life, T: Sized, ExtraData: 'vm_life>/*<'vm_state_life, T: Sized, ExtraData: 'vm_state_life>*/ {
    method_id_max: MethodImplementationID,
    code_regions: HashMap<MethodImplementationID, Range<*const c_void>>,
    code_regions_to_method: RangeMap<*const c_void, MethodImplementationID>,
    max_ptr: *mut c_void,
    phantom_nightly_compiler_bug_workaround: PhantomData<&'vm_life ()>,
    phantom_2: PhantomData<&'vm_life T>,
    phantom_3: PhantomData<&'vm_life ExtraData>,
}

pub struct VMState<'vm_life, T: Sized, ExtraData> {
    inner: RwLock<VMStateInner<'vm_life, T, ExtraData>>,
    //should be per thread
    mmaped_code_region_base: *mut c_void,
    mmaped_code_size: usize,
}

impl<'vm_life, T, ExtraData> VMState<'vm_life, T, ExtraData> {
    pub fn lookup_method_addresses(&self, method_implementation_id: MethodImplementationID) -> Range<*const c_void> {
        self.inner.read().unwrap().code_regions.get(&method_implementation_id).unwrap().clone()
    }
}

impl<T, ExtraData> Drop for VMState<'_, T, ExtraData> {
    fn drop(&mut self) {
        let res = unsafe { libc::munmap(self.mmaped_code_region_base, self.mmaped_code_size) };
        if res != 0 {
            panic!();
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SavedRegistersWithIP {
    pub rip: *const c_void,
    pub saved_registers_without_ip: SavedRegistersWithoutIP,
}

impl SavedRegistersWithIP {
    pub fn apply_diff(&mut self, diff: SavedRegistersWithIPDiff) {
        let SavedRegistersWithIPDiff { rip, saved_registers_without_ip } = diff;
        if let Some(rip) = rip {
            self.rip = rip;
        }
        if let Some(_saved_registers_without_ip) = saved_registers_without_ip {
            todo!()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SavedRegistersWithoutIP {
    pub rax: *mut c_void,
    pub rbx: *mut c_void,
    pub rcx: *mut c_void,
    pub rdx: *mut c_void,
    pub rsi: *mut c_void,
    pub rdi: *mut c_void,
    pub rbp: *mut c_void,
    pub rsp: *mut c_void,
    pub r8: *mut c_void,
    pub r9: *mut c_void,
    pub r10: *mut c_void,
    pub r11: *mut c_void,
    pub r12: *mut c_void,
    pub r13: *mut c_void,
    pub r14: *mut c_void,
    pub xsave_area: [u64; 64],
}

impl SavedRegistersWithoutIP {
    pub fn new_with_all_zero() -> Self {
        Self {
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
        }
    }

    pub fn get_register(&self, register: Register) -> u64 {
        (match register.0 {
            0 => self.rax,
            1 => self.rbx,
            2 => self.rcx,
            3 => self.rdx,
            4 => self.r8,
            5 => self.r9,
            6 => self.r10,
            7 => self.r11,
            8 => self.r12,
            9 => self.r13,
            10 => self.r14,
            _ => todo!()
        }) as u64
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithIPDiff {
    pub rip: Option<*const c_void>,
    pub saved_registers_without_ip: Option<SavedRegistersWithoutIPDiff>,
}

#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithoutIPDiff {
    pub rax: Option<*mut c_void>,
    pub rbx: Option<*mut c_void>,
    pub rcx: Option<*mut c_void>,
    pub rdx: Option<*mut c_void>,
    pub rsi: Option<*mut c_void>,
    pub rdi: Option<*mut c_void>,
    pub rbp: Option<*mut c_void>,
    pub rsp: Option<*mut c_void>,
    pub r8: Option<*mut c_void>,
    pub r9: Option<*mut c_void>,
    pub r10: Option<*mut c_void>,
    pub r11: Option<*mut c_void>,
    pub r12: Option<*mut c_void>,
    pub r13: Option<*mut c_void>,
    pub r14: Option<*mut c_void>,
    pub xsave_area: Option<[u64; 64]>,
}

#[derive(Clone)]
pub struct VMExitEvent {
    pub method: MethodImplementationID,
    pub method_base_address: *const c_void,
    pub saved_guest_registers: SavedRegistersWithIP,
    correctly_exited: bool
}

impl Drop for VMExitEvent{
    fn drop(&mut self) {
        if !self.correctly_exited{
            panic!("Did not handle the vm exit")
        }
    }
}

impl VMExitEvent{
    pub fn indicate_okay_to_drop(&mut self){
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
}

trait ExitHandlerType<'vm_life, ExtraData, T> = Fn(&VMExitEvent, &mut OwnedNativeStack, &mut ExtraData) -> VMExitAction<T> + 'vm_life;

pub struct LaunchedVM<'vm_life, 'extra_data_life, 'l, T, ExtraData: 'vm_life> {
    vm_state: &'l VMState<'vm_life, T, ExtraData>,
    jit_context: JITContext,
    stack_top: *const c_void,
    stack_bottom:*const c_void,
    pub extra: &'extra_data_life mut ExtraData,
    pending_exit: bool
}

impl<'vm_life, 'extra_data_life, T, ExtraData: 'vm_life> Iterator for LaunchedVM<'vm_life, 'extra_data_life, '_, T, ExtraData> {
    type Item = VMExitEvent;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(!self.pending_exit);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rbp);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rsp);
        let vm_exit_event = self.vm_state.run_method_impl(&mut self.jit_context);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rbp);
        self.validate_stack_ptr(self.jit_context.guest_registers.saved_registers_without_ip.rsp);
        self.pending_exit = true;
        Some(vm_exit_event)
    }
}

impl<'vm_life, 'extra_data_life, T, ExtraData: 'vm_life> LaunchedVM<'vm_life, 'extra_data_life, '_, T, ExtraData> {
    pub fn return_to(&mut self, mut event: VMExitEvent, return_register_state: SavedRegistersWithIPDiff){
        assert!(self.pending_exit);
        self.pending_exit = false;
        event.correctly_exited = true;
        self.jit_context.guest_registers.apply_diff(return_register_state);
    }

    fn validate_stack_ptr(&self, ptr: *const c_void){
        assert!((self.stack_top >= ptr && self.stack_bottom <= ptr));
    }
}

impl<'vm_life, T, ExtraData> VMState<'vm_life, T, ExtraData> {
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
                    phantom_3: Default::default()
                }),
                mmaped_code_region_base,
                mmaped_code_size: DEFAULT_CODE_SIZE,
            }
        }
    }

    pub fn launch_vm(&'l self, stack: &'stack_life OwnedNativeStack, method_id: MethodImplementationID, initial_registers: SavedRegistersWithoutIP, extra: &'extra_data mut ExtraData) -> LaunchedVM<'vm_life, 'extra_data, 'l, T, ExtraData> {
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
        let self_: &'l VMState<'vm_life, T, ExtraData> = self;
        let iterator: LaunchedVM<'vm_life, '_, 'l, T, ExtraData> = LaunchedVM { vm_state: self_, jit_context, stack_top: stack.mmaped_top, stack_bottom: stack.mmaped_bottom, extra, pending_exit: false };
        eprintln!("==== VM Start ====");
        return iterator;
    }

    #[allow(named_asm_labels)]
    fn run_method_impl(&self, jit_context: &mut JITContext) -> VMExitEvent {
        // eprintln!("{}",Backtrace::capture().to_string());
        eprintln!("GOING IN AT: rbp:{:?} rsp:{:?} rip:{:?}",
                  jit_context.guest_registers.saved_registers_without_ip.rbp,
                  jit_context.guest_registers.saved_registers_without_ip.rsp, jit_context.guest_registers.rip);
        let jit_context_pointer = jit_context as *mut JITContext as *mut c_void;
        unsafe {
            asm!(
            //save all registers to avoid breaking stuff
            "mov r15, {0}",
            "mov [r15 + {__rust_jvm_rax_native_offset_const}], rax",
            "mov [r15 + {__rust_jvm_rbx_native_offset_const}], rbx",
            "mov [r15 + {__rust_jvm_rcx_native_offset_const}], rcx",
            "mov [r15 + {__rust_jvm_rdx_native_offset_const}], rdx",
            "mov [r15 + {__rust_jvm_rsi_native_offset_const}], rsi",
            "mov [r15 + {__rust_jvm_rdi_native_offset_const}], rdi",
            "mov [r15 + {__rust_jvm_rbp_native_offset_const}], rbp",
            "mov [r15 + {__rust_jvm_rsp_native_offset_const}], rsp",
            "mov [r15 + {__rust_jvm_r8_native_offset_const}], r8",
            "mov [r15 + {__rust_jvm_r9_native_offset_const}], r9",
            "mov [r15 + {__rust_jvm_r10_native_offset_const}], r10",
            "mov [r15 + {__rust_jvm_r11_native_offset_const}], r11",
            "mov [r15 + {__rust_jvm_r12_native_offset_const}], r12",
            "mov [r15 + {__rust_jvm_r13_native_offset_const}], r13",
            "mov [r15 + {__rust_jvm_r14_native_offset_const}], r14",
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
            "mov rax, [r15 + {__rust_jvm_rax_native_offset_const}]",
            "mov rbx, [r15 + {__rust_jvm_rbx_native_offset_const}]",
            "mov rcx, [r15 + {__rust_jvm_rcx_native_offset_const}]",
            "mov rdx, [r15 + {__rust_jvm_rdx_native_offset_const}]",
            "mov rsi, [r15 + {__rust_jvm_rsi_native_offset_const}]",
            "mov rdi, [r15 + {__rust_jvm_rdi_native_offset_const}]",
            "mov rbp, [r15 + {__rust_jvm_rbp_native_offset_const}]",
            "mov rsp, [r15 + {__rust_jvm_rsp_native_offset_const}]",
            "mov r8, [r15 + {__rust_jvm_r8_native_offset_const}]",
            "mov r9, [r15 + {__rust_jvm_r9_native_offset_const}]",
            "mov r10, [r15 + {__rust_jvm_r10_native_offset_const}]",
            "mov r11, [r15 + {__rust_jvm_r11_native_offset_const}]",
            "mov r12, [r15 + {__rust_jvm_r12_native_offset_const}]",
            "mov r13, [r15 + {__rust_jvm_r13_native_offset_const}]",
            "mov r14, [r15 + {__rust_jvm_r14_native_offset_const}]",
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
            __rust_jvm_rax_native_offset_const = const RAX_NATIVE_OFFSET_CONST,
            __rust_jvm_rbx_native_offset_const = const RBX_NATIVE_OFFSET_CONST,
            __rust_jvm_rcx_native_offset_const = const RCX_NATIVE_OFFSET_CONST,
            __rust_jvm_rdx_native_offset_const = const RDX_NATIVE_OFFSET_CONST,
            __rust_jvm_rsi_native_offset_const = const RSI_NATIVE_OFFSET_CONST,
            __rust_jvm_rdi_native_offset_const = const RDI_NATIVE_OFFSET_CONST,
            __rust_jvm_rbp_native_offset_const = const RBP_NATIVE_OFFSET_CONST,
            __rust_jvm_rsp_native_offset_const = const RSP_NATIVE_OFFSET_CONST,
            __rust_jvm_r8_native_offset_const = const R8_NATIVE_OFFSET_CONST,
            __rust_jvm_r9_native_offset_const = const R9_NATIVE_OFFSET_CONST,
            __rust_jvm_r10_native_offset_const = const R10_NATIVE_OFFSET_CONST,
            __rust_jvm_r11_native_offset_const = const R11_NATIVE_OFFSET_CONST,
            __rust_jvm_r12_native_offset_const = const R12_NATIVE_OFFSET_CONST,
            __rust_jvm_r13_native_offset_const = const R13_NATIVE_OFFSET_CONST,
            __rust_jvm_r14_native_offset_const = const R14_NATIVE_OFFSET_CONST,
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
            )
        }
        eprintln!("GOING OUT AT: rbp:{:?} rsp:{:?} rip:{:?}", jit_context.guest_registers.saved_registers_without_ip.rbp, jit_context.guest_registers.saved_registers_without_ip.rsp, jit_context.guest_registers.rip);
        self.generate_exit_event(jit_context.guest_registers.rip, jit_context.guest_registers/*, extra*/)
    }

    fn generate_exit_event(&self, guest_rip: *const c_void, guest_registers: SavedRegistersWithIP/*, extra: &mut ExtraData*/) -> VMExitEvent {
        let inner_read_guard = self.inner.read().unwrap();
        let method_implementation = inner_read_guard.code_regions_to_method.get(&guest_rip);
        match method_implementation {
            None => {
                todo!()
            }
            Some(method_implementation) => {
                let method_implementation = *method_implementation;
                let vm_exit_event = VMExitEvent {
                    method: method_implementation,
                    method_base_address: inner_read_guard.code_regions.get(&method_implementation).unwrap().start,
                    saved_guest_registers: guest_registers,
                    correctly_exited: false
                };
                vm_exit_event
            }
        }
    }

    pub fn gen_vm_exit(assembler: &mut CodeAssembler, before_exit_label: &mut CodeLabel, after_exit_label: &mut CodeLabel, registers_to_save: HashSet<Register>) {
        assembler.set_label(before_exit_label).unwrap();
        assembler.mov(r15 + RAX_GUEST_OFFSET_CONST, rax).unwrap();
        assembler.mov(r15 + RBX_GUEST_OFFSET_CONST, rbx).unwrap();
        for register in registers_to_save {
            assembler.mov(r15 + register.guest_offset_const(), register.to_native_64()).unwrap();
        }
        // assembler.mov(r15 + RBX_GUEST_OFFSET_CONST, rbx).unwrap();
        // assembler.mov(r15 + RCX_GUEST_OFFSET_CONST, rcx).unwrap();
        // assembler.mov(r15 + RDX_GUEST_OFFSET_CONST, rdx).unwrap();
        // assembler.mov(r15 + RDI_GUEST_OFFSET_CONST, rdi).unwrap();
        // assembler.mov(r15 + RSI_GUEST_OFFSET_CONST, rsi).unwrap();
        assembler.mov(r15 + RBP_GUEST_OFFSET_CONST, rbp).unwrap();
        assembler.mov(r15 + RSP_GUEST_OFFSET_CONST, rsp).unwrap();
        // assembler.mov(r15 + R8_GUEST_OFFSET_CONST, r8).unwrap();
        // assembler.mov(r15 + R9_GUEST_OFFSET_CONST, r9).unwrap();
        // assembler.mov(r15 + R10_GUEST_OFFSET_CONST, r10).unwrap();
        // assembler.mov(r15 + R11_GUEST_OFFSET_CONST, r11).unwrap();
        // assembler.mov(r15 + R12_GUEST_OFFSET_CONST, r12).unwrap();
        // assembler.mov(r15 + R13_GUEST_OFFSET_CONST, r13).unwrap();
        // assembler.mov(r15 + R14_GUEST_OFFSET_CONST, r14).unwrap();
        assembler.lea(r10, qword_ptr(before_exit_label.clone())).unwrap();//safe to clober r10 b/c it was saved
        assembler.mov(r15 + RIP_GUEST_OFFSET_CONST, r10).unwrap();
        assembler.jmp(qword_ptr(r15 + RIP_NATIVE_OFFSET_CONST)).unwrap();
        assembler.set_label(after_exit_label).unwrap();
    }

    pub fn get_new_base_address(&self) -> BaseAddress {
        BaseAddress(self.inner.read().unwrap().max_ptr)
    }

    pub fn add_method_implementation(&self, code: Vec<u8>, base_address: BaseAddress) -> MethodImplementationID {
        let mut inner_guard = self.inner.write().unwrap();
        let current_method_id = inner_guard.method_id_max;
        inner_guard.method_id_max.0 += 1;
        let new_method_base = inner_guard.max_ptr;
        assert_eq!(base_address.0, new_method_base);
        let code_len = code.len();
        let end_of_new_method = unsafe {
            new_method_base.offset(code_len as isize)
        };
        let method_range = (new_method_base as *const c_void)..(end_of_new_method as *const c_void);
        inner_guard.code_regions.insert(current_method_id, method_range.clone());
        inner_guard.code_regions_to_method.insert(method_range, current_method_id);
        inner_guard.max_ptr = end_of_new_method;
        unsafe { copy_nonoverlapping(code.as_ptr() as *const c_void, new_method_base as *mut c_void, code_len); }
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

pub const RIP_NATIVE_OFFSET_CONST: usize = 0 + XSAVE_AREA_GUEST_OFFSET_CONST + XSAVE_SIZE + 0;
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