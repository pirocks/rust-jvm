use std::ffi::{VaList, VaListImpl};

use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jboolean, jint, jlong, jmethodID, JNINativeInterface_, jobject, jshort, jvalue};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::MethodId;

use crate::class_loading::check_initing_or_inited_class;
// use log::trace;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use another_jit_vm_ir::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::JavaValueCommon;
use crate::jvm_state::JVMState;
use method_table::from_jmethod_id;
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::interface::{push_type_to_operand_stack, push_type_to_operand_stack_new};
use crate::rust_jni::native_util::{from_object_new, get_interpreter_state, get_state};

pub mod call_nonstatic;
pub mod call_nonvirtual;

unsafe fn call_nonstatic_method<'gc>(env: *mut *const JNINativeInterface_, obj: jobject, method_id: jmethodID, mut l: VarargProvider) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let method_id = from_jmethod_id(method_id);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap(); //todo should really return error instead of unwrap
    let classview = class.view().clone();
    let method = &classview.method_view_i(method_i);
    if method.is_static() {
        unimplemented!()
    }
    let parsed = method.desc();
    let mut args = vec![];
    args.push(NewJavaValueHandle::Object(from_object_new(jvm, obj).unwrap()));
    for type_ in &parsed.arg_types {
        args.push(push_type_to_operand_stack_new(jvm, int_state, type_, &mut l));
    }
    let not_handles = args.iter().map(|handle| handle.as_njv()).collect_vec();
    let res = invoke_virtual_method_i(jvm, int_state, parsed, class, &method, not_handles)?;
    assert!(int_state.throw().is_none());
    return Ok(res);
}

pub unsafe fn call_static_method_impl<'gc>(env: *mut *const JNINativeInterface_, jmethod_id: jmethodID, mut l: VarargProvider) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let method_id = *(jmethod_id as *mut MethodId);
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap(); //todo should really return error instead of lookup
    check_initing_or_inited_class(jvm, int_state, class.cpdtype())?;
    let classfile = &class.view();
    let method = &classfile.method_view_i(method_i);
    let parsed = method.desc();
    let args = push_params_onto_frame_new(jvm, &mut l, int_state, &parsed);
    let not_handles = args.iter().map(|handle| handle.as_njv()).collect();
    let res = invoke_static_impl(jvm, int_state, parsed, class.clone(), method_i, method, not_handles)?;
    Ok(if method.desc().return_type == CPDType::VoidType {
        assert!(res.is_none());
        None
    } else {
        assert!(res.is_some());
        res
    })
}

unsafe fn push_params_onto_frame_new<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    l: &mut VarargProvider,
    int_state: &'_ mut InterpreterStateGuard<'gc, 'l>,
    parsed: &CMethodDescriptor,
) -> Vec<NewJavaValueHandle<'gc>> {
    let mut args = vec![];
    for type_ in &parsed.arg_types {
        args.push(push_type_to_operand_stack_new(jvm, int_state, type_, l));
    }
    args
}

unsafe fn push_params_onto_frame<'gc, 'l>(jvm: &'gc JVMState<'gc>, l: &mut VarargProvider, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, parsed: &CMethodDescriptor) {
    for type_ in &parsed.arg_types {
        push_type_to_operand_stack(jvm, int_state, type_, l)
    }
}

pub mod call_static;

pub enum VarargProvider<'l, 'l2, 'l3> {
    Dots(&'l mut VaListImpl<'l2>),
    VaList(&'l mut VaList<'l2, 'l3>),
    Array(*const jvalue),
}

impl VarargProvider<'_, '_, '_> {
    pub unsafe fn arg_ptr(&mut self) -> jobject {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).l;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }
    pub unsafe fn arg_bool(&mut self) -> jboolean {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).z;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }
    pub unsafe fn arg_short(&mut self) -> jshort {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).s;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_long(&mut self) -> jlong {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).j;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_int(&mut self) -> jint {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).i;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_float(&mut self) -> f32 {
        match self {
            VarargProvider::Dots(l) => f32::from_bits(l.arg::<u32>()),
            VarargProvider::VaList(l) => f32::from_bits(l.arg::<u32>()),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).f;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_double(&mut self) -> f64 {
        match self {
            VarargProvider::Dots(l) => f64::from_bits(l.arg::<u64>()),
            VarargProvider::VaList(l) => f64::from_bits(l.arg::<u64>()),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).d;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_byte(&mut self) -> i8 {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).b;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_char(&mut self) -> u16 {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).c;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }
}