use std::arch::x86_64::_mm256_testc_pd;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::ffi::c_void;
use std::intrinsics::size_of;
use std::marker::PhantomData;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;
use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::{FrameHeader, FrameInfo, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jvalue};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::RuntimeType;
use rust_jvm_common::vtype::VType;

use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OwnedJavaStack, RuntimeJavaStackFrameMut, RuntimeJavaStackFrameRef};
use crate::java_values::{GcManagedObject, JavaValue, Object};
use crate::jit::ByteCodeOffset;
use crate::jit::state::Opaque;
use crate::jit_common::java_stack::JavaStack;
use crate::jvm_state::JVMState;
use crate::runtime_class::RuntimeClass;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RuntimeClassClassId(usize);

#[derive(Clone, Copy)]
pub struct FrameView<'gc_life, 'l> {
    frame_ptr: *mut c_void,
    call_stack: &'l OwnedJavaStack<'gc_life>,
    phantom_data: PhantomData<&'gc_life ()>,
    current_ip: *mut c_void,
}

impl<'gc_life, 'l> FrameView<'gc_life, 'l> {
    pub fn new(ptr: *mut c_void, call_stack: &'l mut OwnedJavaStack<'gc_life>, current_ip: *mut c_void) -> Self {
        let res = Self { frame_ptr: ptr, call_stack, phantom_data: PhantomData::default(), current_ip };
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

    pub fn loader(&self, jvm: &'gc_life JVMState<'gc_life>) -> LoaderName {
        let method_id = self.get_header().methodid;
        if method_id == MethodId::MAX {
            //frame loader unknown b/c opaque frame
            // eprintln!("WARN: opaque frame loader");
            return LoaderName::BootstrapLoader;
        }
        let method_table = jvm.method_table.read().unwrap();
        let (rc, method_i) = method_table.try_lookup(method_id).unwrap();
        jvm.classes.read().unwrap().get_initiating_loader(&rc)
    }

    pub fn method_id(&self) -> Option<MethodId> {
        let methodid = self.get_header().methodid;
        if methodid == 0xbeafbeafbeafbeaf {
            assert!(false);
        }
        Some(methodid)
    }

    pub fn pc(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<ByteCodeOffset, Opaque> {
        let methodid = self.get_header().methodid;
        let byte_code_offset = jvm.jit_state.with(|jit_state| jit_state.borrow().ip_to_bytecode_pc(self.current_ip))?;
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        Ok(byte_code_offset.1)
    }

    pub fn bytecode_index(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<u16, Opaque> {
        let saved_instruction_pointer = todo!();
        let methodid = self.get_header().methodid;
        let byte_code_offset = jvm.jit_state.with(|jit_state| jit_state.borrow().ip_to_bytecode_pc(saved_instruction_pointer))?;
        Ok(byte_code_offset.0)
    }

    pub fn pc_mut(&mut self) -> &mut u16 {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { java_pc, .. } => java_pc,
        }
    }

    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { pc_offset, .. } => pc_offset,
        }
    }

