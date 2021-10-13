use std::collections::HashSet;
use std::ffi::c_void;
use std::intrinsics::size_of;
use std::marker::PhantomData;
use std::mem::transmute;
use std::ptr::{NonNull, null_mut};
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;
use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::{FrameHeader, FrameInfo, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use jit_common::java_stack::JavaStack;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::java_values::{GcManagedObject, JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::runtime_class::RuntimeClass;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RuntimeClassClassId(usize);

#[derive(Debug, Clone, Copy)]
pub struct FrameView<'gc_life, 'l> {
    frame_ptr: *mut c_void,
    call_stack: &'l JavaStack,
    phantom_data: PhantomData<&'gc_life ()>,
}

impl<'gc_life, 'l> FrameView<'gc_life, 'l> {
    pub fn new(ptr: *mut c_void, call_stack: &'l JavaStack) -> Self {
        let res = Self {
            frame_ptr: ptr,
            call_stack,
            phantom_data: PhantomData::default(),
        };
        let _header = res.get_header();
        res
    }

    fn get_header(&self) -> &FrameHeader {
        let res = unsafe { (self.frame_ptr as *const FrameHeader).as_ref() }.unwrap();
        let FrameHeader { magic_part_1, magic_part_2, .. } = *res;
        assert_eq!(magic_part_1, MAGIC_1_EXPECTED);
        assert_eq!(magic_part_2, MAGIC_2_EXPECTED);
        res
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
            FrameInfo::FullyOpaque { operand_stack_depth, .. } => *operand_stack_depth,
            FrameInfo::Native { operand_stack_depth, .. } => *operand_stack_depth,
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
        unsafe { self.frame_ptr.offset((size_of::<FrameHeader>() + size_of::<u64>() * (self.max_locals() as usize)) as isize) }
    }

    fn get_local_var_base(&self) -> *mut c_void {
        //todo should be based of actual layout instead of this
        unsafe { self.frame_ptr.offset((size_of::<FrameHeader>()) as isize) }
    }

    pub(crate) fn raw_write_target(target: *mut c_void, jv: jvalue) {
        unsafe {
            assert_eq!(size_of::<jvalue>(), size_of::<u64>());
            (target as *mut u64).write(jv.j as u64);
        }
    }

    pub(crate) fn write_target(target: *mut c_void, j: JavaValue<'gc_life>) {
        // dbg!("write",target,&j);
        unsafe {
            match j {
                JavaValue::Long(val) => {
                    (target as *mut jlong).write(val)
                }
                JavaValue::Int(val) => {
                    (target as *mut u64).write(0);
                    (target as *mut jint).write(val)
                }
                JavaValue::Short(val) => {
                    (target as *mut u64).write(0);
                    (target as *mut jshort).write(val)
                }
                JavaValue::Byte(val) => {
                    (target as *mut u64).write(0);
                    (target as *mut jbyte).write(val)
                }
                JavaValue::Boolean(val) => {
                    (target as *mut u64).write(0);
                    (target as *mut jboolean).write(val)
                }
                JavaValue::Char(val) => {
                    (target as *mut u64).write(0);
                    (target as *mut jchar).write(val)
                }
                JavaValue::Float(val) => {
                    (target as *mut u64).write(0);
                    (target as *mut jfloat).write(val)
                }
                JavaValue::Double(val) => {
                    (target as *mut jdouble).write(val)
                }
                JavaValue::Object(val) => {
                    match val {
                        None => {
                            (target as *mut jobject).write(null_mut())
                        }
                        Some(val) => {
                            (target as *mut jobject).write(val.raw_ptr_usize() as jobject);
                            // eprintln!("Write:{:?}", target as *mut jobject)
                        }
                    }
                }
                JavaValue::Top => {
                    (target as *mut u64).write(0xBEAFCAFEBEAFCAFE)
                }
            }
        }
    }

    pub(crate) fn read_target(jvm: &'gc_life JVMState<'gc_life>, target: *const c_void, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        // dbg!("read",target,&expected_type);
        unsafe {
            match expected_type {
                RuntimeType::DoubleType => {
                    JavaValue::Double((target as *const jdouble).read())
                }
                RuntimeType::FloatType => {
                    assert_eq!((target as *const u64).read() >> 32, 0);
                    JavaValue::Float((target as *const jfloat).read())
                }
                RuntimeType::IntType => {
                    assert_eq!((target as *const u64).read() >> 32, 0);
                    JavaValue::Int((target as *const jint).read())
                }
                RuntimeType::LongType => {
                    JavaValue::Long((target as *const jlong).read())
                }
                RuntimeType::Ref(_) => {
                    let obj = (target as *const jobject).read();
                    // eprintln!("Read:{:?}", obj);
                    match NonNull::new(obj as *mut c_void) {
                        None => {
                            JavaValue::Object(None)
                        }
                        Some(ptr) => {
                            let res = JavaValue::Object(GcManagedObject::from_native(ptr, jvm).into());
                            // res.self_check();
                            res
                        }
                    }
                }
                RuntimeType::TopType => {
                    dbg!(&expected_type);
                    todo!()
                }
            }
        }
    }

    fn get_frame_ptrs(&self) -> Vec<usize> {
        let mut start = self.frame_ptr;
        let mut res = vec![];
        loop {
            unsafe {
                if start == null_mut() || start == transmute(0xDEADDEADDEADDEADusize) || start == self.call_stack.top {
                    break;
                }
                res.push(start as usize);
                start = (*(start as *mut FrameHeader)).prev_rpb;
            }
        }
        res
    }

    pub fn push_operand_stack(&mut self, j: JavaValue<'gc_life>) {
        let frame_info = self.get_frame_info_mut();
        frame_info.push_operand_stack(j.to_type());
        // dbg!(self.get_frame_ptrs());
        let operand_stack_depth = frame_info.operand_stack_depth_mut();
        let current_depth = *operand_stack_depth;
        *operand_stack_depth += 1;
        let operand_stack_base = self.get_operand_stack_base();
        let target = unsafe { operand_stack_base.offset(((current_depth as usize) * size_of::<jlong>()) as isize) };
        Self::write_target(target, j)
    }

    pub fn pop_operand_stack(&mut self, jvm: &'gc_life JVMState<'gc_life>, expected_type: Option<RuntimeType>) -> Option<JavaValue<'gc_life>> {
        let operand_stack_depth_mut = self.get_frame_info_mut().operand_stack_depth_mut();
        let current_depth = *operand_stack_depth_mut;
        if current_depth == 0 {
            return None;
        }
        *operand_stack_depth_mut -= 1;
        let new_current_depth = *operand_stack_depth_mut;
        let type_ = self.get_frame_info_mut().pop_operand_stack().unwrap();
        let operand_stack_base = self.get_operand_stack_base();
        let target = unsafe { operand_stack_base.offset((new_current_depth as usize * size_of::<jlong>()) as isize) };
        let res = Self::read_target(jvm, target, expected_type.unwrap_or(type_.clone()));
        Some(res)
    }

    pub fn get_current_operand_stack_type_state(&self) -> Vec<RuntimeType> {
        self.get_frame_info().operand_stack_types()
    }

    pub fn set_local_var(&mut self, _jvm: &'gc_life JVMState<'gc_life>, i: u16, jv: JavaValue<'gc_life>) {
        self.get_frame_info_mut().set_local_var_type(jv.to_type(), i as usize);
        let target = unsafe { self.get_local_var_base().offset((i as isize) * size_of::<jlong>() as isize) };
        Self::write_target(target, jv)
    }

    pub fn get_local_var(&self, jvm: &'gc_life JVMState<'gc_life>, i: u16, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        let target = unsafe { self.get_local_var_base().offset((i as isize) * size_of::<jlong>() as isize) };
        Self::read_target(jvm, target, expected_type)
    }

    pub fn get_operand_stack(&self, jvm: &'gc_life JVMState<'gc_life>, from_start: u16, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        let target = unsafe { self.get_operand_stack_base().offset((from_start as isize) * size_of::<jlong>() as isize) };
        Self::read_target(jvm, target, expected_type)
    }

    pub fn set_operand_stack(&mut self, from_start: u16, to_set: JavaValue<'gc_life>) {
        let target = unsafe { self.get_operand_stack_base().offset((from_start as isize) * size_of::<jlong>() as isize) };
        Self::write_target(target, to_set)
    }

    pub fn as_stack_entry_partially_correct(&self, jvm: &'gc_life JVMState<'gc_life>) -> StackEntry<'gc_life> {
        let operand_stack_types = match self.get_frame_info() {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types
        };
        assert_eq!(self.operand_stack_length() as usize, operand_stack_types.len());
        let operand_stack = operand_stack_types.iter().enumerate().map(|(i, ptype)| {
            self.get_operand_stack(jvm, i as u16, ptype.clone())
        }).rev().collect_vec();
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { loader, .. } => {
                StackEntry::new_completely_opaque_frame(*loader, operand_stack)
            }
            FrameInfo::Native { method_id, loader, operand_stack_depth: _, native_local_refs, .. } => {
                let (class_pointer, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                StackEntry {
                    loader: *loader,
                    opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
                    non_native_data: None,
                    local_vars: vec![],
                    operand_stack,
                    native_local_refs: native_local_refs.clone(),
                }
            }
            FrameInfo::JavaFrame { method_id, loader, java_pc, pc_offset, locals_types, .. } => {
                let (class_pointer, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let local_vars = locals_types.iter().enumerate().map(|(i, ptype)| {
                    match ptype {
                        RuntimeType::TopType => JavaValue::Top,
                        _ => self.get_local_var(jvm, i as u16, ptype.clone()),
                    }
                }).collect_vec();
                StackEntry {
                    loader: *loader,
                    opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
                    non_native_data: Some(NonNativeFrameData { pc: *java_pc, pc_offset: *pc_offset }),
                    local_vars,
                    operand_stack,
                    native_local_refs: Default::default(),
                }
            }
        }
    }

    pub fn get_local_refs(&self) -> Vec<HashSet<jobject>> {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.clone(),
            FrameInfo::JavaFrame { .. } => panic!()
        }
    }

    pub fn set_local_refs_top_frame(&mut self, new: HashSet<jobject>) {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => {
                *native_local_refs.last_mut().unwrap() = new;
            }
            FrameInfo::JavaFrame { .. } => panic!()
        }
    }


    pub fn pop_local_refs(&mut self) -> HashSet<jobject> {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => {
                native_local_refs.pop().unwrap()
            }
            FrameInfo::JavaFrame { .. } => panic!()
        }
    }

    pub fn push_local_refs(&mut self, to_push: HashSet<jobject>) {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => {
                native_local_refs.push(to_push)
            }
            FrameInfo::JavaFrame { .. } => panic!()
        }
    }

    pub fn pop(&mut self) -> Option<HashSet<jobject>> {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.pop(),
            FrameInfo::JavaFrame { .. } => panic!()
        }
    }

    pub fn push(&mut self, jobjects: HashSet<jobject>) {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.push(jobjects),
            FrameInfo::JavaFrame { .. } => panic!()
        }
    }
}


