use std::ffi::{VaList, VaListImpl};

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jboolean, jint, jlong, jmethodID, JNINativeInterface_, jobject, jshort, jvalue};
use rust_jvm_common::descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use rust_jvm_common::ptype::PType;

// use log::trace;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::method_table::{from_jmethod_id, MethodId};
use crate::rust_jni::interface::push_type_to_operand_stack;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

pub mod call_nonstatic;
pub mod call_nonvirtual;


unsafe fn call_nonstatic_method(env: *mut *const JNINativeInterface_, obj: jobject, method_id: jmethodID, mut l: VarargProvider) -> Result<Option<JavaValue>, WasException> {
    let method_id = from_jmethod_id(method_id);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();//todo should really return error instead of unwrap
    let classview = class.view().clone();
    let method = &classview.method_view_i(method_i as usize);
    if method.is_static() {
        unimplemented!()
    }
    let parsed = method.desc();
    int_state.push_current_operand_stack(JavaValue::Object(from_object(obj)));
    for type_ in &parsed.parameter_types {
        push_type_to_operand_stack(int_state, type_, &mut l)
    }
    invoke_virtual_method_i(jvm, int_state, parsed, class, &method)?;
    assert!(int_state.throw().is_none());
    Ok(if method.desc().return_type == PType::VoidType {
        None
    } else {
        int_state.pop_current_operand_stack().into()
    })
}

pub unsafe fn call_static_method_impl(env: *mut *const JNINativeInterface_, jmethod_id: jmethodID, mut l: VarargProvider) -> Result<Option<JavaValue>, WasException> {
    let method_id = *(jmethod_id as *mut MethodId);
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();//todo should really return error instead of lookup
    let classfile = &class.view();
    let method = &classfile.method_view_i(method_i as usize);
    let method_descriptor_str = method.desc_str();
    let _name = method.name();
    let parsed = parse_method_descriptor(method_descriptor_str.as_str()).unwrap();
    push_params_onto_frame(&mut l, int_state, &parsed);
    invoke_static_impl(jvm, int_state, parsed, class.clone(), method_i as usize, method)?;
    Ok(if method.desc().return_type == PType::VoidType {
        None
    } else {
        int_state.pop_current_operand_stack().into()
    })
}

unsafe fn push_params_onto_frame(
    l: &mut VarargProvider,
    int_state: &mut InterpreterStateGuard,
    parsed: &MethodDescriptor,
) {
    for type_ in &parsed.parameter_types {
        push_type_to_operand_stack(int_state, type_, l)
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