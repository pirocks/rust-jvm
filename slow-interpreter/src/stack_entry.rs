use std::collections::HashSet;
use std::ffi::c_void;
use std::intrinsics::size_of;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;
use itertools::Itertools;

use another_jit_vm::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use another_jit_vm_ir::ir_stack::IsOpaque;
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::frame_layout::{FrameHeader, NativeStackframeMemoryLayout};
use java5_verifier::SimplifiedVType;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::{ByteCodeOffset, MethodId, StackNativeJavaValue};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::opaque_id_table::OpaqueID;
use rust_jvm_common::runtime_type::RuntimeType;
use rust_jvm_common::vtype::VType;

use crate::interpreter::real_interpreter_state::InterpreterJavaValue;
use crate::interpreter_state::{NativeFrameInfo, OpaqueFrameInfo};
use crate::ir_to_java_layer::java_stack::{OpaqueFrameIdOrMethodID, OwnedJavaStack, RuntimeJavaStackFrameMut, RuntimeJavaStackFrameRef};
use crate::java_values::{GcManagedObject, JavaValue, native_to_new_java_value_rtype};
use crate::jit::state::Opaque;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::NewJavaValue;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RuntimeClassClassId(usize);

#[derive(Clone, Copy)]
pub struct FrameView<'gc, 'l> {
    frame_ptr: *mut c_void,
    _call_stack: &'l OwnedJavaStack<'gc>,
    phantom_data: PhantomData<&'gc ()>,
}

impl<'gc, 'l> FrameView<'gc, 'l> {
    pub fn new(ptr: *mut c_void, call_stack: &'l mut OwnedJavaStack<'gc>) -> Self {
        let res = Self { frame_ptr: ptr, _call_stack: call_stack, phantom_data: PhantomData::default() };
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

    pub fn loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
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

    pub fn pc(&self, jvm: &'gc JVMState<'gc>) -> Result<ByteCodeOffset, Opaque> {
        let methodid = self.get_header().methodid;
        let byte_code_offset = todo!()/*jvm.jit_state.with(|jit_state| jit_state.borrow().ip_to_bytecode_pc(self.current_ip))?*/;
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        /*Ok(byte_code_offset.1)*/
        todo!()
    }

    pub fn operand_stack_length(&self, jvm: &'gc JVMState<'gc>) -> Result<u16, Opaque> {
        let methodid = self.get_header().methodid;
        if methodid == usize::MAX {
            panic!()
        }
        let pc = self.pc(jvm)?;

        let function_frame_type = &jvm.function_frame_type_data.read().unwrap().no_tops;
        let this_function = function_frame_type.get(&methodid).unwrap();
        //todo issue here is that we can't use instruct pointer b/c we might have iterated up through vm call and instruct pointer will be saved.
        Ok(this_function.get(&pc).unwrap().unwrap_full_frame().stack_map.len() as u16)
    }

    pub fn stack_types(&self, jvm: &'gc JVMState<'gc>) -> Result<Vec<RuntimeType>, Opaque> {
        let methodid = self.get_header().methodid;
        if methodid == usize::MAX {
            panic!()
        }
        let pc = self.pc(jvm)?;
        let function_frame_type = &jvm.function_frame_type_data.read().unwrap().no_tops;
        let stack_map = &function_frame_type.get(&methodid).unwrap().get(&pc).unwrap().unwrap_full_frame().stack_map;
        let mut res = vec![];
        for vtype in &stack_map.data {
            res.push(vtype.to_runtime_type());
        }
        Ok(res)
    }

    fn max_locals(&self, jvm: &'gc JVMState<'gc>) -> Result<u16, Opaque> {
        let methodid = self.get_header().methodid;
        if methodid == usize::MAX {
            panic!()
        }
        let pc = self.pc(jvm)?;
        let function_frame_type = &jvm.function_frame_type_data.read().unwrap().tops;
        let locals = &function_frame_type.get(&methodid).unwrap().get(&pc).unwrap().unwrap_full_frame().locals;
        Ok(locals.len() as u16)
    }

    fn get_operand_stack_base(&self, jvm: &'gc JVMState<'gc>) -> Result<*mut c_void, Opaque> {
        //todo should be based of actual layout instead of this
        Ok(unsafe { self.frame_ptr.offset((size_of::<FrameHeader>() + size_of::<u64>() * (self.max_locals(jvm)? as usize)) as isize) })
    }

    fn get_local_var_base(&self) -> *mut c_void {
        //todo should be based of actual layout instead of this
        unsafe { self.frame_ptr.offset((size_of::<FrameHeader>()) as isize) }
    }

    pub(crate) fn write_target(target: *mut c_void, j: JavaValue<'gc>) {
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

    pub(crate) fn read_target(jvm: &'gc JVMState<'gc>, target: *const c_void, expected_type: RuntimeType) -> JavaValue<'gc> {
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

    pub fn set_local_var(&mut self, _jvm: &'gc JVMState<'gc>, i: u16, jv: JavaValue<'gc>) {
        let target = unsafe { self.get_local_var_base().offset((i as isize) * size_of::<jlong>() as isize) };
        Self::write_target(target, jv)
    }

    pub fn get_local_var(&self, jvm: &'gc JVMState<'gc>, i: u16, expected_type: RuntimeType) -> JavaValue<'gc> {
        let target = unsafe { self.get_local_var_base().offset((i as isize) * size_of::<jlong>() as isize) };
        Self::read_target(jvm, target, expected_type)
    }

    pub fn get_operand_stack(&self, jvm: &'gc JVMState<'gc>, from_start: u16, expected_type: RuntimeType) -> JavaValue<'gc> {
        let operand_stack_base = self.get_operand_stack_base(jvm).unwrap();
        let target = unsafe { operand_stack_base.offset((from_start as isize) * size_of::<jlong>() as isize) };
        Self::read_target(jvm, target, expected_type)
    }