pub struct StackIter<'vm_life, 'l> {
    jvm: &'vm_life JVMState<'vm_life>,
    current_frame: *mut c_void,
    java_stack: &'l JavaStack,
    top: *mut c_void,
}

impl<'l, 'k> StackIter<'l, 'k> {
    pub fn new(jvm: &'l JVMState<'l>, java_stack: &'k JavaStack) -> Self {
        Self {
            jvm,
            current_frame: java_stack.current_frame_ptr(),
            java_stack,
            top: java_stack.top,
        }
    }
}

impl<'vm_life> Iterator for StackIter<'vm_life, '_> {
    type Item = StackEntry<'vm_life>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame != self.top {
            let frame_view = FrameView::new(self.current_frame, self.java_stack);
            let _ = frame_view.get_header();
            let res = Some(frame_view.as_stack_entry_partially_correct(self.jvm));
            self.current_frame = frame_view.get_header().prev_rpb;
            res
        } else {
            None
        }
    }
}


/// If the frame is opaque then this data is optional.
/// This data would typically be present in a native function call, but not be present in JVMTI frames
#[derive(Debug, Clone)]
pub struct OpaqueFrameOptional<'gc_life> {
    pub class_pointer: Arc<RuntimeClass<'gc_life>>,
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
pub struct StackEntry<'gc_life> {
    pub(crate) loader: LoaderName,
    pub(crate) opaque_frame_optional: Option<OpaqueFrameOptional<'gc_life>>,
    pub(crate) non_native_data: Option<NonNativeFrameData>,
    pub(crate) local_vars: Vec<JavaValue<'gc_life>>,
    pub(crate) operand_stack: Vec<JavaValue<'gc_life>>,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
}

