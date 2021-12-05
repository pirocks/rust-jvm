#![feature(asm)]
// save all registers when entering and exiting vm -
// methodid to code id mapping is handled seperately
// exit handling has registered handling but actual handling is seperate -
// have another layer above this which gets rid of native points and does everytthing in terms of IR
// have java layer above that

use std::collections::HashMap;
use std::ffi::c_void;
use std::intrinsics::copy_nonoverlapping;
use std::ops::Range;
use std::ptr::null_mut;
use std::sync::RwLock;

use iced_x86::{Code, Instruction, MemoryOperand, Register};
use iced_x86::code_asm::{CodeAssembler, CodeLabel, r15};
use libc::{MAP_ANONYMOUS, MAP_NORESERVE, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
use memoffset::offset_of;
use rangemap::RangeMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct MethodImplementationID(usize);

pub struct MethodOffset(usize);

pub struct VMStateInner<'vm_state_life, T: Sized> {
    method_id_max: MethodImplementationID,
    exit_handlers: HashMap<MethodImplementationID, Box<dyn Fn(&VMExitEvent) -> VMExitAction<T> + 'vm_state_life>>,
    code_regions: HashMap<MethodImplementationID, Range<*mut c_void>>,
    code_regions_to_method: RangeMap<*mut c_void, MethodImplementationID>,
    max_ptr: *mut c_void,
}

pub struct VMState<'vm_life, T: Sized> {
    inner: RwLock<VMStateInner<'vm_life, T>>,
    mmaped_code_region_base: *mut c_void,
    mmaped_code_size: usize,
}

impl<T> Drop for VMState<'_, T> {
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
    pub rip: *mut c_void,
    pub saved_registers_without_ip: SavedRegistersWithoutIP,
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
        todo!()
    }
}

#[derive(Copy, Clone)]
pub struct VMExitEvent {
    pub method: MethodImplementationID,
    pub method_base_address: *mut c_void,
    pub saved_guest_registers: SavedRegistersWithIP,
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

impl<'vm_state_life, T> VMState<'vm_state_life, T> {
    //don't store exit type in here, that can go in register or derive from ip, include base method address in  event
    pub fn new() -> VMState<'vm_state_life, T> {
        const DEFAULT_CODE_SIZE: usize = 1024 * 1024 * 1024;
        unsafe {
            let mmaped_code_region_base = libc::mmap(null_mut(), DEFAULT_CODE_SIZE, PROT_READ | PROT_WRITE | PROT_EXEC, MAP_ANONYMOUS | MAP_PRIVATE | MAP_NORESERVE, -1, 0) as *mut c_void;
            VMState {
                inner: RwLock::new(VMStateInner {
                    method_id_max: MethodImplementationID(0),
                    exit_handlers: Default::default(),
                    code_regions: Default::default(),
                    code_regions_to_method: Default::default(),
                    max_ptr: mmaped_code_region_base,
                }),
                mmaped_code_region_base,
                mmaped_code_size: DEFAULT_CODE_SIZE,
            }
        }
    }


    pub fn launch_vm(&self, method_id: MethodImplementationID, initial_registers: SavedRegistersWithoutIP) -> T {
        let code_region: Range<*mut c_void> = self.inner.read().unwrap().code_regions.get(&method_id).unwrap().clone();
        let branch_to = code_region.start;
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
        let rip_native_offset = offset_of!(SavedRegistersWithIP, rip) + offset_of!(SavedRegistersWithIP, saved_registers_without_ip) + offset_of!(JITContext, vm_native_saved_registers);
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
        loop {
            let vm_exit_action = self.run_method_impl(&mut jit_context);
            match vm_exit_action {
                VMExitAction::ExitVMCompletely { return_data } => {
                    return return_data;
                }
                VMExitAction::ReturnTo { return_register_state } => {
                    jit_context.guest_registers = return_register_state;
                }
            }
        }
    }