    pub fn set_operand_stack(&mut self, from_start: u16, to_set: JavaValue<'gc>) {
        let target = unsafe { self.get_operand_stack_base(todo!()).unwrap().offset((from_start as isize) * size_of::<jlong>() as isize) };
        Self::write_target(target, to_set)
    }

    pub fn as_stack_entry_partially_correct(&self, jvm: &'gc JVMState<'gc>) -> StackEntry {
        match self.operand_stack_length(jvm) {
            Ok(_) => {}
            Err(_) => {
                return todo!()/*StackEntry::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "as_stack_entry_partially_correct")*/;
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
        /*StackEntry {
            loader,
            opaque_frame_id: None,
            opaque_frame_optional: Some(OpaqueFrameOptional { class_pointer, method_i }),
            non_native_data: Some(NonNativeFrameData { pc: self.pc(jvm).unwrap().0, pc_offset: 0xDDDDDDD }),
            local_vars: vec![],
            operand_stack,
            native_local_refs: Default::default(),
        }*/
        todo!()
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
}

/// If the frame is opaque then this data is optional.
/// This data would typically be present in a native function call, but not be present in JVMTI frames
#[derive(Debug, Clone)]
pub struct OpaqueFrameOptional<'gc> {
    pub class_pointer: Arc<RuntimeClass<'gc>>,
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

#[derive(Clone)]
pub struct JavaFramePush<'gc, 'k> {
    pub(crate) method_id: MethodId,
    pub(crate) local_vars: Vec<NewJavaValue<'gc, 'k>>,
    pub(crate) operand_stack: Vec<NewJavaValue<'gc, 'k>>,
}

#[derive(Clone)]
pub struct NativeFramePush<'gc, 'k> {
    pub(crate) method_id: MethodId,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
    pub(crate) local_vars: Vec<NewJavaValue<'gc, 'k>>,
    pub(crate) operand_stack: Vec<NewJavaValue<'gc, 'k>>,
}

#[derive(Clone)]
pub struct OpaqueFramePush {
    pub(crate) opaque_id: OpaqueID,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
}

#[derive(Clone)]
pub enum StackEntryPush<'gc, 'k> {
    Java(JavaFramePush<'gc, 'k>),
    // a native function call frame
    Native(NativeFramePush<'gc, 'k>),
    Opaque(OpaqueFramePush),
}

impl<'gc, 'k> StackEntryPush<'gc, 'k> {
    pub fn new_native_frame(jvm: &'gc JVMState<'gc>, class_pointer: Arc<RuntimeClass<'gc>>, method_i: u16, args: Vec<NewJavaValue<'gc, 'k>>) -> NativeFramePush<'gc, 'k> {
        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer, method_i);
        NativeFramePush {
            method_id,
            native_local_refs: vec![HashSet::new()],
            local_vars: args,
            operand_stack: vec![],
        }
    }

    pub fn new_java_frame(jvm: &'gc JVMState<'gc>, class_pointer: Arc<RuntimeClass<'gc>>, method_i: u16, args: Vec<NewJavaValue<'gc, 'k>>) -> JavaFramePush<'gc, 'k> {
        let max_locals = class_pointer.view().method_view_i(method_i).code_attribute().unwrap().max_locals;
        assert_eq!(args.len(), max_locals as usize);
        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
        // assert!(jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }).is_some());
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&class_pointer);
        let mut guard = jvm.method_table.write().unwrap();
        let _method_id = guard.get_method_id(class_pointer.clone(), method_i);
        let class_view = class_pointer.view();
        let method_view = class_view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        let operand_stack = (0..code.max_stack).map(|_| NewJavaValue::Top).collect_vec();
        JavaFramePush {
            method_id,
            local_vars: args,
            operand_stack,
        }
    }

    pub fn new_completely_opaque_frame(jvm: &'gc JVMState<'gc>, loader: LoaderName, operand_stack: Vec<JavaValue<'gc>>, debug_str: &'static str) -> OpaqueFramePush {
        //need a better name here
        assert!(operand_stack.is_empty());
        assert_eq!(loader, LoaderName::BootstrapLoader);// loader should be set from thread loader for new threads
        let opaque_id = jvm.opaque_ids.write().unwrap().new_opaque_id(debug_str);
        OpaqueFramePush {
            opaque_id,
            native_local_refs: vec![],
        }
    }
}


#[derive(Debug, Clone)]
pub enum StackEntry {
    Java {
        method_id: MethodId,
    },
    Native {
        // a native function call frame
        method_id: MethodId,
    },
    Opaque {
        opaque_id: OpaqueID,
    },
}
//
// #[derive(Debug, Clone)]
// pub struct StackEntry<'gc> {
//     pub(crate) loader: LoaderName,
//     pub(crate) opaque_frame_id: Option<u64>,
//     pub(crate) opaque_frame_optional: Option<OpaqueFrameOptional<'gc>>,
//     pub(crate) non_native_data: Option<NonNativeFrameData>,
//     pub(crate) local_vars: Vec<JavaValue<'gc>>,
//     pub(crate) operand_stack: Vec<JavaValue<'gc>>,
//     pub(crate) native_local_refs: Vec<HashSet<jobject>>,
// }

pub struct StackEntryMut<'gc, 'l> {
    pub frame_view: RuntimeJavaStackFrameMut<'gc, 'l>,
}

impl<'gc, 'l> StackEntryMut<'gc, 'l> {
    pub fn local_vars_mut<'k>(&'k mut self, jvm: &'gc JVMState<'gc>) -> LocalVarsMut<'gc, 'l, 'k> {
        LocalVarsMut::Jit { frame_view: &mut self.frame_view, jvm }
    }

    pub fn local_vars<'k>(&'k self, jvm: &'gc JVMState<'gc>) -> LocalVarsRef<'gc, 'l, 'k> {
        todo!()/*LocalVarsRef::Jit {
            frame_view: &self.frame_view.downgrade(),
            jvm,
            pc: None,
        }*/
    }

    pub fn push(&mut self, j: JavaValue<'gc>) {
        todo!()
    }

    pub fn pop(&mut self, expected_type: Option<RuntimeType>) -> JavaValue<'gc> {
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

    pub fn operand_stack_mut(self) -> OperandStackMut<'gc, 'l> {
        OperandStackMut { frame_view: self.frame_view }
    }

    pub fn operand_stack_ref<'k>(&'k mut self, _jvm: &'gc JVMState<'gc>) -> OperandStackRef<'gc, 'l, 'k> {
        /*match self {
            /*StackEntryMut::LegacyInterpreter { entry, .. } => {
                OperandStackRef::LegacyInterpreter { operand_stack: entry.operand_stack_mut() }
            }*/
            StackEntryMut::Jit { frame_view, jvm } => OperandStackRef::Jit { frame_view, jvm },
        }*/
        todo!()
    }

    pub fn debug_extract_raw_frame_view<'k>(&'k mut self) -> &'k mut FrameView<'gc, 'l> {
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
pub enum LocalVarsMut<'gc, 'l, 'k> {
    /*LegacyInterpreter {
        vars: &'l mut Vec<JavaValue<'gc>>
    },*/
    Jit {
        frame_view: &'k mut RuntimeJavaStackFrameMut<'gc, 'l>,
        jvm: &'gc JVMState<'gc>,
    },
}

