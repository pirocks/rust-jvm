use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::jint;
use rust_jvm_common::descriptor_parser::{MethodDescriptor, parse_method_descriptor};

use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::array_out_of_bounds_exception::ArrayOutOfBoundsException;
use crate::java::lang::boolean::Boolean;
use crate::java::lang::byte::Byte;
use crate::java::lang::char::Char;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::illegal_argument_exception::IllegalArgumentException;
use crate::java::lang::int::Int;
use crate::java::lang::long::Long;
use crate::java::lang::null_pointer_exception::NullPointerException;
use crate::java::lang::short::Short;
use crate::java_values::{ExceptionReturn, JavaValue, Object};
use crate::JVMState;
use crate::runtime_class::RuntimeClass;

pub fn lookup_method_parsed(state: &JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor) -> Option<(usize, Arc<RuntimeClass>)> {
    lookup_method_parsed_impl(state, int_state, class, name, descriptor)
}

pub fn lookup_method_parsed_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor) -> Option<(usize, Arc<RuntimeClass>)> {
    let view = class.view();
    let posible_methods = view.lookup_method_name(&name);
    let filtered = posible_methods.into_iter().filter(|m| {
        if m.is_signature_polymorphic() {
            true
        } else {
            &m.desc() == descriptor
        }
    }).collect::<Vec<_>>();
    assert!(filtered.len() <= 1);
    match filtered.iter().next() {
        None => {
            let class_name = class.view().super_name().unwrap();//todo is this unwrap safe?
            let lookup_type = PTypeView::Ref(ReferenceTypeView::Class(class_name));
            let super_class = assert_inited_or_initing_class(jvm, lookup_type); //todo this unwrap could fail, and this should really be using check_inited_class
            lookup_method_parsed_impl(jvm, int_state, super_class, name, descriptor)
        }
        Some(method_view) => {
            Some((method_view.method_i(), class.clone()))
        }
    }
}


pub fn string_obj_to_string(str_obj: Arc<Object>) -> String {
    let temp = str_obj.lookup_field("value");
    let chars = temp.unwrap_array();
    let borrowed_elems = chars.mut_array();
    char::decode_utf16(borrowed_elems.iter().map(|jv| jv.unwrap_char())).collect::<Result<String, _>>().expect("really weird string encountered")//todo so techincally java strings need not be valid so we can't return a rust string and have to do everything on bytes
}

pub fn throw_npe_res<T: ExceptionReturn>(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<T, WasException> {
    let _ = throw_npe::<T>(jvm, int_state);
    Err(WasException)
}

pub fn throw_npe<T: ExceptionReturn>(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> T {
    let npe_object = match NullPointerException::new(jvm, int_state) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating NPE");
            return T::invalid_default();
        }
    }.object().into();
    int_state.set_throw(npe_object);
    T::invalid_default()
}


pub fn throw_array_out_of_bounds_res<T: ExceptionReturn>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, index: jint) -> Result<T, WasException> {
    let _ = throw_array_out_of_bounds::<T>(jvm, int_state, index);
    Err(WasException)
}

pub fn throw_array_out_of_bounds<T: ExceptionReturn>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, index: jint) -> T {
    let bounds_object = match ArrayOutOfBoundsException::new(jvm, int_state, index) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating Array out of bounds");
            return T::invalid_default();
        }
    }.object().into();
    int_state.set_throw(bounds_object);
    T::invalid_default()
}

pub fn throw_illegal_arg_res<T: ExceptionReturn>(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<T, WasException> {
    let _ = throw_illegal_arg::<T>(jvm, int_state);
    Err(WasException)
}

pub fn throw_illegal_arg<T: ExceptionReturn>(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> T {
    let bounds_object = match IllegalArgumentException::new(jvm, int_state) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating illegal arg exception");
            return T::invalid_default();
        }
    }.object().into();
    int_state.set_throw(bounds_object);
    T::invalid_default()
}

pub fn java_value_to_boxed_object(jvm: &JVMState, int_state: &mut InterpreterStateGuard, java_value: JavaValue) -> Result<Option<Arc<Object>>, WasException> {
    Ok(match java_value {
        //todo what about that same object optimization
        JavaValue::Long(param) => Long::new(jvm, int_state, param)?.object().into(),
        JavaValue::Int(param) => Int::new(jvm, int_state, param)?.object().into(),
        JavaValue::Short(param) => Short::new(jvm, int_state, param)?.object().into(),
        JavaValue::Byte(param) => Byte::new(jvm, int_state, param)?.object().into(),
        JavaValue::Boolean(param) => Boolean::new(jvm, int_state, param)?.object().into(),
        JavaValue::Char(param) => Char::new(jvm, int_state, param)?.object().into(),
        JavaValue::Float(param) => Float::new(jvm, int_state, param)?.object().into(),
        JavaValue::Double(param) => Double::new(jvm, int_state, param)?.object().into(),
        JavaValue::Object(obj) => obj,
        JavaValue::Top => panic!()
    })
}


pub fn run_static_or_virtual(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: &Arc<RuntimeClass>, method_name: String, desc_str: String) -> Result<(), WasException> {
    let parsed_desc = parse_method_descriptor(desc_str.as_str()).unwrap();
    let view = class.view();
    let res_fun = view.lookup_method(&method_name, &parsed_desc);
    let method_view = match res_fun {
        Some(x) => x,
        None => panic!(),
    };
    let md = method_view.desc();
    if method_view.is_static() {
        invoke_static_impl(jvm, int_state, md, class.clone(), method_view.method_i(), &method_view)
    } else {
        invoke_virtual_method_i(jvm, int_state, md, class.clone(), &method_view)
    }
}


pub fn unwrap_or_npe<T>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, to_unwrap: Option<T>) -> Result<T, WasException> {
    match to_unwrap {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(unwrapped) => Ok(unwrapped)
    }
}