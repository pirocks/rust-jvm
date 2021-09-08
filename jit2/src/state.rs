use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::intrinsics::copy_nonoverlapping;
use std::ptr::null_mut;
use std::thread::LocalKey;

use itertools::Itertools;

use rust_jvm_common::compressed_classfile::code::CompressedCode;

use crate::{ir_to_native, IRInstructionIndex, LabelName, MethodID, to_ir, ToNative, VMExitType};
use crate::ir::IRLabel;

thread_local! {
pub static CODE_ADDRESS : RefCell<*mut c_void> = RefCell::new(null_mut());
}

pub struct JITState {
    code: &'static LocalKey<RefCell<*mut c_void>>,
    function_addresses: Vec<*mut c_void>,
    // indexed by method_id
    current_end: *mut c_void,
    current_jit_instr: IRInstructionIndex,
    exits: HashMap<*mut c_void, VMExitType>,
    labels: HashMap<LabelName, *mut c_void>,
    labeler: Labeler,
}

impl JITState {
    pub fn add_function(&mut self, code: &CompressedCode, methodid: MethodID) -> *mut c_void {
        let CompressedCode {
            instructions,
            max_locals,
            max_stack,
            exception_table,
            stack_map_table
        } = code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();
        let ir = to_ir(cinstructions, self.current_jit_instr, &mut self.labeler, todo!()).unwrap();
        let ToNative {
            code,
            new_labels
        } = ir_to_native(ir, self.current_end);
        let install_at = self.current_end;
        unsafe { self.current_end = install_at.offset(code.len() as isize); }
        self.code.with(|code_refcell| {
            const TWO_GIG: isize = 2 * 1024 * 1024 * 1024;
            unsafe {
                if self.current_end.offset_from(*code_refcell.borrow()) > TWO_GIG {
                    panic!()
                }
            }
        });
        self.labels.extend(new_labels.into_iter());
        unsafe {
            copy_nonoverlapping(
                code.as_ptr(),
                install_at as *mut u8,
                code.len(),
            )
        }
        install_at
    }
}


pub struct Labeler {
    current_label: u32,
}

impl Labeler {
    pub fn new_label(&mut self, labels_vec: &mut Vec<IRLabel>) -> LabelName {
        let current_label = self.current_label.checked_add(1).unwrap();
        self.current_label = current_label;
        let res = LabelName(current_label);
        labels_vec.push(IRLabel { name: res });
        res
    }
}