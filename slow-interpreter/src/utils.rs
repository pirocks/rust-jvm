use std::sync::Arc;

use libffi::high::arg;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::jint;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

use crate::{JVMState, NewJavaValue};
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
use crate::java_values::{ExceptionReturn, GcManagedObject, JavaValue};
use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle, NewJavaValueHandle};
use crate::runtime_class::RuntimeClass;

pub fn lookup_method_parsed(jvm: &'gc_life JVMState<'gc_life>, class: Arc<RuntimeClass<'gc_life>>, name: MethodName, descriptor: &CMethodDescriptor) -> Option<(u16, Arc<RuntimeClass<'gc_life>>)> {
    lookup_method_parsed_impl(jvm, class, name, descriptor)
}

pub fn lookup_method_parsed_impl(jvm: &'gc_life JVMState<'gc_life>, class: Arc<RuntimeClass<'gc_life>>, name: MethodName, descriptor: &CMethodDescriptor) -> Option<(u16, Arc<RuntimeClass<'gc_life>>)> {
    let view = class.view();
    let posible_methods = view.lookup_method_name(name);
    let filtered = posible_methods.into_iter().filter(|m| if m.is_signature_polymorphic() { true } else { m.desc() == descriptor }).collect::<Vec<_>>();
    assert!(filtered.len() <= 1);
    match filtered.iter().next() {
        None => {
            let class_name = class.view().super_name().unwrap(); //todo is this unwrap safe?
            let lookup_type = CPDType::Ref(CPRefType::Class(class_name));
            let super_class = assert_inited_or_initing_class(jvm, lookup_type); //todo this unwrap could fail, and this should really be using check_inited_class
            lookup_method_parsed_impl(jvm, super_class, name, descriptor)
        }
        Some(method_view) => Some((method_view.method_i(), class.clone())),
    }
}

pub fn string_obj_to_string<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, str_obj: AllocatedObject<'gc_life, '_>) -> String {
    let str_class_pointer = assert_inited_or_initing_class(jvm, CClassName::string().into());
    let temp = str_obj.lookup_field(&str_class_pointer, FieldName::field_value());
    let chars = temp.unwrap_array(jvm);
    let borrowed_elems = chars.array_iterator();
    char::decode_utf16(borrowed_elems.map(|jv| jv.as_njv().unwrap_char_strict())).collect::<Result<String, _>>().expect("really weird string encountered")
    //todo so techincally java strings need not be valid so we can't return a rust string and have to do everything on bytes
}

pub fn throw_npe_res<T: ExceptionReturn>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<T, WasException> {
    let _ = throw_npe::<T>(jvm, int_state);
    Err(WasException)
}

pub fn throw_npe<T: ExceptionReturn>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> T {
    todo!()
    /*let npe_object = match NullPointerException::new(jvm, int_state) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating NPE");
            return T::invalid_default();
        }
    }
        .object()
        .into();
    int_state.set_throw(Some(npe_object));
    T::invalid_default()*/
}

pub fn throw_array_out_of_bounds_res<T: ExceptionReturn>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, index: jint) -> Result<T, WasException> {
    let _ = throw_array_out_of_bounds::<T>(jvm, int_state, index);
    Err(WasException)
}

pub fn throw_array_out_of_bounds<T: ExceptionReturn>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, index: jint) -> T {
    /*let bounds_object = match ArrayOutOfBoundsException::new(jvm, int_state, index) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating Array out of bounds");
            return T::invalid_default();
        }
    }
        .object()
        .into();
    int_state.set_throw(Some(bounds_object));
    T::invalid_default()*/
    todo!()
}

pub fn throw_illegal_arg_res<T: ExceptionReturn>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<T, WasException> {
    let _ = throw_illegal_arg::<T>(jvm, int_state);
    Err(WasException)
}

pub fn throw_illegal_arg<T: ExceptionReturn>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> T {
    /*let bounds_object = match IllegalArgumentException::new(jvm, int_state) {
        Ok(npe) => npe,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating illegal arg exception");
            return T::invalid_default();
        }
    }
        .object()
        .into();
    int_state.set_throw(Some(bounds_object));
    T::invalid_default()*/
    todo!()
}

pub fn java_value_to_boxed_object(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, java_value: JavaValue<'gc_life>) -> Result<Option<AllocatedObject<'gc_life, 'static>>, WasException> {
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
        JavaValue::Object(obj) => todo!(), /*obj*/
        JavaValue::Top => panic!(),
    })
}

pub fn run_static_or_virtual<'gc_life, 'l>(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    class: &Arc<RuntimeClass<'gc_life>>,
    method_name: MethodName,
    desc: &CMethodDescriptor,
    args: Vec<NewJavaValue<'gc_life, '_>>,
) -> Result<Option<NewJavaValueHandle<'gc_life>>, WasException> {
    let view = class.view();
    let res_fun = view.lookup_method(method_name, desc);
    let method_view = match res_fun {
        Some(x) => x,
        None => panic!(),
    };
    if method_view.is_static() {
        invoke_static_impl(jvm, int_state, desc, class.clone(), method_view.method_i(), &method_view, args)
    } else {
        invoke_virtual_method_i(jvm, int_state, desc, class.clone(), &method_view, args)
    }
}

pub fn unwrap_or_npe<T>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, to_unwrap: Option<T>) -> Result<T, WasException> {
    match to_unwrap {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(unwrapped) => Ok(unwrapped),
    }
}