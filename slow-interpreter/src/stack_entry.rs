use std::collections::HashSet;
use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::jobject;
use rust_jvm_common::classfile::CPIndex;

use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;

#[derive(Debug)]
pub struct StackEntry {
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: Option<CPIndex>,

    pub local_vars: Vec<JavaValue>,
    pub operand_stack: Vec<JavaValue>,
    pub pc: usize,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: isize,
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub(crate) opaque: bool,//todo ideally this would not be needed and instead we would just have native frames and non-native
}

impl StackEntry {
    pub(crate) fn new_native_frame(class_pointer: Arc<RuntimeClass>) -> StackEntry {
        StackEntry {
            class_pointer,
            method_i: None,
            local_vars: vec![],
            operand_stack: vec![],
            pc: 0,
            pc_offset: 0,
            native_local_refs: vec![HashSet::new()],
            opaque: true
        }
    }

    pub fn pop(&mut self) -> JavaValue {
        self.operand_stack.pop().unwrap_or_else(|| {
            // let classfile = &self.class_pointer.classfile;
            // let method = &classfile.methods[self.method_i as usize];
            // dbg!(&method.method_name(&classfile));
            // dbg!(&method.code_attribute().unwrap().code);
            // dbg!(&self.pc);
            panic!()
        })
    }
    pub fn push(&mut self, j: JavaValue) {
        self.operand_stack.push(j)
    }

    pub fn is_opaque(&self) -> bool {
        assert!(self.is_native());
        return self.opaque && self.is_native();
    }

    pub fn is_native(&self) -> bool {
        let method_i = match self.method_i {
            None => return false,
            Some(i) => i,
        };
        self.class_pointer.view().method_view_i(method_i as usize).is_native()
    }
}

