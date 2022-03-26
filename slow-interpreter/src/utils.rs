use std::sync::Arc;


use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::jint;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};

use crate::{JVMState, NewAsObjectOrJavaValue, NewJavaValue};
use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::boolean::Boolean;
use crate::java::lang::byte::Byte;
use crate::java::lang::char::Char;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::illegal_argument_exception::IllegalArgumentException;
use crate::java::lang::int::Int;
use crate::java::lang::long::Long;
use crate::java::lang::short::Short;
use crate::java_values::{ExceptionReturn, JavaValue};
use crate::new_java_values::{AllocatedObject, NewJavaValueHandle};

pub fn lookup_method_parsed<'gc>(jvm: &'gc JVMState<'gc>, class: Arc<RuntimeClass<'gc>>, name: MethodName, descriptor: &CMethodDescriptor) -> Option<(u16, Arc<RuntimeClass<'gc>>)> {
    lookup_method_parsed_impl(jvm, class, name, descriptor)
}

pub fn lookup_method_parsed_impl<'gc>(jvm: &'gc JVMState<'gc>, class: Arc<RuntimeClass<'gc>>, name: MethodName, descriptor: &CMethodDescriptor) -> Option<(u16, Arc<RuntimeClass<'gc>>)> {
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

pub fn string_obj_to_string<'gc>(jvm: &'gc JVMState<'gc>, str_obj: AllocatedObject<'gc, '_>) -> String {
    let str_class_pointer = assert_inited_or_initing_class(jvm, CClassName::string().into());
    let temp = str_obj.lookup_field(&str_class_pointer, FieldName::field_value());
    let chars = temp.unwrap_array(jvm);
    let borrowed_elems = chars.array_iterator();
    char::decode_utf16(borrowed_elems.map(|jv| jv.as_njv().unwrap_char_strict())).collect::<Result<String, _>>().expect("really weird string encountered")
    //todo so techincally java strings need not be valid so we can't return a rust string and have to do everything on bytes
}

pub fn throw_npe_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<T, WasException> {
    let _ = throw_npe::<T>(jvm, int_state);
    Err(WasException)
}

pub fn throw_npe<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> T {
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

pub fn throw_array_out_of_bounds_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, index: jint) -> Result<T, WasException> {
    let _ = throw_array_out_of_bounds::<T>(jvm, int_state, index);
    Err(WasException)
}

pub fn throw_array_out_of_bounds<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, index: jint) -> T {
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

pub fn throw_illegal_arg_res<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<T, WasException> {
    let _ = throw_illegal_arg::<T>(jvm, int_state);
    Err(WasException)
}

pub fn throw_illegal_arg<'gc, 'l, T: ExceptionReturn>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> T {
    let illegal_arg_object = match IllegalArgumentException::new(jvm, int_state) {
        Ok(illegal_arg) => illegal_arg,
        Err(WasException {}) => {
            eprintln!("Warning error encountered creating illegal arg exception");
            return T::invalid_default();
        }
    }.object();
    int_state.set_throw(Some(illegal_arg_object));
    T::invalid_default()
}

pub fn java_value_to_boxed_object<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, java_value: JavaValue<'gc>) -> Result<Option<AllocatedObject<'gc, 'static>>, WasException> {
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

pub fn run_static_or_virtual<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &'_ mut InterpreterStateGuard<'gc, 'l>,
    class: &Arc<RuntimeClass<'gc>>,
    method_name: MethodName,
    desc: &CMethodDescriptor,
    args: Vec<NewJavaValue<'gc, '_>>,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
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

pub fn unwrap_or_npe<'gc, 'l, T>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, to_unwrap: Option<T>) -> Result<T, WasException> {
    match to_unwrap {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(unwrapped) => Ok(unwrapped),
    }
}