    #[allow(named_asm_labels)]
    fn run_method_impl(&self, jit_context: &mut JITContext) -> VMExitAction<T> {
        let jit_context_pointer = jit_context as *mut JITContext as *mut c_void;
        unsafe {
            asm!(
            //save all registers to avoid breaking stuff
            "mov r15, {0}",
            "mov [r15 + {rax_native_offset_const}], rax",
            "mov [r15 + {rbx_native_offset_const}], rbx",
            "mov [r15 + {rcx_native_offset_const}], rcx",
            "mov [r15 + {rdx_native_offset_const}], rdx",
            "mov [r15 + {rsi_native_offset_const}], rsi",
            "mov [r15 + {rdi_native_offset_const}], rdi",
            "mov [r15 + {rbp_native_offset_const}], rbp",
            "mov [r15 + {rsp_native_offset_const}], rsp",
            "mov [r15 + {r8_native_offset_const}], r8",
            "mov [r15 + {r9_native_offset_const}], r9",
            "mov [r15 + {r10_native_offset_const}], r10",
            "mov [r15 + {r11_native_offset_const}], r11",
            "mov [r15 + {r12_native_offset_const}], r12",
            "mov [r15 + {r13_native_offset_const}], r13",
            "mov [r15 + {r14_native_offset_const}], r14",
            "xsave [r15 + {xsave_area_native_offset_const}]",
            "lea rax, [rip+__rust_jvm_internal_after_enter]",
            "mov [r15 + {rip_native_offset_const}], rax",
            //load expected register values
            "mov rax,[r15 + {rax_guest_offset_const}]",
            "mov rbx,[r15 + {rbx_guest_offset_const}]",
            "mov rcx,[r15 + {rcx_guest_offset_const}]",
            "mov rdx,[r15 + {rdx_guest_offset_const}]",
            "mov rsi,[r15 + {rsi_guest_offset_const}]",
            "mov rdi,[r15 + {rdi_guest_offset_const}]",
            "mov rbp,[r15 + {rbp_guest_offset_const}]",
            "mov rsp,[r15 + {rsp_guest_offset_const}]",
            "mov r8,[r15 + {r8_guest_offset_const}]",
            "mov r9,[r15 + {r9_guest_offset_const}]",
            "mov r10,[r15 + {r10_guest_offset_const}]",
            "mov r11,[r15 + {r11_guest_offset_const}]",
            "mov r12,[r15 + {r12_guest_offset_const}]",
            "mov r13,[r15 + {r13_guest_offset_const}]",
            "mov r14,[r15 + {r14_guest_offset_const}]",
            "xrstor [r15 + {xsave_area_guest_offset_const}]",
            "call qword ptr [r15 + rdi_guest_offset_const]",
            "__rust_jvm_internal_after_enter:",
            in(reg) jit_context_pointer,
            // rip_guest_offset_const = const RIP_GUEST_OFFSET_CONST,
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
            rip_native_offset_const = const RIP_NATIVE_OFFSET_CONST,
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
            )
        }
        self.handle_vm_exit(jit_context.guest_registers.rip, jit_context.guest_registers)
    }

    fn handle_vm_exit(&self, guest_rip: *mut c_void, guest_registers: SavedRegistersWithIP) -> VMExitAction<T> {
        let inner_read_guard = self.inner.read().unwrap();
        let method_implementation = inner_read_guard.code_regions_to_method.get(&guest_rip);
        match method_implementation {
            None => {
                todo!()
            }
            Some(method_implementation) => {
                let handler = inner_read_guard.exit_handlers.get(&method_implementation).unwrap();
                let vm_exit_event = VMExitEvent {
                    method: *method_implementation,
                    method_base_address: inner_read_guard.code_regions.get(method_implementation).unwrap().start,
                    saved_guest_registers: guest_registers,
                };
                return handler(&vm_exit_event);
            }
        }
    }