    pub fn pc_offset(&self) -> i32 {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { pc_offset, .. } => *pc_offset,
        }
    }

    pub fn operand_stack_length(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<u16, Opaque> {
        let methodid = self.get_header().methodid;
        if methodid == usize::MAX {
            panic!()
        }
        let pc = self.pc(jvm)?;

        let function_frame_type = jvm.function_frame_type_data.read().unwrap();
        let this_function = function_frame_type.get(&methodid).unwrap();
        //todo issue here is that we can't use instruct pointer b/c we might have iterated up through vm call and instruct pointer will be saved.
        Ok(this_function.get(&pc.0).unwrap().stack_map.len() as u16)
    }

    pub fn stack_types(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<Vec<RuntimeType>, Opaque> {
        let methodid = self.get_header().methodid;
        if methodid == usize::MAX {
            panic!()
        }
        let pc = self.pc(jvm)?;
        let function_frame_type = jvm.function_frame_type_data.read().unwrap();
        let stack_map = &function_frame_type.get(&methodid).unwrap().get(&pc.0).unwrap().stack_map;
        let mut res = vec![];
        for vtype in &stack_map.data {
            res.push(vtype.to_runtime_type());
        }
        Ok(res)
    }

    pub fn is_native(&self) -> bool {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => false,
            FrameInfo::Native { .. } => true,
            FrameInfo::JavaFrame { .. } => false,
        }
    }

    fn max_locals(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<u16, Opaque> {
        let methodid = self.get_header().methodid;
        if methodid == usize::MAX {
            panic!()
        }
        let pc = self.pc(jvm)?;
        let function_frame_type = jvm.function_frame_type_data.read().unwrap();
        let locals = &function_frame_type.get(&methodid).unwrap().get(&pc.0).unwrap().locals;
        Ok(locals.len() as u16)
    }

    fn get_operand_stack_base(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<*mut c_void, Opaque> {
        //todo should be based of actual layout instead of this
        Ok(unsafe { self.frame_ptr.offset((size_of::<FrameHeader>() + size_of::<u64>() * (dbg!(self.max_locals(jvm)?) as usize)) as isize) })
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
                JavaValue::Long(val) => (target as *mut jlong).write(val),
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
                JavaValue::Double(val) => (target as *mut jdouble).write(val),
                JavaValue::Object(val) => {
                    match val {
                        None => (target as *mut jobject).write(null_mut()),
                        Some(val) => {
                            (target as *mut jobject).write(val.raw_ptr_usize() as jobject);
                            // eprintln!("Write:{:?}", target as *mut jobject)
                        }
                    }
                }
                JavaValue::Top => (target as *mut u64).write(0xBEAFCAFEBEAFCAFE),
            }
        }
    }

    pub(crate) fn read_target(jvm: &'gc_life JVMState<'gc_life>, target: *const c_void, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        // dbg!("read",target,&expected_type);
        unsafe {
            match expected_type {
                RuntimeType::DoubleType => JavaValue::Double((target as *const jdouble).read()),
                RuntimeType::FloatType => {
                    assert_eq!((target as *const u64).read() >> 32, 0);
                    JavaValue::Float((target as *const jfloat).read())
                }
                RuntimeType::IntType => {
                    assert_eq!((target as *const u64).read() >> 32, 0);
                    JavaValue::Int((target as *const jint).read())
                }
                RuntimeType::LongType => JavaValue::Long((target as *const jlong).read()),
                RuntimeType::Ref(_) => {
                    let obj = (target as *const jobject).read();
                    // eprintln!("Read:{:?}", obj);
                    match NonNull::new(obj as *mut c_void) {
                        None => JavaValue::Object(None),
                        Some(ptr) => {
                            if ptr.as_ptr() == usize::MAX as *mut c_void {
                                todo!()
                            }
                            let res = JavaValue::Object(GcManagedObject::from_native(ptr, jvm).into());
                            // res.self_check();
                            res
                        }
                    }
                }
                RuntimeType::TopType => JavaValue::Top,
            }
        }
    }

    fn get_frame_ptrs(&self) -> Vec<usize> {
        let mut start = self.frame_ptr;
        let mut res = vec![];
        loop {
            unsafe {
                if start == null_mut() || start == transmute(0xDEADDEADDEADDEADusize) || start == todo!() {
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
        let operand_stack_depth = frame_info.operand_stack_depth_mut();
        let current_depth = *operand_stack_depth;
        *operand_stack_depth += 1;
        let operand_stack_base = self.get_operand_stack_base(todo!()).unwrap();
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
        let operand_stack_base = self.get_operand_stack_base(todo!()).unwrap();
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
        let operand_stack_base = self.get_operand_stack_base(jvm).unwrap();
        let target = unsafe { operand_stack_base.offset((from_start as isize) * size_of::<jlong>() as isize) };
        Self::read_target(jvm, target, expected_type)
    }

    pub fn set_operand_stack(&mut self, from_start: u16, to_set: JavaValue<'gc_life>) {
        let target = unsafe { self.get_operand_stack_base(todo!()).unwrap().offset((from_start as isize) * size_of::<jlong>() as isize) };
        Self::write_target(target, to_set)
    }

    pub fn as_stack_entry_partially_correct(&self, jvm: &'gc_life JVMState<'gc_life>) -> StackEntry<'gc_life> {
        match self.operand_stack_length(jvm) {
            Ok(_) => {}
            Err(_) => {
                return StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader, vec![]);
            }
        }

        let operand_stack_types = self.stack_types(jvm).unwrap();
        assert_eq!(self.operand_stack_length(jvm).unwrap() as usize, operand_stack_types.len());
        let operand_stack = operand_stack_types
            .iter()
            .enumerate()
            .map(|(i, ptype)| {
                /*self.get_operand_stack(jvm, i as u16, ptype.clone())*/
                dbg!(ptype);
                JavaValue::Top
            })
            .rev()
            .collect_vec();
        let (class_pointer, method_i) = jvm.method_table.read().unwrap().try_lookup(self.method_id().unwrap()).unwrap();
        /*let local_vars = locals_types.iter().enumerate().map(|(i, ptype)| {
            match ptype {
                RuntimeType::TopType => JavaValue::Top,
                _ => self.get_local_var(jvm, i as u16, ptype.clone()),
            }
        }).collect_vec();*/
        let loader = self.loader(jvm);
        StackEntry {
            loader,
            opaque_frame_id: None,
            opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
            non_native_data: Some(NonNativeFrameData { pc: self.pc(jvm).unwrap().0, pc_offset: 0xDDDDDDD }),
            local_vars: vec![],
            operand_stack,
            native_local_refs: Default::default(),
        }
        /*match self.get_frame_info() {
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
        }*/
    }

    pub fn get_local_refs(&self) -> Vec<HashSet<jobject>> {
        match self.get_frame_info() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.clone(),
            FrameInfo::JavaFrame { .. } => panic!(),
        }
    }

    pub fn set_local_refs_top_frame(&mut self, new: HashSet<jobject>) {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => {
                *native_local_refs.last_mut().unwrap() = new;
            }
            FrameInfo::JavaFrame { .. } => panic!(),
        }
    }

    pub fn pop_local_refs(&mut self) -> HashSet<jobject> {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.pop().unwrap(),
            FrameInfo::JavaFrame { .. } => panic!(),
        }
    }

    pub fn push_local_refs(&mut self, to_push: HashSet<jobject>) {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.push(to_push),
            FrameInfo::JavaFrame { .. } => panic!(),
        }
    }

    pub fn pop(&mut self) -> Option<HashSet<jobject>> {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.pop(),
            FrameInfo::JavaFrame { .. } => panic!(),
        }
    }

    pub fn push(&mut self, jobjects: HashSet<jobject>) {
        match self.get_frame_info_mut() {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { native_local_refs, .. } => native_local_refs.push(jobjects),
            FrameInfo::JavaFrame { .. } => panic!(),
        }
    }
}