impl<'gc, 'l, 'k> LocalVarsMut<'gc, 'l, 'k> {
    pub fn set<'irrelevant>(&mut self, i: u16, to: NewJavaValue<'gc, 'irrelevant>) {
        match self {
            /*LocalVarsMut::LegacyInterpreter { .. } => todo!(),*/
            LocalVarsMut::Jit { frame_view, jvm } => todo!()/*frame_view.set_local_var(jvm, i, to.to_jv()),*/
        }
    }

    //todo move this and/or refactor
    pub fn interpreter_set(&mut self, i: u16, ijv: InterpreterJavaValue) {
        unsafe { self.raw_set(i, ijv.to_raw()) };
    }

    pub unsafe fn raw_set(&mut self, i: u16, raw: u64) {
        match self {
            LocalVarsMut::Jit { jvm, frame_view, } => {
                frame_view.ir_mut.write_data(i as usize, raw)
            }
        }
    }
}

#[derive(Clone)]
pub enum LocalVarsRef<'gc, 'l, 'k> {
    /*    LegacyInterpreter {
        vars: &'l Vec<JavaValue>
    },*/
    Jit {
        frame_view: &'k RuntimeJavaStackFrameRef<'gc, 'l>,
        jvm: &'gc JVMState<'gc>,
        pc: Option<ByteCodeOffset>,
    },
}

