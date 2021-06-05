use std::ffi::c_void;
use std::ops::{Index, IndexMut};
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;

use classfile_view::loading::LoaderName;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use gc_memory_layout_common::{FrameHeader, FrameInfo};
use jvmti_jni_bindings::jobject;
use rust_jvm_common::classfile::CPIndex;

use crate::java_values::{JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::runtime_class::RuntimeClass;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RuntimeClassClassId(usize);

#[derive(Debug, Clone)]
pub struct FrameView(*mut c_void);

impl FrameView {
    fn get_header(&self) -> &FrameHeader {
        unsafe { (self.0 as *const FrameHeader).as_ref() }.unwrap()
    }

    fn get_frame_info(&self) -> &FrameInfo {
        unsafe { self.get_header().frame_info_ptr.as_ref() }.unwrap()
    }

    pub fn loader(&self) -> LoaderName {
        *match self.get_frame_info() {
            FrameInfo::FullyOpaque { loader } => loader,
            FrameInfo::Native { loader, .. } => loader,
            FrameInfo::JavaFrame { loader, .. } => loader
        }
    }

    pub fn try_class_pointer(&self) -> Option<RuntimeClassClassId> {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => None,
            FrameInfo::Native { runtime_class_id, .. } => Some(RuntimeClassClassId(*runtime_class_id)),
            FrameInfo::JavaFrame { runtime_class_id, .. } => Some(RuntimeClassClassId(*runtime_class_id))
        }
    }
}


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
    native_local_refs: Vec<BiMap<ByAddress<Arc<Object>>, jobject>>,
}

#[derive(Debug)]
pub enum StackEntryMut<'l> {
    LegacyInterpreter {
        entry: &'l mut StackEntry
    },
    Jit {
        frame_view: FrameView
    },
}

impl StackEntryMut<'_> {
    pub fn pc_mut(&mut self) -> &mut usize {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => { entry.pc_mut() }
            StackEntryMut::Jit { .. } => todo!(),
        }
    }

    pub fn pc_offset_mut(&mut self) -> &mut isize {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pc_offset_mut()
            }
            StackEntryMut::Jit { .. } => todo!()
        }
    }

    pub fn to_ref<'l>(&'l self) -> StackEntryRef<'l> {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => StackEntryRef::LegacyInterpreter { entry },
            StackEntryMut::Jit { frame_view, .. } => StackEntryRef::Jit { frame_view }
        }
    }

    pub fn class_pointer(&self) -> Arc<RuntimeClass> {
        self.to_ref().class_pointer().clone()
    }
}

//todo maybe I should do something about all the boilerplate but leaving as is for now
#[derive(Debug)]
pub enum LocalVarsMut<'l> {
    LegacyInterpreter {
        vars: &'l mut Vec<JavaValue>
    },
    Jit {
        frame_view: &'l mut FrameView
    },
}

impl Index<usize> for LocalVarsMut<'_> {
    type Output = JavaValue;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            LocalVarsMut::LegacyInterpreter { vars } => {
                &vars[index]
            }
            LocalVarsMut::Jit { .. } => {
                todo!()
            }
        }
    }
}

impl IndexMut<usize> for LocalVarsMut<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            LocalVarsMut::LegacyInterpreter { vars } => {
                &mut vars[index]
            }
            LocalVarsMut::Jit { frame_view } => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum LocalVarsRef<'l> {
    LegacyInterpreter {
        vars: &'l Vec<JavaValue>
    },
    Jit {
        frame_view: &'l FrameView
    },
}

impl Index<usize> for LocalVarsRef<'_> {
    type Output = JavaValue;

    fn index(&self, index: usize) -> &Self::Output {
        todo!()
    }
}

pub enum OperandStackRef<'l> {
    LegacyInterpreter {
        operand_stack: &'l Vec<JavaValue>
    },
    Jit {
        frame_view: &'l FrameView
    },
}

impl OperandStackRef<'_> {
    pub fn is_empty(&self) -> bool {
        match self {
            OperandStackRef::LegacyInterpreter { .. } => todo!(),
            OperandStackRef::Jit { .. } => todo!()
        }
    }

    pub fn len(&self) -> usize {
        match self {
            OperandStackRef::LegacyInterpreter { .. } => todo!(),
            OperandStackRef::Jit { .. } => todo!()
        }
    }

    pub fn last(&self) -> Option<&JavaValue> {
        todo!()
    }
}

impl Index<usize> for OperandStackRef<'_> {
    type Output = JavaValue;

    fn index(&self, index: usize) -> &Self::Output {
        todo!()
    }
}

impl Index<usize> for OperandStackMut<'_> {
    type Output = JavaValue;

    fn index(&self, index: usize) -> &Self::Output {
        todo!()
    }
}

impl IndexMut<usize> for OperandStackMut<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        todo!()
    }
}

pub enum OperandStackMut<'l> {
    LegacyInterpreter {
        operand_stack: &'l mut Vec<JavaValue>
    },
    Jit {
        frame_view: &'l mut FrameView
    },
}