pub enum StackEntryMut<'gc_life, 'l> {
    /*LegacyInterpreter {
        entry: &'l mut StackEntry
    },*/
    Jit {
        frame_view: FrameView<'gc_life, 'l>,
        jvm: &'gc_life JVMState<'gc_life>,
    },
}

impl<'gc_life, 'l> StackEntryMut<'gc_life, 'l> {
    pub fn set_pc(&mut self, new_pc: u16) {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                *entry.pc_mut() = new_pc;
            }*/
            StackEntryMut::Jit { frame_view, .. } => {
                *frame_view.pc_mut() = new_pc;
            }
        }
    }

    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pc_offset_mut()
            }*/
            StackEntryMut::Jit { frame_view, .. } => {
                frame_view.pc_offset_mut()
            }
        }
    }

    pub fn to_ref(&self) -> StackEntryRef<'gc_life, 'l> {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => StackEntryRef::LegacyInterpreter { entry },*/
            StackEntryMut::Jit { frame_view, .. } => StackEntryRef::Jit { frame_view: frame_view.clone() }
        }
    }

    pub fn class_pointer(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
        self.to_ref().class_pointer(jvm).clone()
    }

    pub fn local_vars_mut(&'k mut self) -> LocalVarsMut<'gc_life, 'l, 'k> {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsMut::LegacyInterpreter { vars: entry.local_vars_mut(jvm) }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => {
                LocalVarsMut::Jit { frame_view, jvm }
            }
        }
    }

    pub fn local_vars(&'k self) -> LocalVarsRef<'gc_life, 'k, 'l> {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsRef::LegacyInterpreter { vars: entry.local_vars_mut(jvm) }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => {
                LocalVarsRef::Jit { frame_view, jvm }
            }
        }
    }

    pub fn push(&mut self, j: JavaValue<'gc_life>) {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.push(j);
            }*/
            StackEntryMut::Jit { .. } => {
                self.operand_stack_mut().push(j);
            }
        }
    }

    pub fn pop(&mut self, expected_type: Option<RuntimeType>) -> JavaValue<'gc_life> {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pop()
            }*/
            StackEntryMut::Jit { .. } => {
                match self.operand_stack_mut().pop(expected_type) {
                    Some(x) => x,
                    None => {
                        panic!()
                    }
                }
            }
        }
    }

    pub fn operand_stack_mut(&'k mut self) -> OperandStackMut<'gc_life, 'l, 'k> {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                OperandStackMut::LegacyInterpreter { operand_stack: entry.operand_stack_mut() }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => {
                OperandStackMut::Jit { frame_view, jvm }
            }
        }
    }

    pub fn operand_stack_ref(&'k mut self, _jvm: &'gc_life JVMState<'gc_life>) -> OperandStackRef<'gc_life, 'l, 'k> {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                OperandStackRef::LegacyInterpreter { operand_stack: entry.operand_stack_mut() }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => {
                OperandStackRef::Jit { frame_view, jvm }
            }
        }
    }

    pub fn debug_extract_raw_frame_view(&'k mut self) -> &'k mut FrameView<'gc_life, 'l> {
        match self {
            StackEntryMut::Jit { frame_view, .. } => frame_view
        }
    }

    pub fn set_pc_offset(&mut self, offset: i32) {
        match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                *entry.pc_offset_mut() = offset
            }*/
            StackEntryMut::Jit { frame_view, .. } => {
                *frame_view.pc_offset_mut() = offset;
            }
        }
    }
}