impl<'gc> LocalVarsRef<'gc, '_, '_> {
    pub fn get(&self, i: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        match self {
            LocalVarsRef::Jit { frame_view, jvm, pc } => {
                let method_id = frame_view.ir_ref.method_id().expect("local vars should have method id probably");
                let num_args = self.num_vars();
                assert!(i < num_args);
                if jvm.is_native_by_method_id(method_id) {
                    if let Some(pc) = *pc {
                        let function_frame_data_guard = &jvm.function_frame_type_data.read().unwrap().tops;
                        let function_frame_data = match function_frame_data_guard.get(&method_id) {
                            Some(function_frame_data) => {
                                let frame = function_frame_data.get(&pc).unwrap();
                                let verification_data_expected_frame_data = frame.unwrap_full_frame().locals[i as usize].to_runtime_type();
                                assert_eq!(verification_data_expected_frame_data, expected_type);
                            }
                            None => {}
                        };
                    }
                }
                let data = frame_view.ir_ref.data(i as usize);//todo replace this with a layout lookup thing again
                let native_jv = StackNativeJavaValue { as_u64: data };
                native_to_new_java_value_rtype(native_jv, expected_type, jvm)
            }
        }
    }

    pub fn interpreter_get(&self, i: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        let raw = unsafe { self.raw_get(i) };
        InterpreterJavaValue::from_raw(raw, rtype)
    }

    pub unsafe fn raw_get(&self, i: u16) -> u64 {
        match self {
            LocalVarsRef::Jit { jvm, frame_view, pc } => {
                frame_view.ir_ref.data(i as usize)
            }
        }
    }

    pub fn num_vars(&self) -> u16 {
        match self {
            LocalVarsRef::Jit { frame_view, jvm, pc } => {
                let method_id = frame_view.ir_ref.method_id().expect("local vars should have method id probably");
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let num_args = method_view.code_attribute().map(|code| code.max_locals).unwrap_or(method_view.num_args());
                num_args
            }
        }
    }
}

pub enum OperandStackRef<'gc, 'l, 'k> {
    /*LegacyInterpreter {
        operand_stack: &'l Vec<JavaValue>
    },*/
    Jit {
        frame_view: &'k RuntimeJavaStackFrameRef<'gc, 'l>,
        jvm: &'gc JVMState<'gc>,
        pc: Option<ByteCodeOffset>,
    },
}