pub struct StackIter<'vm_life, 'l> {
    jvm: &'vm_life JVMState<'vm_life>,
    current_frame: *mut c_void,
    java_stack: &'l JavaStack,
    top: *mut c_void,
    current_ip: Option<*mut c_void>,
}

impl<'l, 'k> StackIter<'l, 'k> {
    pub fn new(jvm: &'l JVMState<'l>, java_stack: &'k JavaStack) -> Self {
        Self {
            jvm,
            current_frame: java_stack.current_frame_ptr(),
            java_stack,
            top: java_stack.top,
            current_ip: None,
        }
    }
}

impl<'vm_life> Iterator for StackIter<'vm_life, '_> {
    type Item = StackEntry<'vm_life>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame != self.top && self.current_ip != Some(0x0000010080000000 as *mut c_void) {
            //todo cnstant
            let frame_view = FrameView::new(self.current_frame, todo!(), self.current_ip.unwrap_or(self.java_stack.saved_registers.unwrap().instruction_pointer));
            let header = frame_view.get_header();
            self.current_ip = Some(header.prev_rip);
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
    pub(crate) opaque_frame_id: Option<u64>,
    pub(crate) opaque_frame_optional: Option<OpaqueFrameOptional<'gc_life>>,
    pub(crate) non_native_data: Option<NonNativeFrameData>,
    pub(crate) local_vars: Vec<JavaValue<'gc_life>>,
    pub(crate) operand_stack: Vec<JavaValue<'gc_life>>,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
}