//todo maybe I should do something about all the boilerplate but leaving as is for now
pub enum LocalVarsMut<'gc_life, 'l, 'k> {
    /*LegacyInterpreter {
        vars: &'l mut Vec<JavaValue<'gc_life>>
    },*/
    Jit {
        frame_view: &'k mut FrameView<'gc_life, 'l>,
        jvm: &'gc_life JVMState<'gc_life>,
    },
}

impl<'gc_life, 'l, 'k> LocalVarsMut<'gc_life, 'l, 'k> {
    pub fn set(&mut self, i: u16, to: JavaValue<'gc_life>) {
        match self {
            /*LocalVarsMut::LegacyInterpreter { .. } => todo!(),*/
            LocalVarsMut::Jit { frame_view, jvm } => {
                frame_view.set_local_var(jvm, i, to)
            }
        }
    }
}

#[derive(Clone)]
pub enum LocalVarsRef<'gc_life, 'l, 'k> {
    /*    LegacyInterpreter {
            vars: &'l Vec<JavaValue>
        },*/
    Jit {
        frame_view: &'k FrameView<'gc_life, 'l>,
        jvm: &'gc_life JVMState<'gc_life>,
    },
}

impl<'gc_life> LocalVarsRef<'gc_life, '_, '_> {
    pub fn get(&self, i: u16, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        match self {
            /*LocalVarsRef::LegacyInterpreter { .. } => todo!(),*/
            LocalVarsRef::Jit { frame_view, jvm } => {
                frame_view.get_local_var(jvm, i, expected_type)
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LocalVarsRef::Jit { frame_view, .. } => {
                match frame_view.get_frame_info() {
                    FrameInfo::FullyOpaque { .. } => panic!(),
                    FrameInfo::Native { .. } => panic!(),
                    FrameInfo::JavaFrame { num_locals, locals_types, .. } => {
                        assert_eq!(locals_types.len(), *num_locals as usize);

                        *num_locals as usize
                    }
                }
            }
        }
    }
}

pub enum OperandStackRef<'gc_life, 'l, 'k> {
    /*LegacyInterpreter {
        operand_stack: &'l Vec<JavaValue>
    },*/
    Jit {
        frame_view: &'k FrameView<'gc_life, 'l>,
        jvm: &'gc_life JVMState<'gc_life>,
    },
}