impl<'gc, 'l, 'k> OperandStackRef<'gc, 'l, 'k> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> u16 {
        let types_len = self.types().len();
        /*match self {
            /*OperandStackRef::LegacyInterpreter { .. } => todo!(),*/
            OperandStackRef::Jit { frame_view, jvm, pc } => {

            }
        }*/
        types_len as u16
    }

    pub fn last(&self) -> Option<&JavaValue<'gc>> {
        todo!()
    }

    pub unsafe fn raw_get(&self, from_start: u16) -> StackNativeJavaValue {
        match self {
            OperandStackRef::Jit { frame_view, jvm, pc } => {
                let num_locals = LocalVarsRef::Jit {
                    frame_view,
                    jvm,
                    pc: *pc,
                }.num_vars();
                let data = frame_view.ir_ref.data((num_locals as usize + from_start as usize) as usize);//todo replace this with a layout lookup thing again
                StackNativeJavaValue { as_u64: data }
            }
        }
    }


    fn num_locals(&self) -> u16 {
        match self {
            OperandStackRef::Jit { frame_view, jvm, pc } => {
                LocalVarsRef::Jit {
                    frame_view,
                    jvm,
                    pc: *pc,
                }.num_vars()
            }
        }
    }

    pub fn interpreter_get(&self, from_start: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        let raw = unsafe { self.raw_get(from_start).as_u64 };
        InterpreterJavaValue::from_raw(raw, rtype)
    }

    pub fn get(&'_ self, from_start: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        match self {
            OperandStackRef::Jit { frame_view, jvm, pc } => {
                let num_locals = self.num_locals();
                let data = frame_view.ir_ref.data((num_locals as usize + from_start as usize) as usize);//todo replace this with a layout lookup thing again
                assert_eq!(self.types().iter().nth(from_start as usize).unwrap(), &expected_type);
                let native_jv = StackNativeJavaValue { as_u64: data };
                native_to_new_java_value_rtype(native_jv,expected_type, jvm)
            }
        }
    }

    pub fn get_from_end(&self, from_end: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        match self {
            OperandStackRef::Jit { frame_view, jvm, pc } => {
                let num_locals = self.num_locals();
                let types_len = self.types().len();
                let from_start = types_len - from_end as usize - 1;
                let data = frame_view.ir_ref.data((num_locals as usize + from_start as usize) as usize);//todo replace this with a layout lookup thing again
                assert!(self.types().iter().rev().nth(from_end as usize).unwrap().compatible_with_dumb(&expected_type));
                let native_jv = StackNativeJavaValue { as_u64: data };
                native_to_new_java_value_rtype(native_jv, expected_type, jvm)
            }
        }
    }

    pub fn types(&self) -> Vec<RuntimeType> {
        match self {
            OperandStackRef::Jit { frame_view, jvm, pc } => {
                let method_id = frame_view.ir_ref.method_id().expect("local vars should have method id probably");
                let pc = pc.unwrap();
                let function_frame_data_guard = &jvm.function_frame_type_data.read().unwrap().no_tops;
                let function_frame_data = function_frame_data_guard.get(&method_id).unwrap();
                let frame = function_frame_data.get(&pc).unwrap();//todo this get frame thing is duped in a bunch of places
                frame.unwrap_full_frame().stack_map.data.iter().map(|vtype| vtype.to_runtime_type()).rev().collect()
            }
        }
    }


    pub fn simplified_types(&self) -> Vec<SimplifiedVType> {
        match self {
            OperandStackRef::Jit { frame_view, jvm, pc } => {
                let method_id = frame_view.ir_ref.method_id().expect("local vars should have method id probably");
                let pc = pc.unwrap();
                let function_frame_data_guard = &jvm.function_frame_type_data.read().unwrap().no_tops;
                let function_frame_data = function_frame_data_guard.get(&method_id).unwrap();
                let frame = function_frame_data.get(&pc).unwrap();//todo this get frame thing is duped in a bunch of places
                frame.unwrap_partial_inferred_frame().operand_stack.iter().map(|vtype| *vtype).collect()
            }
        }
    }

    pub fn types_vals(&self) -> Vec<JavaValue<'gc>> {
        todo!()
        /*match self {
            OperandStackRef::Jit { frame_view, jvm } => frame_view.as_stack_entry_partially_correct(jvm).operand_stack.clone(),
        }*/
    }
}

