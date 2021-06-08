use std::collections::HashSet;
use std::ffi::c_void;
use std::intrinsics::size_of;
use std::ops::{DerefMut, Index, IndexMut};
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;

use classfile_view::loading::LoaderName;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use gc_memory_layout_common::{FrameHeader, FrameInfo, StackframeMemoryLayout};
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use rust_jvm_common::classfile::CPIndex;

use crate::java_values::{JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::native_util::{from_object, to_object};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RuntimeClassClassId(usize);

#[derive(Debug, Clone, Copy)]
pub struct FrameView(*mut c_void);

impl FrameView {
    pub fn new(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    fn get_header(&self) -> &FrameHeader {
        unsafe { (self.0 as *const FrameHeader).as_ref() }.unwrap()
    }

    fn get_frame_info(&self) -> &FrameInfo {
        unsafe { self.get_header().frame_info_ptr.as_ref() }.unwrap()
    }

    fn get_frame_info_mut(&mut self) -> &mut FrameInfo {
        unsafe { self.get_header().frame_info_ptr.as_mut() }.unwrap()
    }

    pub fn loader(&self) -> LoaderName {
        *match self.get_frame_info() {
            FrameInfo::FullyOpaque { loader, .. } => loader,
            FrameInfo::Native { loader, .. } => loader,
            FrameInfo::JavaFrame { loader, .. } => loader
        }
    }

    pub fn method_id(&self) -> Option<MethodId> {
        Some(match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => None?,
            FrameInfo::Native { method_id, .. } => *method_id,
            FrameInfo::JavaFrame { method_id, .. } => *method_id
        })
    }

    pub fn pc(&self) -> u16 {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { java_pc, .. } => *java_pc
        }
    }

    pub fn pc_mut(&mut self) -> &mut u16 {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { java_pc, .. } => java_pc
        }
    }


    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { pc_offset, .. } => pc_offset
        }
    }

    pub fn pc_offset(&self) -> i32 {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { pc_offset, .. } => *pc_offset
        }
    }

    pub fn operand_stack_length(&self) -> u16 {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { operand_stack_depth, .. } => *operand_stack_depth
        }
    }

    pub fn is_native(&self) -> bool {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => false,
            FrameInfo::Native { .. } => true,
            FrameInfo::JavaFrame { .. } => false
        }
    }

    fn max_locals(&self) -> u16 {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => 0,
            FrameInfo::Native { .. } => 0,
            FrameInfo::JavaFrame { num_locals: max_locals, .. } => *max_locals
        }
    }

    fn get_operand_stack_base(&self) -> *mut c_void {
        //todo should be based of actual layout instead of this
        unsafe { self.0.offset((size_of::<FrameHeader>() + size_of::<u64>() * (self.max_locals() as usize)) as isize) }
    }

    fn get_local_var_base(&self) -> *mut c_void {
        //todo should be based of actual layout instead of this
        unsafe { self.0.offset((size_of::<FrameHeader>()) as isize) }
    }

    fn write_target(target: *mut c_void, j: JavaValue) {
        unsafe {
            match j {
                JavaValue::Long(val) => {
                    (target as *mut jlong).write(val)
                }
                JavaValue::Int(val) => {
                    (target as *mut jint).write(val)
                }
                JavaValue::Short(val) => {
                    (target as *mut jshort).write(val)
                }
                JavaValue::Byte(val) => {
                    (target as *mut jbyte).write(val)
                }
                JavaValue::Boolean(val) => {
                    (target as *mut jboolean).write(val)
                }
                JavaValue::Char(val) => {
                    (target as *mut jchar).write(val)
                }
                JavaValue::Float(val) => {
                    (target as *mut jfloat).write(val)
                }
                JavaValue::Double(val) => {
                    (target as *mut jdouble).write(val)
                }
                JavaValue::Object(val) => {
                    let to_write = to_object(val);
                    dbg!(&to_write);
                    dbg!(target);
                    (target as *mut jobject).write(to_write)
                }
                JavaValue::Top => panic!()
            }
        }
    }

    fn read_target(target: *const c_void, expected_type: PTypeView) -> JavaValue {
        unsafe {
            match expected_type {
                PTypeView::ByteType => {
                    JavaValue::Byte((target as *const jbyte).read())
                }
                PTypeView::CharType => {
                    JavaValue::Char((target as *const jchar).read())
                }
                PTypeView::DoubleType => {
                    JavaValue::Double((target as *const jdouble).read())
                }
                PTypeView::FloatType => {
                    JavaValue::Float((target as *const jfloat).read())
                }
                PTypeView::IntType => {
                    JavaValue::Int((target as *const jint).read())
                }
                PTypeView::LongType => {
                    JavaValue::Long((target as *const jlong).read())
                }
                PTypeView::Ref(ref_) => {
                    JavaValue::Object(from_object((target as *const jobject).read()))
                }
                PTypeView::ShortType => {
                    JavaValue::Short((target as *const jshort).read())
                }
                PTypeView::BooleanType => {
                    JavaValue::Boolean((target as *const jboolean).read())
                }
                PTypeView::VoidType |
                PTypeView::TopType |
                PTypeView::NullType |
                PTypeView::Uninitialized(_) |
                PTypeView::UninitializedThis |
                PTypeView::UninitializedThisOrClass(_) => todo!()
            }
        }
    }

    pub fn push_operand_stack(&mut self, j: JavaValue) {
        let operand_stack_depth = self.get_frame_info_mut().operand_stack_depth_mut();
        let current_depth = *operand_stack_depth;
        *operand_stack_depth += 1;
        let operand_stack_base = self.get_operand_stack_base();
        let target = unsafe { operand_stack_base.offset(((current_depth as usize) * size_of::<jlong>()) as isize) };
        Self::write_target(target, j)
    }

    pub fn pop_operand_stack(&mut self, expected_type: PTypeView) -> Option<JavaValue> {
        let operand_stack_depth_mut = self.get_frame_info_mut().operand_stack_depth_mut();
        let current_depth = *operand_stack_depth_mut;
        if current_depth == 0 {
            return None;
        }
        *operand_stack_depth_mut -= 1;
        let new_current_depth = *operand_stack_depth_mut;
        let operand_stack_base = self.get_operand_stack_base();
        let target = unsafe { operand_stack_base.offset((new_current_depth as usize * size_of::<jlong>()) as isize) };
        Some(Self::read_target(target, expected_type))
    }

    pub fn set_local_var(&mut self, i: u16, jv: JavaValue) {
        let target = unsafe { self.get_local_var_base().offset((i as isize) * size_of::<jlong>() as isize) };
        Self::write_target(target, jv)
    }

    pub fn get_local_var(&self, i: u16, expected_type: PTypeView) -> JavaValue {
        let target = unsafe { self.get_local_var_base().offset((i as isize) * size_of::<jlong>() as isize) };
        Self::read_target(target, expected_type)
    }
}