impl<'gc_life, 'l, 'k> OperandStackRef<'gc_life, 'l, 'k> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> u16 {
        match self {
            /*OperandStackRef::LegacyInterpreter { .. } => todo!(),*/
            OperandStackRef::Jit { frame_view, .. } => {
                frame_view.operand_stack_length()
            }
        }
    }

    pub fn last(&self) -> Option<&JavaValue<'gc_life>> {
        todo!()
    }

    pub fn get(&'_ self, from_start: u16, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        match self {
            /*OperandStackRef::LegacyInterpreter { .. } => todo!(),*/
            OperandStackRef::Jit { frame_view, jvm } => {
                frame_view.get_operand_stack(jvm, from_start, expected_type)
            }
        }
    }

    pub fn types(&self) -> Vec<RuntimeType> {
        match self {
            OperandStackRef::Jit { frame_view, .. } => {
                match frame_view.get_frame_info() {
                    FrameInfo::FullyOpaque { .. } => panic!(),
                    FrameInfo::Native { .. } => panic!(),
                    FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types.clone()
                }
            }
        }
    }
    pub fn types_vals(&self) -> Vec<JavaValue<'gc_life>> {
        match self {
            OperandStackRef::Jit { frame_view, jvm } => {
                frame_view.as_stack_entry_partially_correct(jvm).operand_stack.clone()
            }
        }
    }
}