pub struct OperandStackMut<'gc, 'l> {
    pub(crate) frame_view: RuntimeJavaStackFrameMut<'gc, 'l>,
}

impl<'gc, 'l> OperandStackMut<'gc, 'l> {
    pub fn push(&mut self, j: JavaValue<'gc>) {
        /*match self {
            /*OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.push(j);
            }*/
            OperandStackMut::Jit { frame_view, .. } => frame_view.push_operand_stack(j),
        }*/
        todo!()
    }

    pub fn pop(&mut self, rtype: Option<RuntimeType>) -> Option<JavaValue<'gc>> {
        /*match self {
            /*OperandStackMut::LegacyInterpreter { operand_stack, .. } => {
                operand_stack.pop()
            }*/
            OperandStackMut::Jit { frame_view, jvm } => frame_view.pop_operand_stack(jvm, rtype),
        }*/
        todo!()
    }

    pub fn insert(&mut self, index: usize, j: JavaValue<'gc>) {
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

    pub unsafe fn raw_write(&mut self, from_start: u16, val: u64) {
        let method_id = self.frame_view.downgrade().ir_ref.method_id().unwrap();
        let max_locals = self.frame_view.jvm.max_locals_by_method_id(method_id);
        self.frame_view.ir_mut.write_data((max_locals as usize + from_start as usize) as usize, val);//todo replace this with a layout lookup thing again
    }

    pub fn interpreter_set(&mut self, from_start: u16, val: InterpreterJavaValue) {
        let raw = val.to_raw();
        unsafe { self.raw_write(from_start, val.to_raw()) }
    }
}

pub struct StackEntryRef<'gc, 'l> {
    pub(crate) frame_view: RuntimeJavaStackFrameRef<'gc, 'l>,
    //todo in future may want to pass ir instr index info back down to ir layer, esp is relevant data is in registers
    pub(crate) pc: Option<ByteCodeOffset>,// option is for opaque frames or similar.
}

impl<'gc, 'l> StackEntryRef<'gc, 'l> {
    pub fn frame_view(&self, jvm: &'gc JVMState<'gc>) -> &FrameView<'gc, 'l> {
        todo!()
    }