/// If the frame is opaque then this data is optional.
/// This data would typically be present in a native function call, but not be present in JVMTI frames
#[derive(Debug, Clone)]
pub struct OpaqueFrameOptional {
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,
}

///This data is only present in non-native frames,
/// program counter is not meaningful in a native frame
#[derive(Debug, Clone)]
pub struct NonNativeFrameData {
    pub pc: u16,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: i32,
}

#[derive(Debug, Clone)]
pub struct StackEntry {
    pub(crate) loader: LoaderName,
    pub(crate) opaque_frame_optional: Option<OpaqueFrameOptional>,
    pub(crate) non_native_data: Option<NonNativeFrameData>,
    pub(crate) local_vars: Vec<JavaValue>,
    pub(crate) operand_stack: Vec<JavaValue>,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
}

#[derive(Debug)]
pub enum StackEntryMut<'l> {
    LegacyInterpreter {
        entry: &'l mut StackEntry
    },
    Jit {
        frame_view: FrameView,
    },
}

impl StackEntryMut<'_> {
    pub fn set_pc(&mut self, new_pc: u16) {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                *entry.pc_mut() = new_pc;
            }
            StackEntryMut::Jit { frame_view, .. } => {
                *frame_view.pc_mut() = new_pc;
            }
        }
    }

    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pc_offset_mut()
            }
            StackEntryMut::Jit { .. } => todo!()
        }
    }

    pub fn to_ref(&self) -> StackEntryRef {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => StackEntryRef::LegacyInterpreter { entry },
            StackEntryMut::Jit { frame_view, .. } => StackEntryRef::Jit { frame_view: frame_view.clone() }
        }
    }

    pub fn class_pointer(&self, jvm: &JVMState) -> Arc<RuntimeClass> {
        self.to_ref().class_pointer(jvm).clone()
    }

    pub fn local_vars_mut(&mut self) -> LocalVarsMut {
        match self {
            StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsMut::LegacyInterpreter { vars: entry.local_vars_mut() }
            }
            StackEntryMut::Jit { frame_view, .. } => {
                LocalVarsMut::Jit { frame_view }
            }
        }
    }

    pub fn local_vars(&mut self) -> LocalVarsRef {
        match self {
            StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsRef::LegacyInterpreter { vars: entry.local_vars_mut() }
            }
            StackEntryMut::Jit { frame_view, .. } => {
                LocalVarsRef::Jit { frame_view }
            }
        }
    }

    pub fn push(&mut self, j: JavaValue) {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.push(j);
            }
            StackEntryMut::Jit { .. } => {
                self.operand_stack_mut().push(j);
            }
        }
    }

    pub fn pop(&mut self, expected_type: PTypeView) -> JavaValue {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pop()
            }
            StackEntryMut::Jit { .. } => {
                self.operand_stack_mut().pop(expected_type).unwrap()
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

    pub fn set_pc_offset(&mut self, offset: i32) {
        match self {
            StackEntryMut::LegacyInterpreter { entry, .. } => {
                *entry.pc_offset_mut() = offset
            }
            StackEntryMut::Jit { frame_view, .. } => {
                *frame_view.pc_offset_mut() = offset;
            }
        }
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

impl LocalVarsMut<'_> {
    pub fn set(&mut self, i: u16, to: JavaValue) {
        match self {
            LocalVarsMut::LegacyInterpreter { .. } => todo!(),
            LocalVarsMut::Jit { frame_view, .. } => {
                frame_view.set_local_var(i, to)
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

impl LocalVarsRef<'_> {
    pub fn get(&self, i: u16, expected_type: PTypeView) -> JavaValue {
        match self {
            LocalVarsRef::LegacyInterpreter { .. } => todo!(),
            LocalVarsRef::Jit { frame_view, .. } => {
                frame_view.get_local_var(i, expected_type)
            }
        }
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

    pub fn len(&self) -> u16 {
        match self {
            OperandStackRef::LegacyInterpreter { .. } => todo!(),
            OperandStackRef::Jit { frame_view, .. } => {
                frame_view.operand_stack_length()
            }
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
        frame_view: &'l mut FrameView,
    },
}

impl OperandStackMut<'_> {
    pub fn push(&mut self, j: JavaValue) {
        match self {
            OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.push(j);
            }
            OperandStackMut::Jit { frame_view } => {
                frame_view.push_operand_stack(j)
            }
        }
    }

    pub fn pop(&mut self, ptypeview: PTypeView) -> Option<JavaValue> {
        match self {
            OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.pop()
            }
            OperandStackMut::Jit { frame_view, .. } => {
                frame_view.pop_operand_stack(ptypeview)
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

#[derive(Debug, Clone)]
pub enum StackEntryRef<'l> {
    LegacyInterpreter {
        entry: &'l StackEntry
    },
    Jit {
        frame_view: FrameView,
    },
}


impl StackEntryRef<'_> {
    pub fn loader(&self) -> LoaderName {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.loader()
            }
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.loader()
            }
        }
    }

    pub fn try_class_pointer(&self, jvm: &JVMState) -> Option<Arc<RuntimeClass>> {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.try_class_pointer().cloned()
            }
            StackEntryRef::Jit { frame_view, .. } => {
                let method_id = frame_view.method_id()?;
                let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                Some(rc)
            }
        }
    }

    pub fn class_pointer(&self, jvm: &JVMState) -> Arc<RuntimeClass> {
        self.try_class_pointer(jvm).unwrap()
    }

    pub fn pc(&self) -> u16 {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.pc()
            }
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.pc()
            }
        }
    }

    pub fn pc_offset(&self) -> i32 {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => { entry.pc_offset() }
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.pc_offset()
            }
        }
    }

    pub fn method_i(&self, jvm: &JVMState) -> CPIndex {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => { entry.method_i() }
            StackEntryRef::Jit { frame_view, .. } => {
                let method_id = frame_view.method_id().unwrap();
                let (_, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                method_i
            }
        }
    }

    pub fn operand_stack(&self) -> OperandStackRef {
        match self {
            StackEntryRef::LegacyInterpreter { .. } => todo!(),
            StackEntryRef::Jit { frame_view, .. } => {
                OperandStackRef::Jit { frame_view }
            }
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => entry.is_native(),
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.is_native()
            }
        }
    }

    pub fn native_local_refs(&self) -> &mut Vec<BiMap<ByAddress<Arc<Object>>, jobject>> {
        match self {
            StackEntryRef::LegacyInterpreter { entry, .. } => todo!("{:?}", entry),
            StackEntryRef::Jit { frame_view, .. } => todo!("{:?}", frame_view)
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
            native_local_refs: vec![HashSet::new()],
        }
    }

    pub fn new_java_frame(jvm: &JVMState, class_pointer: Arc<RuntimeClass>, method_i: u16, args: Vec<JavaValue>) -> Self {
        let max_locals = class_pointer.view().method_view_i(method_i).code_attribute().unwrap().max_locals;
        assert!(args.len() >= max_locals as usize);
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&class_pointer);
        let mut guard = jvm.method_table.write().unwrap();
        let _method_id = guard.get_method_id(class_pointer.clone(), method_i);
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
            native_local_refs: vec![HashSet::new()],
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

    pub fn pc_mut(&mut self) -> &mut u16 {
        &mut self.non_native_data.as_mut().unwrap().pc
    }

    pub fn pc(&self) -> u16 {
        self.try_pc().unwrap()
    }

    pub fn try_pc(&self) -> Option<u16> {
        self.non_native_data.as_ref().map(|x| x.pc)
    }


    //todo a lot of duplication here between mut and non-mut variants
    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        &mut self.non_native_data.as_mut().unwrap().pc_offset
    }

    pub fn pc_offset(&self) -> i32 {
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
        self.class_pointer().view().method_view_i(method_i).is_native()
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