pub enum OperandStackMut<'gc_life, 'l, 'k> {
    /*    LegacyInterpreter {
            operand_stack: &'l mut Vec<JavaValue<'gc_life>>
        }*/
    Jit {
        frame_view: &'k mut FrameView<'gc_life, 'l>,
        jvm: &'gc_life JVMState<'gc_life>,
    },
}

impl OperandStackMut<'gc_life, 'l, 'k> {
    pub fn push(&mut self, j: JavaValue<'gc_life>) {
        match self {
            /*OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.push(j);
            }*/
            OperandStackMut::Jit { frame_view, .. } => {
                frame_view.push_operand_stack(j)
            }
        }
    }

    pub fn pop(&mut self, rtype: Option<RuntimeType>) -> Option<JavaValue<'gc_life>> {
        match self {
            /*OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.pop()
            }*/
            OperandStackMut::Jit { frame_view, jvm } => {
                frame_view.pop_operand_stack(jvm, rtype)
            }
        }
    }

    pub fn insert(&mut self, index: usize, j: JavaValue<'gc_life>) {
        let mut temp = vec![];
        for _ in index..self.len() {
            temp.push(self.pop(None).unwrap());
        }
        self.push(j);
        for to_push in temp {
            self.push(to_push);
        }
    }

    pub fn len(&self) -> usize {
        (match self {
            /*OperandStackMut::LegacyInterpreter { .. } => todo!(),*/
            OperandStackMut::Jit { frame_view, .. } => frame_view.operand_stack_length()
        }) as usize
    }
}

#[derive(Debug, Clone)]
pub enum StackEntryRef<'gc_life, 'l> {
    /*LegacyInterpreter {
        entry: &'l StackEntry
    },*/
    Jit {
        frame_view: FrameView<'gc_life, 'l>,
    },
}


impl<'gc_life, 'l> StackEntryRef<'gc_life, 'l> {
    pub fn loader(&self) -> LoaderName {
        match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.loader()
            }*/
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.loader()
            }
        }
    }

    pub fn try_class_pointer(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<Arc<RuntimeClass<'gc_life>>> {
        match self {
            /*            StackEntryRef::LegacyInterpreter { entry, .. } => {
                            entry.try_class_pointer().cloned()
                        }
            */            StackEntryRef::Jit { frame_view, .. } => {
                let method_id = frame_view.method_id()?;
                let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                Some(rc)
            }
        }
    }

    pub fn class_pointer(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
        self.try_class_pointer(jvm).unwrap()
    }

    pub fn pc(&self) -> u16 {
        match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.pc()
            }*/
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.pc()
            }
        }
    }

    pub fn pc_offset(&self) -> i32 {
        match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => { entry.pc_offset() }*/
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.pc_offset()
            }
        }
    }

    pub fn method_i(&self, jvm: &'gc_life JVMState<'gc_life>) -> CPIndex {
        match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => { entry.method_i() }*/
            StackEntryRef::Jit { frame_view, .. } => {
                let method_id = frame_view.method_id().unwrap();
                let (_, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                method_i
            }
        }
    }

    pub fn operand_stack(&'k self, jvm: &'gc_life JVMState<'gc_life>) -> OperandStackRef<'gc_life, 'l, 'k> {
        match self {
            /*StackEntryRef::LegacyInterpreter { .. } => todo!(),*/
            StackEntryRef::Jit { frame_view, .. } => {
                OperandStackRef::Jit { frame_view, jvm }
            }
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => entry.is_native(),*/
            StackEntryRef::Jit { frame_view, .. } => {
                frame_view.is_native()
            }
        }
    }

    pub fn native_local_refs(&self) -> &mut Vec<BiMap<ByAddress<GcManagedObject<'gc_life>>, jobject>> {
        match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => todo!("{:?}", entry),*/
            StackEntryRef::Jit { frame_view, .. } => todo!("{:?}", frame_view)
        }
    }

    pub fn local_vars(&'l self, jvm: &'gc_life JVMState<'gc_life>) -> LocalVarsRef<'gc_life, 'l, 'k> {
        match self {
            /*            StackEntryRef::LegacyInterpreter { entry } => {
                            LocalVarsRef::LegacyInterpreter { vars: entry.local_vars(jvm) }
                        }
            */            StackEntryRef::Jit { frame_view } => {
                LocalVarsRef::Jit { frame_view, jvm }
            }
        }
    }
}