    pub fn loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        let method_id = match self.frame_view.ir_ref.method_id() {
            Ok(x) => x,
            Err(IsOpaque {}) => {
                //opaque frame todo, should lookup loader by opaque id
                return LoaderName::BootstrapLoader;
            }
        };
        let rc = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0;
        jvm.classes.read().unwrap().get_initiating_loader(&rc)
    }

    pub fn try_class_pointer(&self, jvm: &'gc JVMState<'gc>) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        let method_id = self.frame_view.ir_ref.method_id()?;
        let rc = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0;
        Ok(rc)
    }

    pub fn class_pointer(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
        self.try_class_pointer(jvm).unwrap()
    }

    pub fn pc(&self, jvm: &'gc JVMState<'gc>) -> ByteCodeOffset {
        self.pc.unwrap()
    }

    pub fn try_pc(&self, jvm: &'gc JVMState<'gc>) -> Option<ByteCodeOffset> {
        self.pc
    }

    pub fn pc_offset(&self) -> i32 {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => { entry.pc_offset() }*/
            StackEntryRef::Jit { frame_view, .. } => frame_view.pc_offset(),
        }*/
        todo!()
    }

    pub fn method_i(&self, jvm: &'gc JVMState<'gc>) -> u16 {
        let method_id = self.frame_view.ir_ref.method_id().unwrap();
        jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().1
    }

    pub fn operand_stack<'k>(&'k self, jvm: &'gc JVMState<'gc>) -> OperandStackRef<'gc, 'l, 'k> {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { .. } => todo!(),*/
            StackEntryRef::Jit { frame_view, .. } => OperandStackRef::Jit { frame_view, jvm },
        }*/
        OperandStackRef::Jit { frame_view: &self.frame_view, jvm, pc: self.pc }
    }

    pub fn is_native_method(&self) -> bool {
        match self.frame_view.ir_ref.method_id() {
            Err(IsOpaque {}) => false,
            Ok(method_id) => {
                self.frame_view.jvm.is_native_by_method_id(method_id)
            }
        }
    }

    pub fn is_opaque(&self) -> bool {
        let opaque_frame_or_method_id = OpaqueFrameIdOrMethodID::from_native(self.frame_view.ir_ref.raw_method_id());
        opaque_frame_or_method_id.is_opaque()
    }


    pub fn native_local_refs(&self) -> &mut Vec<BiMap<ByAddress<GcManagedObject<'gc>>, jobject>> {
        /*match self {
            /*StackEntryRef::LegacyInterpreter { entry, .. } => todo!("{:?}", entry),*/
            StackEntryRef::Jit { frame_view, .. } => todo!("{:?}", frame_view),
        }*/
        todo!()
    }

    pub fn local_vars<'k>(&'k self, jvm: &'gc JVMState<'gc>) -> LocalVarsRef<'gc, 'l, 'k> {
        LocalVarsRef::Jit {
            frame_view: &self.frame_view,
            jvm,
            pc: self.pc,
        }
    }

    pub fn full_frame_available(&self, jvm: &'gc JVMState<'gc>) -> bool {
        let method_id = self.frame_view.ir_ref.method_id().unwrap();
        let pc = self.pc(jvm);
        let read_guard = &jvm.function_frame_type_data.read().unwrap().tops;
        let function_frame_type = read_guard.get(&method_id).unwrap();
        let frame = function_frame_type.get(&pc).unwrap();
        frame.try_unwrap_full_frame().is_some()
    }

    pub fn local_var_simplified_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<SimplifiedVType> {
        let method_id = self.frame_view.ir_ref.method_id().unwrap();
        let pc = self.pc(jvm);
        let read_guard = &jvm.function_frame_type_data.read().unwrap().tops;
        let function_frame_type = read_guard.get(&method_id).unwrap();
        function_frame_type.get(&pc).unwrap().unwrap_partial_inferred_frame().local_vars.clone()
    }

    pub fn local_var_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<VType> {
        let method_id = self.frame_view.ir_ref.method_id().unwrap();
        let pc = self.pc(jvm);
        let read_guard = &jvm.function_frame_type_data.read().unwrap().tops;
        let function_frame_type = read_guard.get(&method_id).unwrap();
        function_frame_type.get(&pc).unwrap().unwrap_full_frame().locals.deref().clone()
    }

    pub fn opaque_frame_ptr(&self) -> *mut OpaqueFrameInfo {
        assert!(self.is_opaque());
        let raw_u64 = self.frame_view.ir_ref.data(0);
        raw_u64 as usize as *mut c_void as *mut OpaqueFrameInfo
    }

    pub fn native_frame_ptr(&self) -> *mut NativeFrameInfo {
        assert!(self.is_native_method());
        let method_id = self.frame_view.ir_ref.method_id().unwrap();
        let max_locals = self.frame_view.jvm.num_local_vars_native(method_id);
        let layout = NativeStackframeMemoryLayout { num_locals: max_locals };
        let raw_u64 = self.frame_view.ir_ref.read_at_offset(layout.data_entry());
        raw_u64 as usize as *mut c_void as *mut NativeFrameInfo
    }

    pub fn ir_stack_entry_debug_print(&self) {
        let ir_ref = &self.frame_view.ir_ref;
        for i in 0..10 {
            unsafe { eprintln!("{:?}:{:?}", ir_ref.frame_ptr().as_ptr().sub(i * size_of::<u64>()), ir_ref.data(i as usize) as *const c_void); }
        }
        // dbg!(ir_ref.method_id());
        // dbg!(ir_ref.frame_ptr());
        // dbg!(ir_ref.ir_method_id());
        // dbg!(ir_ref.frame_size(&self.frame_view.jvm.java_vm_state.ir));
    }
}