pub struct StackEntryMut<'gc_life, 'l> {
    pub frame_view: RuntimeJavaStackFrameMut<'l, 'gc_life>,
}

impl<'gc_life, 'l> StackEntryMut<'gc_life, 'l> {
    pub fn set_pc(&mut self, new_pc: u16) {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                *entry.pc_mut() = new_pc;
            }*/
            StackEntryMut::Jit { frame_view, .. } => {
                *frame_view.pc_mut() = new_pc;
            }
        }*/
        todo!()
    }

    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pc_offset_mut()
            }*/
            StackEntryMut::Jit { frame_view, .. } => frame_view.pc_offset_mut(),
        }*/
        todo!()
    }

    pub fn to_ref(&self) -> StackEntryRef<'gc_life, 'l> {
        StackEntryRef { frame_view: todo!() }
    }

    pub fn class_pointer(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
        self.to_ref().class_pointer(jvm).clone()
    }

    pub fn local_vars_mut(&'k mut self) -> LocalVarsMut<'gc_life, 'l, 'k> {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsMut::LegacyInterpreter { vars: entry.local_vars_mut(jvm) }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => LocalVarsMut::Jit { frame_view, jvm },
        }*/
        todo!()
    }

    pub fn local_vars(&'k self) -> LocalVarsRef<'gc_life, 'l, 'k> {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry } => {
                LocalVarsRef::LegacyInterpreter { vars: entry.local_vars_mut(jvm) }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => LocalVarsRef::Jit { frame_view, jvm },
        }*/
        todo!()
    }

    pub fn push(&mut self, j: JavaValue<'gc_life>) {
        todo!()
    }

    pub fn pop(&mut self, expected_type: Option<RuntimeType>) -> JavaValue<'gc_life> {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                entry.pop()
            }*/
            StackEntryMut::Jit { .. } => match self.operand_stack_mut().pop(expected_type) {
                Some(x) => x,
                None => {
                    panic!()
                }
            },
        }*/
        todo!()
    }

    pub fn operand_stack_mut(&'k mut self) -> OperandStackMut<'gc_life, 'l> {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                OperandStackMut::LegacyInterpreter { operand_stack: entry.operand_stack_mut() }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => OperandStackMut::Jit { frame_view, jvm },
        }*/
        todo!()
    }

    pub fn operand_stack_ref(&'k mut self, _jvm: &'gc_life JVMState<'gc_life>) -> OperandStackRef<'gc_life, 'l, 'k> {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                OperandStackRef::LegacyInterpreter { operand_stack: entry.operand_stack_mut() }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => OperandStackRef::Jit { frame_view, jvm },
        }*/
        todo!()
    }

    pub fn debug_extract_raw_frame_view(&'k mut self) -> &'k mut FrameView<'gc_life, 'l> {
        /*match self {
            StackEntryMut::Jit { frame_view, .. } => frame_view,
        }*/
        todo!()
    }

    pub fn set_pc_offset(&mut self, offset: i32) {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                *entry.pc_offset_mut() = offset
            }*/
            StackEntryMut::Jit { frame_view, .. } => {
                *frame_view.pc_offset_mut() = offset;
            }
        }*/
        todo!()
    }
}

//todo maybe I should do something about all the boilerplate but leaving as is for now
pub enum LocalVarsMut<'gc_life, 'l, 'k> {
    /*LegacyInterpreter {
        vars: &'l mut Vec<JavaValue<'gc_life>>
    },*/
    Jit { frame_view: &'k mut FrameView<'gc_life, 'l>, jvm: &'gc_life JVMState<'gc_life> },
}

impl<'gc_life, 'l, 'k> LocalVarsMut<'gc_life, 'l, 'k> {
    pub fn set(&mut self, i: u16, to: JavaValue<'gc_life>) {
        match self {
            /*LocalVarsMut::LegacyInterpreter { .. } => todo!(),*/
            LocalVarsMut::Jit { frame_view, jvm } => frame_view.set_local_var(jvm, i, to),
        }
    }
}

