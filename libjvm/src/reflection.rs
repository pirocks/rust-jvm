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
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::jvmti::event_callbacks::JVMTIEvent::ClassPrepare;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state, to_object};
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
unsafe extern "system" fn JVM_InvokeMethod<'gc>(env: *mut JNIEnv, method: jobject, obj: jobject, args0: jobjectArray) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm: &'gc JVMState<'gc> = get_state(env);
    // assert_eq!(obj, std::ptr::null_mut()); //non-static methods not supported atm.
    let method_obj = match from_object_new(jvm, method) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let args_not_null = match from_object_new(jvm, args0) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let args = args_not_null.unwrap_array(jvm);
    let method_name_str = match method_obj.as_allocated_obj().get_var_top_level(jvm, FieldName::field_name()).unwrap_object() {
            None => return throw_npe(jvm, int_state),
            Some(method_name) => method_name.cast_string().to_rust_string(jvm),
        };
    let method_name = MethodName(jvm.string_pool.add_name(method_name_str, false));
    let signature = match method_obj.as_allocated_obj().get_var_top_level(jvm, FieldName::field_signature()).unwrap_object() {
            None => return throw_npe(jvm, int_state),
            Some(method_sig) => method_sig.cast_string().to_rust_string(jvm),
        };
    let clazz_java_val = method_obj.as_allocated_obj().get_var_top_level(jvm, FieldName::field_clazz());
    let target_class_refcell_borrow = clazz_java_val.cast_class().expect("todo").as_type(jvm);
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
    let invoke_virtual_obj = NewJavaValueHandle::from_optional_object(from_object_new(jvm,obj));
    let mut res_args = if obj  == null_mut(){
        vec![]
    }else {
        vec![invoke_virtual_obj.as_njv()]
    };
    let collected_args_array = args.array_iterator().collect_vec();
    for (arg, type_) in collected_args_array.iter().zip(parsed_md.arg_types.iter()) {
        let arg : &NewJavaValueHandle<'gc> = arg;
        let arg = match type_ {
            CompressedParsedDescriptorType::BooleanType => NewJavaValue::Boolean(arg.cast_boolean().inner_value(jvm)),
            CompressedParsedDescriptorType::ByteType => NewJavaValue::Byte(arg.cast_byte().inner_value(jvm)),
            CompressedParsedDescriptorType::ShortType => NewJavaValue::Short(arg.cast_short().inner_value(jvm)),
            CompressedParsedDescriptorType::CharType => NewJavaValue::Char(arg.cast_char().inner_value(jvm)),
            CompressedParsedDescriptorType::IntType => NewJavaValue::Int(arg.cast_int().inner_value(jvm)),
            CompressedParsedDescriptorType::LongType => NewJavaValue::Long(arg.cast_long().inner_value(jvm)),
            CompressedParsedDescriptorType::FloatType => NewJavaValue::Float(arg.cast_float().inner_value(jvm)),
            CompressedParsedDescriptorType::DoubleType => NewJavaValue::Double(arg.cast_double().inner_value(jvm)),
            _ => arg.as_njv(),
        };
        res_args.push(arg.clone());
    }

    //todo clean this up, and handle invoke special
    let is_virtual = !target_runtime_class.view().lookup_method(method_name, &parsed_md).unwrap().is_static();
    let res = if is_virtual {
        invoke_virtual(jvm, int_state, method_name, &parsed_md, res_args).unwrap().unwrap()
    } else {
        run_static_or_virtual(jvm, int_state, &target_runtime_class, method_name, &parsed_md, res_args).unwrap().unwrap()
    };

    new_local_ref_public_new(res.as_njv().unwrap_object_alloc(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let constructor_obj = match from_object_new(jvm, c) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let signature_str_obj = constructor_obj.as_allocated_obj().get_var_top_level(jvm, FieldName::field_signature());
    let temp_4 = constructor_obj.as_allocated_obj().get_var_top_level(jvm, FieldName::field_clazz());
    let clazz = match class_object_to_runtime_class(&temp_4.cast_class().expect("todo"), jvm, int_state) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    if let Err(WasException {}) = check_loaded_class(jvm, int_state, clazz.cpdtype()) {
        return null_mut();
    };
    let signature_object = match signature_str_obj.unwrap_object() {
        None => return throw_npe(jvm, int_state),
        Some(signature) => signature,
    };
    let mut signature_str = string_obj_to_string(
        jvm,
        signature_object.as_allocated_obj(),
    );
    let MethodDescriptor { parameter_types, return_type } = parse_method_descriptor(signature_str.as_str()).unwrap();
    let args = if args0.is_null() {
        vec![]
    } else {
        let temp_1 = match from_object_new(jvm, args0) {
            Some(x) => x,
            None => {
                return throw_npe(jvm, int_state);
            }
        };
        let elems_refcell = temp_1.unwrap_array(jvm);
        elems_refcell
            .array_iterator()
            .zip(parameter_types.iter())
            .map(|(arg, ptype)| {
                //todo dupe with standard method invoke
                match CPDType::from_ptype(ptype, &jvm.string_pool) {
                    CompressedParsedDescriptorType::BooleanType => NewJavaValueHandle::Boolean(arg.cast_boolean().inner_value(jvm)),
                    CompressedParsedDescriptorType::ByteType => NewJavaValueHandle::Byte(arg.cast_byte().inner_value(jvm)),
                    CompressedParsedDescriptorType::ShortType => NewJavaValueHandle::Short(arg.cast_short().inner_value(jvm)),
                    CompressedParsedDescriptorType::CharType => NewJavaValueHandle::Char(arg.cast_char().inner_value(jvm)),
                    CompressedParsedDescriptorType::IntType => NewJavaValueHandle::Int(arg.cast_int().inner_value(jvm)),
                    CompressedParsedDescriptorType::LongType => NewJavaValueHandle::Long(arg.cast_long().inner_value(jvm)),
                    CompressedParsedDescriptorType::FloatType => NewJavaValueHandle::Float(arg.cast_float().inner_value(jvm)),
                    CompressedParsedDescriptorType::DoubleType => NewJavaValueHandle::Double(arg.cast_double().inner_value(jvm)),
                    _ => arg,
                }
            })
            .collect::<Vec<_>>()
    };
    let signature = CMethodDescriptor {
        arg_types: parameter_types.into_iter().map(|ptype| CPDType::from_ptype(&ptype, &jvm.string_pool)).collect_vec(),
        return_type: CPDType::from_ptype(&return_type, &jvm.string_pool), //todo use from_leaacy instead
    };
    let obj = new_object(jvm, int_state, &clazz);
    let mut full_args = vec![obj.new_java_value()];
    full_args.extend(args.iter().map(|handle|handle.as_njv()));
    // dbg!(&full_args);
    run_constructor(jvm, int_state, clazz, full_args, &signature);
    new_local_ref_public_new(Some(obj.as_allocated_obj()), int_state)
}