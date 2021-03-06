use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;

use classfile_view::loading::LoaderName;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::jobject;
use rust_jvm_common::classfile::CPIndex;

use crate::java_values::{JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::runtime_class::RuntimeClass;

/// If the frame is opaque then this data is optional.
/// This data would typically be present in a native function call, but not be present in JVMTI frames
#[derive(Debug, Clone)]
struct OpaqueFrameOptional {
    class_pointer: Arc<RuntimeClass>,
    method_i: CPIndex,
}

///This data is only present in non-native frames,
/// program counter is not meaningful in a native frame
#[derive(Debug, Clone)]
struct NonNativeFrameData {
    pc: usize,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pc_offset: isize,
}

#[derive(Debug, Clone)]
pub struct StackEntry {
    loader: LoaderName,
    opaque_frame_optional: Option<OpaqueFrameOptional>,
    non_native_data: Option<NonNativeFrameData>,
    local_vars: Vec<JavaValue>,
    operand_stack: Vec<JavaValue>,
    pub(crate) native_local_refs: Vec<BiMap<ByAddress<Arc<Object>>, jobject>>,
}

impl StackEntry {
    pub fn new_completely_opaque_frame(loader: LoaderName) -> Self {
        //need a better name here
        Self {
            loader,
            opaque_frame_optional: None,
            non_native_data: None,
            local_vars: vec![],
            operand_stack: vec![],
            native_local_refs: vec![BiMap::new()],
        }
    }

    pub fn new_java_frame(jvm: &JVMState, class_pointer: Arc<RuntimeClass>, method_i: u16, args: Vec<JavaValue>) -> Self {
        let max_locals = class_pointer.view().method_view_i(method_i as usize).code_attribute().unwrap().max_locals;
        assert!(args.len() >= max_locals as usize);
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&class_pointer);
        Self {
            loader,
            opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
            non_native_data: Some(NonNativeFrameData { pc: 0, pc_offset: 0 }),
            local_vars: args,
            operand_stack: vec![],
            native_local_refs: vec![],
        }
    }

    pub fn new_native_frame(jvm: &JVMState, class_pointer: Arc<RuntimeClass>, method_i: u16, args: Vec<JavaValue>) -> Self {
        Self {
            loader: jvm.classes.read().unwrap().get_initiating_loader(&class_pointer),
            opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
            non_native_data: None,
            local_vars: args,
            operand_stack: vec![],
            native_local_refs: vec![BiMap::new()],
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

    pub fn class_pointer(&self) -> &Arc<RuntimeClass> {
        &match self.opaque_frame_optional.as_ref() {
            Some(x) => x,
            None => {
                unimplemented!()
            },
        }.class_pointer
    }


    pub fn try_class_pointer(&self) -> Option<&Arc<RuntimeClass>> {
        Some(&self.opaque_frame_optional.as_ref()?.class_pointer)
    }

    pub fn local_vars(&self) -> &Vec<JavaValue> {
        &self.local_vars
    }

    pub fn local_vars_mut(&mut self) -> &mut Vec<JavaValue> {
        &mut self.local_vars
    }

    pub fn operand_stack_mut(&mut self) -> &mut Vec<JavaValue> {
        &mut self.operand_stack
    }

    pub fn operand_stack(&self) -> &Vec<JavaValue> {
        &self.operand_stack
    }

    pub fn pc_mut(&mut self) -> &mut usize {
        &mut self.non_native_data.as_mut().unwrap().pc
    }

    pub fn pc(&self) -> usize {
        self.try_pc().unwrap()
    }

    pub fn try_pc(&self) -> Option<usize> {
        self.non_native_data.as_ref().map(|x| x.pc)
    }


    //todo a lot of duplication here between mut and non-mut variants
    pub fn pc_offset_mut(&mut self) -> &mut isize {
        &mut self.non_native_data.as_mut().unwrap().pc_offset
    }

    pub fn pc_offset(&self) -> isize {
        self.non_native_data.as_ref().unwrap().pc_offset
    }

    pub fn method_i(&self) -> CPIndex {
        self.opaque_frame_optional.as_ref().unwrap().method_i
    }

    pub fn try_method_i(&self) -> Option<CPIndex> {
        self.opaque_frame_optional.as_ref().map(|x| x.method_i)
    }

    pub fn is_native(&self) -> bool {
        let method_i = match self.try_method_i() {
            None => return true,
            Some(i) => i,
        };
        self.class_pointer().view().method_view_i(method_i as usize).is_native()
    }

    pub fn convert_to_native(&mut self) {
        self.non_native_data.take();
    }

    pub fn operand_stack_types(&self) -> Vec<PTypeView> {
        self.operand_stack().iter().map(|type_| type_.to_type()).collect()
    }

    pub fn local_vars_types(&self) -> Vec<PTypeView> {
        self.local_vars().iter().map(|type_| type_.to_type()).collect()
    }

    pub fn loader(&self) -> LoaderName {
        self.loader
    }

    pub fn privileged_frame(&self) -> bool {
        todo!()
    }

    pub fn is_opaque_frame(&self) -> bool {
        self.try_class_pointer().is_none() || self.try_method_i().is_none() || self.is_native()
    }
}