#[derive(Clone)]
pub enum LocalVarsRef<'gc_life, 'l, 'k> {
    /*    LegacyInterpreter {
        vars: &'l Vec<JavaValue>
    },*/
    Jit { frame_view: &'k FrameView<'gc_life, 'l>, jvm: &'gc_life JVMState<'gc_life> },
}

impl<'gc_life> LocalVarsRef<'gc_life, '_, '_> {
    pub fn get(&self, i: u16, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        match self {
            /*LocalVarsRef::LegacyInterpreter { .. } => todo!(),*/
            LocalVarsRef::Jit { frame_view, jvm } => frame_view.get_local_var(jvm, i, expected_type),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LocalVarsRef::Jit { frame_view, .. } => match frame_view.get_frame_info() {
                FrameInfo::FullyOpaque { .. } => panic!(),
                FrameInfo::Native { .. } => panic!(),
                FrameInfo::JavaFrame { num_locals, locals_types, .. } => {
                    assert_eq!(locals_types.len(), *num_locals as usize);

                    *num_locals as usize
                }
            },
        }
    }
}

pub enum OperandStackRef<'gc_life, 'l, 'k> {
    /*LegacyInterpreter {
        operand_stack: &'l Vec<JavaValue>
    },*/
    Jit { frame_view: &'k FrameView<'gc_life, 'l>, jvm: &'gc_life JVMState<'gc_life> },
}

impl<'gc_life, 'l, 'k> OperandStackRef<'gc_life, 'l, 'k> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> u16 {
        match self {
            /*OperandStackRef::LegacyInterpreter { .. } => todo!(),*/
            OperandStackRef::Jit { frame_view, jvm } => frame_view.operand_stack_length(jvm).unwrap(),
        }
    }

    pub fn last(&self) -> Option<&JavaValue<'gc_life>> {
        todo!()
    }

    pub fn get(&'_ self, from_start: u16, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        match self {
            /*OperandStackRef::LegacyInterpreter { .. } => todo!(),*/
            OperandStackRef::Jit { frame_view, jvm } => frame_view.get_operand_stack(jvm, from_start, expected_type),
        }
    }

    pub fn types(&self) -> Vec<RuntimeType> {
        match self {
            OperandStackRef::Jit { frame_view, jvm } => frame_view.stack_types(jvm).unwrap(),
        }
    }
    pub fn types_vals(&self) -> Vec<JavaValue<'gc_life>> {
        match self {
            OperandStackRef::Jit { frame_view, jvm } => frame_view.as_stack_entry_partially_correct(jvm).operand_stack.clone(),
        }
    }
}

pub struct  OperandStackMut<'gc_life, 'l> {
    pub(crate) frame_view: RuntimeJavaStackFrameMut<'l, 'gc_life>,
}

impl OperandStackMut<'gc_life, 'l> {
    pub fn push(&mut self, j: JavaValue<'gc_life>) {
        /*match self {
            /*OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.push(j);
            }*/
            OperandStackMut::Jit { frame_view, .. } => frame_view.push_operand_stack(j),
        }*/
        todo!()
    }

    pub fn pop(&mut self, rtype: Option<RuntimeType>) -> Option<JavaValue<'gc_life>> {
        /*match self {
            /*OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.pop()
            }*/
            OperandStackMut::Jit { frame_view, jvm } => frame_view.pop_operand_stack(jvm, rtype),
        }*/
        todo!()
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
        /*(match self {
            /*OperandStackMut::LegacyInterpreter { .. } => todo!(),*/
            OperandStackMut::Jit { frame_view, jvm } => frame_view.operand_stack_length(jvm).unwrap(),
        }) as usize*/
        todo!()
    }
}

pub struct StackEntryRef<'gc_life, 'l> {
    pub(crate) frame_view: RuntimeJavaStackFrameRef<'l, 'gc_life>,
}