    pub fn gen_vm_exit(assembler: &mut CodeAssembler) -> VMExitLabel {
        let mut before_exit_label = assembler.create_label();
        let mut after_exit_label = assembler.create_label();
        assembler.set_label(&mut before_exit_label).unwrap();
        assembler.add_instruction(Instruction::with2(Code::Mov_rm64_r64, MemoryOperand::with_base_displ(Register::R15, RIP_GUEST_OFFSET_CONST as i64), Register::RIP).unwrap()).unwrap();
        assembler.jmp(r15 + RIP_NATIVE_OFFSET_CONST).unwrap();
        assembler.set_label(&mut after_exit_label).unwrap();
        VMExitLabel {
            before_exit_label,
            after_exit_label,
        }
    }

    pub fn get_new_base_address(&self) -> BaseAddress {
        BaseAddress(self.inner.read().unwrap().max_ptr)
    }

    pub fn add_method_implementation(&self, method: Method<'vm_state_life, T>, base_address: BaseAddress) -> MethodImplementationID {
        let mut inner_guard = self.inner.write().unwrap();
        let current_method_id = inner_guard.method_id_max;
        inner_guard.method_id_max.0 += 1;
        let Method { code, exit_handler } = method;
        inner_guard.exit_handlers.insert(current_method_id, exit_handler);
        let new_method_base = inner_guard.max_ptr;
        assert_eq!(base_address.0, new_method_base);
        let code_len = code.len();
        let end_of_new_method = unsafe {
            inner_guard.max_ptr.offset(code_len as isize)
        };
        let method_range = new_method_base..end_of_new_method;
        inner_guard.code_regions.insert(current_method_id, method_range.clone());
        inner_guard.code_regions_to_method.insert(method_range, current_method_id);
        inner_guard.max_ptr = end_of_new_method;
        unsafe { copy_nonoverlapping(code.as_ptr() as *const c_void, new_method_base, code_len); }
        current_method_id
    }
}

#[must_use]
pub struct BaseAddress(pub *const c_void);

pub struct VMExitLabel {
    before_exit_label: CodeLabel,
    after_exit_label: CodeLabel,
}

pub struct Method<'vm_state_life, T: Sized> {
    pub code: Vec<u8>,
    pub exit_handler: Box<dyn Fn(&VMExitEvent) -> VMExitAction<T> + 'vm_state_life>,
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

pub const RIP_NATIVE_OFFSET_CONST: usize = 0 + 120 + 4096 + 0;
pub const RAX_NATIVE_OFFSET_CONST: usize = 0 + 120 + 4096 + 8;
pub const RBX_NATIVE_OFFSET_CONST: usize = 8 + 120 + 4096 + 8;
pub const RCX_NATIVE_OFFSET_CONST: usize = 16 + 120 + 4096 + 8;
pub const RDX_NATIVE_OFFSET_CONST: usize = 24 + 120 + 4096 + 8;
pub const RSI_NATIVE_OFFSET_CONST: usize = 32 + 120 + 4096 + 8;
pub const RDI_NATIVE_OFFSET_CONST: usize = 40 + 120 + 4096 + 8;
pub const RBP_NATIVE_OFFSET_CONST: usize = 48 + 120 + 4096 + 8;
pub const RSP_NATIVE_OFFSET_CONST: usize = 56 + 120 + 4096 + 8;
pub const R8_NATIVE_OFFSET_CONST: usize = 64 + 120 + 4096 + 8;
pub const R9_NATIVE_OFFSET_CONST: usize = 72 + 120 + 4096 + 8;
pub const R10_NATIVE_OFFSET_CONST: usize = 80 + 120 + 4096 + 8;
pub const R11_NATIVE_OFFSET_CONST: usize = 88 + 120 + 4096 + 8;
pub const R12_NATIVE_OFFSET_CONST: usize = 96 + 120 + 4096 + 8;
pub const R13_NATIVE_OFFSET_CONST: usize = 104 + 120 + 4096 + 8;
pub const R14_NATIVE_OFFSET_CONST: usize = 112 + 120 + 4096 + 8;
pub const XSAVE_AREA_NATIVE_OFFSET_CONST: usize = 120 + 120 + 4096 + 8;