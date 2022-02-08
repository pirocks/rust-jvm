use std::borrow::Borrow;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;

use itertools::Itertools;

use classfile_view::view::{ClassView, HasAccessFlags};
use jvmti_jni_bindings::{jclass, JNIEnv, jobject, jobjectArray};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
use rust_jvm_common::descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use rust_jvm_common::descriptor_parser::Descriptor::Method;
use slow_interpreter::class_loading::{check_initing_or_inited_class, check_loaded_class};
use slow_interpreter::instructions::invoke::virtual_::invoke_virtual;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::java::lang::boolean::Boolean;
use slow_interpreter::java::lang::byte::Byte;
use slow_interpreter::java::lang::char::Char;
use slow_interpreter::java::lang::double::Double;
use slow_interpreter::java::lang::float::Float;
use slow_interpreter::java::lang::integer::Integer;
use slow_interpreter::java::lang::long::Long;
use slow_interpreter::java::lang::short::Short;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::jvmti::event_callbacks::JVMTIEvent::ClassPrepare;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{run_static_or_virtual, string_obj_to_string, throw_npe};

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewObject(env: *mut JNIEnv, obj: jobject, currClass: jclass, initClass: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetClassSigners(env: *mut JNIEnv, cls: jclass, signers: jobjectArray) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InvokeMethod(env: *mut JNIEnv, method: jobject, obj: jobject, args0: jobjectArray) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    assert_eq!(obj, std::ptr::null_mut()); //non-static methods not supported atm.
    let method_obj = match from_object(jvm, method) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let args_not_null = match from_object(jvm, args0) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let args = args_not_null.unwrap_array();
    let method_name_str = string_obj_to_string(
        jvm,
        match method_obj.lookup_field(jvm, FieldName::field_name()).unwrap_object() {
            None => return throw_npe(jvm, int_state),
            Some(method_name) => method_name,
        },
    );
    let method_name = MethodName(jvm.string_pool.add_name(method_name_str, false));
    let signature = string_obj_to_string(
        jvm,
        match method_obj.lookup_field(jvm, FieldName::field_signature()).unwrap_object() {
            None => return throw_npe(jvm, int_state),
            Some(method_name) => method_name,
        },
    );
    let clazz_java_val = method_obj.lookup_field(jvm, FieldName::field_clazz());
    let target_class_refcell_borrow = clazz_java_val.to_new().cast_class().expect("todo").as_type(jvm);
    let target_class = target_class_refcell_borrow;
    if target_class.is_primitive() || target_class.is_array() {
        unimplemented!()
    }
    let target_class_name = target_class.unwrap_class_type();
    let target_runtime_class = match check_initing_or_inited_class(jvm, int_state, target_class_name.into()) {
        Ok(x) => x,
        Err(WasException {}) => return null_mut(),
    };

    //todo this arg array setup is almost certainly wrong.
    let MethodDescriptor { parameter_types, return_type } = parse_method_descriptor(&signature).unwrap();
    let parsed_md = CMethodDescriptor {
        arg_types: parameter_types.into_iter().map(|ptype| CPDType::from_ptype(&ptype, &jvm.string_pool)).collect_vec(),
        return_type: CPDType::from_ptype(&return_type, &jvm.string_pool),
    };
    for (arg, type_) in args.array_iterator(jvm).zip(parsed_md.arg_types.iter()) {
        let arg = match type_ {
            CompressedParsedDescriptorType::BooleanType => JavaValue::Boolean(arg.cast_boolean().inner_value(jvm)),
            CompressedParsedDescriptorType::ByteType => JavaValue::Byte(arg.cast_byte().inner_value(jvm)),
            CompressedParsedDescriptorType::ShortType => JavaValue::Short(arg.cast_short().inner_value(jvm)),
            CompressedParsedDescriptorType::CharType => JavaValue::Char(arg.cast_char().inner_value(jvm)),
            CompressedParsedDescriptorType::IntType => JavaValue::Int(arg.cast_int().inner_value(jvm)),
            CompressedParsedDescriptorType::LongType => JavaValue::Long(arg.cast_long().inner_value(jvm)),
            CompressedParsedDescriptorType::FloatType => JavaValue::Float(arg.cast_float().inner_value(jvm)),
            CompressedParsedDescriptorType::DoubleType => JavaValue::Double(arg.cast_double().inner_value(jvm)),
            _ => arg.clone(),
        };
        int_state.push_current_operand_stack(arg.clone());
    }

    //todo clean this up, and handle invoke special
    let is_virtual = !target_runtime_class.view().lookup_method(method_name, &parsed_md).unwrap().is_static();
    if is_virtual {
        invoke_virtual(jvm, int_state, method_name, &parsed_md);
    } else {
        run_static_or_virtual(jvm, int_state, &target_runtime_class, method_name, &parsed_md, todo!());
    }

    new_local_ref_public(int_state.pop_current_operand_stack(Some(CClassName::object().into())).unwrap_object(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let constructor_obj = match from_object(jvm, c) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let signature_str_obj = constructor_obj.lookup_field(jvm, FieldName::field_signature());
    let temp_4 = constructor_obj.lookup_field(jvm, FieldName::field_clazz());
    let clazz = match class_object_to_runtime_class(&temp_4.to_new().cast_class().expect("todo"), jvm, int_state) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    if let Err(WasException {}) = check_loaded_class(jvm, int_state, clazz.cpdtype()) {
        return null_mut();
    };
    let mut signature_str = string_obj_to_string(
        jvm,
        match signature_str_obj.unwrap_object() {
            None => return throw_npe(jvm, int_state),
            Some(signature) => signature,
        },
    );
    dbg!(&signature_str);
    dbg!(clazz.cpdtype().unwrap_class_type().0.to_str(&jvm.string_pool));
    let MethodDescriptor { parameter_types, return_type } = parse_method_descriptor(signature_str.as_str()).unwrap();
    let args = if args0.is_null() {
        vec![]
    } else {
        let temp_1 = match from_object(jvm, args0) {
            Some(x) => x,
            None => {
                return throw_npe(jvm, int_state);
            }
        };
        let elems_refcell = temp_1.unwrap_array();
        elems_refcell
            .array_iterator(jvm)
            .zip(parameter_types.iter())
            .map(|(arg, ptype)| {
                //todo dupe with standard method invoke
                match CPDType::from_ptype(ptype, &jvm.string_pool) {
                    CompressedParsedDescriptorType::BooleanType => JavaValue::Boolean(arg.cast_boolean().inner_value(jvm)),
                    CompressedParsedDescriptorType::ByteType => JavaValue::Byte(arg.cast_byte().inner_value(jvm)),
                    CompressedParsedDescriptorType::ShortType => JavaValue::Short(arg.cast_short().inner_value(jvm)),
                    CompressedParsedDescriptorType::CharType => JavaValue::Char(arg.cast_char().inner_value(jvm)),
                    CompressedParsedDescriptorType::IntType => JavaValue::Int(arg.cast_int().inner_value(jvm)),
                    CompressedParsedDescriptorType::LongType => JavaValue::Long(arg.cast_long().inner_value(jvm)),
                    CompressedParsedDescriptorType::FloatType => JavaValue::Float(arg.cast_float().inner_value(jvm)),
                    CompressedParsedDescriptorType::DoubleType => JavaValue::Double(arg.cast_double().inner_value(jvm)),
                    _ => arg.clone(),
                }
            })
            .collect::<Vec<_>>()
    };
    let signature = CMethodDescriptor {
        arg_types: parameter_types.into_iter().map(|ptype| CPDType::from_ptype(&ptype, &jvm.string_pool)).collect_vec(),
        return_type: CPDType::from_ptype(&return_type, &jvm.string_pool), //todo use from_leaacy instead
    };
    let obj = new_object(jvm, int_state, &clazz);
    let mut full_args = vec![obj.to_jv().clone()];
    full_args.extend(args.iter().cloned());
    // dbg!(&full_args);
    run_constructor(jvm, int_state, clazz, full_args, &signature);
    new_local_ref_public(todo!()/*obj.unwrap_object()*/, int_state)
}