impl<'gc_life, 'l> StackEntryRef<'gc_life, 'l> {
    pub fn frame_view(&self, jvm: &'gc_life JVMState<'gc_life>) -> &FrameView<'gc_life, 'l> {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.loader()
            }*/
            StackEntryRef::Jit { frame_view, .. } => frame_view,
        }*/
        todo!()
    }

    pub fn loader(&self, jvm: &'gc_life JVMState<'gc_life>) -> LoaderName {
        let method_id = match self.frame_view.ir_ref.method_id() {
            Some(x) => x,
            None => {
                //opaque frame todo, should lookup loader by opaque id
                return LoaderName::BootstrapLoader;
            }
        };
        let rc = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0;
        jvm.classes.read().unwrap().get_initiating_loader(&rc)
    }

    pub fn try_class_pointer(&self, jvm: &'gc_life JVMState<'gc_life>) -> Option<Arc<RuntimeClass<'gc_life>>> {
        let method_id = self.frame_view.ir_ref.method_id()?;
        let rc = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0;
        Some(rc)
    }

    pub fn class_pointer(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
        self.try_class_pointer(jvm).unwrap()
    }

    pub fn pc(&self, jvm: &'gc_life JVMState<'gc_life>) -> ByteCodeOffset {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => {
                entry.pc()
            }*/
            StackEntryRef::Jit { frame_view, .. } => frame_view.pc(jvm).unwrap(),
        }*/
        todo!()
    }

    pub fn pc_offset(&self) -> i32 {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => { entry.pc_offset() }*/
            StackEntryRef::Jit { frame_view, .. } => frame_view.pc_offset(),
        }*/
        todo!()
    }

    pub fn method_i(&self, jvm: &'gc_life JVMState<'gc_life>) -> CPIndex {
        let method_id = self.frame_view.ir_ref.method_id().unwrap();
        jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().1
    }

    pub fn operand_stack(&'k self, jvm: &'gc_life JVMState<'gc_life>) -> OperandStackRef<'gc_life, 'l, 'k> {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { .. } => todo!(),*/
            StackEntryRef::Jit { frame_view, .. } => OperandStackRef::Jit { frame_view, jvm },
        }*/
        OperandStackRef::Jit { frame_view: self.frame_view(jvm), jvm }
    }

    pub fn is_native(&self) -> bool {
        let res = match self.frame_view.ir_ref.method_id() {
            None => false,
            Some(method_id) => {
                self.frame_view.jvm.is_native_by_method_id(method_id)
            }
        };

        if res {
            assert!(self.frame_view.ir_ref.ir_method_id().is_none());
        }
        res
    }

    pub fn native_local_refs(&self) -> &mut Vec<BiMap<ByAddress<GcManagedObject<'gc_life>>, jobject>> {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => todo!("{:?}", entry),*/
            StackEntryRef::Jit { frame_view, .. } => todo!("{:?}", frame_view),
        }*/
        todo!()
    }

    pub fn local_vars(&'l self, jvm: &'gc_life JVMState<'gc_life>) -> LocalVarsRef<'gc_life, 'l, 'k> {
        /*match self {
            /*            StackEntryRef::LegacyInterpreter { entry } => {
                            LocalVarsRef::LegacyInterpreter { vars: entry.local_vars(jvm) }
                        }
            */
            StackEntryRef::Jit { frame_view } => LocalVarsRef::Jit { frame_view, jvm },
        }*/
        todo!()
    }
}

impl<'gc_life> StackEntry<'gc_life> {
    pub fn new_completely_opaque_frame(loader: LoaderName, operand_stack: Vec<JavaValue<'gc_life>>) -> Self {
        //need a better name here
        Self {
            loader,
            opaque_frame_id: Some(0),
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
        let class_view = class_pointer.view();
        let method_view = class_view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        let operand_stack = (0..code.max_stack).map(|_| JavaValue::Top).collect_vec();
        Self {
            loader,
            opaque_frame_id: Some(1),
            opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
            non_native_data: Some(NonNativeFrameData { pc: 0, pc_offset: 0 }),
            local_vars: args,
            operand_stack,
            native_local_refs: vec![],
        }
    }

    pub fn new_native_frame(jvm: &'gc_life JVMState<'gc_life>, class_pointer: Arc<RuntimeClass<'gc_life>>, method_i: u16, args: Vec<JavaValue<'gc_life>>) -> Self {
        Self {
            loader: jvm.classes.read().unwrap().get_initiating_loader(&class_pointer),
            opaque_frame_id: Some(2),
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
        }
            .class_pointer
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