impl<'gc> StackEntry {
    pub fn pop(&mut self) -> JavaValue<'gc> {
        todo!()
        /*self.operand_stack.pop().unwrap_or_else(|| {
            // let classfile = &self.class_pointer.classfile;
            // let method = &classfile.methods[self.method_i as usize];
            // dbg!(&method.method_name(&classfile));
            // dbg!(&method.code_attribute().unwrap().code);
            // dbg!(&self.pc);
            panic!()
        })*/
    }
    pub fn push(&mut self, j: JavaValue<'gc>) {
        todo!()
        /*self.operand_stack.push(j)*/
    }

    pub fn class_pointer(&self) -> &Arc<RuntimeClass<'gc>> {
        todo!()
        /*&match self.opaque_frame_optional.as_ref() {
            Some(x) => x,
            None => {
                unimplemented!()
            }
        }
            .class_pointer*/
    }

    pub fn try_class_pointer(&self, jvm: &'gc JVMState<'gc>) -> Option<Arc<RuntimeClass<'gc>>> {
        match self {
            StackEntry::Native { method_id } |
            StackEntry::Java { method_id } => {
                let (rc, method_id) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                return Some(rc);
            }
            StackEntry::Opaque { .. } => None,
        }
    }

    pub fn local_vars(&self) -> &Vec<JavaValue<'gc>> {
        todo!()
        /*&self.local_vars*/
    }

    pub fn local_vars_mut(&mut self) -> &mut Vec<JavaValue<'gc>> {
        todo!()
        /*&mut self.local_vars*/
    }

    pub fn operand_stack_mut(&mut self) -> &mut Vec<JavaValue<'gc>> {
        todo!()
        /*&mut self.operand_stack*/
    }

    pub fn operand_stack(&self) -> &Vec<JavaValue<'gc>> {
        todo!()
        /*&self.operand_stack*/
    }

    pub fn pc_mut(&mut self) -> &mut u16 {
        todo!()
        /*&mut self.non_native_data.as_mut().unwrap().pc*/
    }

    pub fn pc(&self) -> u16 {
        self.try_pc().unwrap()
    }

    pub fn try_pc(&self) -> Option<u16> {
        todo!()
        /*self.non_native_data.as_ref().map(|x| x.pc)*/
    }

    //todo a lot of duplication here between mut and non-mut variants
    pub fn pc_offset_mut(&mut self) -> &mut i32 {
        todo!()
        /*&mut self.non_native_data.as_mut().unwrap().pc_offset*/
    }

    pub fn pc_offset(&self) -> i32 {
        todo!()
        /*self.non_native_data.as_ref().unwrap().pc_offset*/
    }

    pub fn method_i(&self) -> CPIndex {
        todo!()
        /*self.opaque_frame_optional.as_ref().unwrap().method_i*/
    }

    pub fn try_method_i(&self) -> Option<CPIndex> {
        todo!()
        /*self.opaque_frame_optional.as_ref().map(|x| x.method_i)*/
    }

    pub fn is_native(&self) -> bool {
        let method_i = match self.try_method_i() {
            None => return true,
            Some(i) => i,
        };
        self.class_pointer().view().method_view_i(method_i).is_native()
    }

    pub fn convert_to_native(&mut self) {
        todo!()
        /*self.non_native_data.take();*/
    }

    pub fn operand_stack_types(&self) -> Vec<RuntimeType> {
        self.operand_stack().iter().map(|type_| type_.to_type()).collect()
    }

    pub fn local_vars_types(&self) -> Vec<RuntimeType> {
        self.local_vars().iter().map(|type_| type_.to_type()).collect()
    }

    pub fn loader(&self) -> LoaderName {
        todo!()
        /*self.loader*/
    }

    pub fn privileged_frame(&self) -> bool {
        todo!()
    }

    pub fn is_opaque_frame(&self, jvm: &'gc JVMState<'gc>) -> bool {
        self.try_class_pointer(jvm).is_none() || self.try_method_i().is_none() || self.is_native()
    }

    pub fn current_method_id(&self, jvm: &'gc JVMState<'gc>) -> Option<MethodId> {
        todo!()
        /*let optional = self.opaque_frame_optional.as_ref()?;
        let mut guard = jvm.method_table.write().unwrap();
        Some(guard.get_method_id(optional.class_pointer.clone(), optional.method_i))*/
    }
}

// impl<'gc> AsRef<StackEntry<'gc>> for StackEntry<'gc> {
//     fn as_ref(&self) -> &StackEntry<'gc> {
//         self
//     }
// }
//
// impl<'gc> AsMut<StackEntry<'gc>> for StackEntry<'gc> {
//     fn as_mut(&mut self) -> &mut StackEntry<'gc> {
//         self
//     }
// }