impl OperandStackMut<'_> {
    pub fn push(&mut self, j: JavaValue) {
        match self {
            OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.push(j);
            }
            OperandStackMut::Jit { frame_view, .. } => {
                todo!()
            }
        }
    }

    pub fn pop(&mut self) -> Option<JavaValue> {
        match self {
            OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.pop()
            }
            OperandStackMut::Jit { frame_view, .. } => {
                todo!()
            }
        }
    }

    pub fn insert(&self, index: usize, j: JavaValue) {
        match self {
            OperandStackMut::LegacyInterpreter { .. } => todo!(),
            OperandStackMut::Jit { .. } => todo!()
        }
    }

    pub fn len(&self) -> usize {
        match self {
            OperandStackMut::LegacyInterpreter { .. } => todo!(),
            OperandStackMut::Jit { .. } => todo!()
        }
    }
}

impl StackEntryMut<'_> {
    pub fn local_vars_mut(&mut self) -> LocalVarsMut {
        match self {
            StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsMut::LegacyInterpreter { vars: entry.local_vars_mut() }
            }
            StackEntryMut::Jit { frame_view } => {
                LocalVarsMut::Jit { frame_view: frame_view }
            }
        }
    }

    pub fn local_vars(&mut self) -> LocalVarsRef {
        match self {
            StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsRef::LegacyInterpreter { vars: entry.local_vars_mut() }
            }
            StackEntryMut::Jit { frame_view } => {
                LocalVarsRef::Jit { frame_view: frame_view }
            }
        }
    }

    pub fn push(&mut self, j: JavaValue) {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.push(j);
            }
            StackEntryMut::Jit { frame_view, .. } => {
                todo!()
            }
        }
    }

    pub fn pop(&mut self) -> JavaValue {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pop()
            }
            StackEntryMut::Jit { frame_view, .. } => {
                todo!()
            }
        }
    }

    pub fn operand_stack_mut(&mut self) -> OperandStackMut {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                OperandStackMut::LegacyInterpreter { operand_stack: entry.operand_stack_mut() }
            }
            StackEntryMut::Jit { frame_view, .. } => {
                OperandStackMut::Jit { frame_view }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum StackEntryRef<'l> {
    LegacyInterpreter {
        entry: &'l StackEntry
    },
    Jit {
        frame_view: &'l FrameView
    },
}


impl StackEntryRef<'_> {
    pub fn loader(&self) -> LoaderName {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.loader()
            }
            StackEntryRef::Jit { frame_view, .. } => {
                todo!()
            }
        }
    }

    pub fn try_class_pointer(&self) -> Option<&Arc<RuntimeClass>> {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.try_class_pointer()
            }
            StackEntryRef::Jit { frame_view, .. } => {
                todo!()
            }
        }
    }

    pub fn class_pointer(&self) -> &Arc<RuntimeClass> {
        self.try_class_pointer().unwrap()
    }

    pub fn pc(&self) -> usize {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.pc()
            }
            StackEntryRef::Jit { .. } => todo!()
        }
    }

    pub fn pc_offset(&self) -> isize {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => { entry.pc_offset() }
            StackEntryRef::Jit { .. } => todo!()
        }
    }

    pub fn method_i(&self) -> CPIndex {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => { entry.method_i() }
            StackEntryRef::Jit { .. } => todo!()
        }
    }

    pub fn operand_stack(&self) -> OperandStackRef {
        match self {
            StackEntryRef::LegacyInterpreter { .. } => todo!(),
            StackEntryRef::Jit { .. } => todo!()
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => entry.is_native(),
            StackEntryRef::Jit { .. } => todo!()
        }
    }

    pub fn native_local_refs(&self) -> &mut Vec<BiMap<ByAddress<Arc<Object>>, jobject>> {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => todo!(),
            StackEntryRef::Jit { frame_view, .. } => todo!()
        }
    }

    pub fn local_vars(&self) -> LocalVarsRef {
        match self {
            StackEntryRef::LegacyInterpreter { entry } => {
                LocalVarsRef::LegacyInterpreter { vars: entry.local_vars() }
            }
            StackEntryRef::Jit { frame_view } => {
                LocalVarsRef::Jit { frame_view }
            }
        }
    }
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
        let mut guard = jvm.method_table.write().unwrap();
        let method_id = guard.get_method_id(class_pointer.clone(), method_i);
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
            }
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

    pub fn current_method_id(&self, jvm: &JVMState) -> Option<MethodId> {
        let optional = self.opaque_frame_optional.as_ref()?;
        let mut guard = jvm.method_table.write().unwrap();
        Some(guard.get_method_id(optional.class_pointer.clone(), optional.method_i))
    }
}

impl AsRef<StackEntry> for StackEntry {
    fn as_ref(&self) -> &StackEntry {
        self
    }
}

impl AsMut<StackEntry> for StackEntry {
    fn as_mut(&mut self) -> &mut StackEntry {
        self
    }
}