impl<'gc_life> StackEntry<'gc_life> {
    pub fn new_completely_opaque_frame(loader: LoaderName, operand_stack: Vec<JavaValue<'gc_life>>) -> Self {
        //need a better name here
        Self {
            loader,
            opaque_frame_optional: None,
            non_native_data: None,
            local_vars: vec![],
            operand_stack,
            native_local_refs: vec![HashSet::new()],
        }
    }

    pub fn new_java_frame(jvm: &'gc_life JVMState<'gc_life>, class_pointer: Arc<RuntimeClass<'gc_life>>, method_i: u16, args: Vec<JavaValue<'gc_life>>) -> Self {
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

    pub fn new_native_frame(jvm: &'gc_life JVMState<'gc_life>, class_pointer: Arc<RuntimeClass<'gc_life>>, method_i: u16, args: Vec<JavaValue<'gc_life>>) -> Self {
        Self {
            loader: jvm.classes.read().unwrap().get_initiating_loader(&class_pointer),
            opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
            non_native_data: None,
            local_vars: args,
            operand_stack: vec![],
            native_local_refs: vec![HashSet::new()],
        }
    }

    pub fn pop(&mut self) -> JavaValue<'gc_life> {
        self.operand_stack.pop().unwrap_or_else(|| {
            // let classfile = &self.class_pointer.classfile;
            // let method = &classfile.methods[self.method_i as usize];
            // dbg!(&method.method_name(&classfile));
            // dbg!(&method.code_attribute().unwrap().code);
            // dbg!(&self.pc);
            panic!()
        })
    }
    pub fn push(&mut self, j: JavaValue<'gc_life>) {
        self.operand_stack.push(j)
    }

    pub fn class_pointer(&self) -> &Arc<RuntimeClass<'gc_life>> {
        &match self.opaque_frame_optional.as_ref() {
            Some(x) => x,
            None => {
                unimplemented!()
            }
        }.class_pointer
    }


    pub fn try_class_pointer(&self) -> Option<&Arc<RuntimeClass<'gc_life>>> {
        Some(&self.opaque_frame_optional.as_ref()?.class_pointer)
    }

    pub fn local_vars(&self) -> &Vec<JavaValue<'gc_life>> {
        &self.local_vars
    }

    pub fn local_vars_mut(&mut self) -> &mut Vec<JavaValue<'gc_life>> {
        &mut self.local_vars
    }

    pub fn operand_stack_mut(&mut self) -> &mut Vec<JavaValue<'gc_life>> {
        &mut self.operand_stack
    }

    pub fn operand_stack(&self) -> &Vec<JavaValue<'gc_life>> {
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

    pub fn operand_stack_types(&self) -> Vec<RuntimeType> {
        self.operand_stack().iter().map(|type_| type_.to_type()).collect()
    }

    pub fn local_vars_types(&self) -> Vec<RuntimeType> {
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

    pub fn current_method_id(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<MethodId> {
        let optional = self.opaque_frame_optional.as_ref()?;
        let mut guard = jvm.method_table.write().unwrap();
        Some(guard.get_method_id(optional.class_pointer.clone(), optional.method_i))
    }
}

impl<'gc_life> AsRef<StackEntry<'gc_life>> for StackEntry<'gc_life> {
    fn as_ref(&self) -> &StackEntry<'gc_life> {
        self
    }
}

impl<'gc_life> AsMut<StackEntry<'gc_life>> for StackEntry<'gc_life> {
    fn as_mut(&mut self) -> &mut StackEntry<'gc_life> {
        self
    }
}
