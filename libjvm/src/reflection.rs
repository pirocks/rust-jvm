use std::borrow::Borrow;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;

use itertools::Itertools;

use classfile_view::view::{ClassView, HasAccessFlags};
use jni_interface::util::class_object_to_runtime_class;
use jvmti_jni_bindings::{jclass, JNIEnv, jobject, jobjectArray};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use rust_jvm_common::descriptor_parser::Descriptor::Method;
use rust_jvm_common::ptype::PType;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::better_java_stack::opaque_frame::OpaqueFrame;
use slow_interpreter::class_loading::{check_initing_or_inited_class, check_loaded_class};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter::common::invoke::virtual_::invoke_virtual;
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::java_values::{ExceptionReturn, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::allocated_objects::{AllocatedHandle, AllocatedNormalObjectHandle};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object};
use slow_interpreter::stdlib::java::lang::boolean::Boolean;
use slow_interpreter::stdlib::java::lang::byte::Byte;
use slow_interpreter::stdlib::java::lang::char::Char;
use slow_interpreter::stdlib::java::lang::double::Double;
use slow_interpreter::stdlib::java::lang::float::Float;
use slow_interpreter::stdlib::java::lang::int::Int;
use slow_interpreter::stdlib::java::lang::long::Long;
use slow_interpreter::stdlib::java::lang::short::Short;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::{java_value_to_boxed_object, run_static_or_virtual, throw_npe};

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
    let jvm: &'gc JVMState<'gc> = get_state(env);
    let int_state = get_interpreter_state(env);
    let method_obj = match from_object_new(jvm, method) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state, get_throw(env));
        }
    };

    let method_name_str = match method_obj.unwrap_normal_object_ref().get_var_top_level(jvm, FieldName::field_name()).unwrap_object() {
        None => return throw_npe(jvm, int_state, get_throw(env)),
        Some(method_name) => method_name.cast_string().to_rust_string(jvm),
    };
    let method_name = MethodName(jvm.string_pool.add_name(method_name_str, false));
    let clazz_java_val = method_obj.unwrap_normal_object_ref().get_var_top_level(jvm, FieldName::field_clazz());
    let target_class_refcell_borrow = clazz_java_val.cast_class().expect("todo").as_type(jvm);
    let target_class = target_class_refcell_borrow;
    if target_class.is_primitive() || target_class.is_array() {
        unimplemented!()
    }
    let target_class_name = target_class.unwrap_class_type();
    let target_runtime_class = match check_initing_or_inited_class(jvm, int_state, target_class_name.into()) {
        Ok(x) => x,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    };

    let method = method_obj.cast_method();
    let parameter_types = method.parameter_types(jvm).iter().map(|paramater_type| paramater_type.as_type(jvm)).collect_vec();
    let return_types = method.get_return_type(jvm).as_type(jvm);
    let parsed_md = CMethodDescriptor {
        arg_types: parameter_types,
        return_type: return_types,
    };
    let is_virtual = !target_runtime_class.view().lookup_method(method_name, &parsed_md).unwrap().is_static();
    let invoke_virtual_obj = NewJavaValueHandle::from_optional_object(from_object_new(jvm, obj));
    let mut res_args = if !is_virtual{
        vec![]
    } else {
        vec![invoke_virtual_obj]
    };

    let res_args = match from_object_new(jvm, args0) {
        Some(args_not_null) => {
            //todo this arg array setup is almost certainly wrong.
            //todo clean this up, and handle invoke special
            let args = args_not_null.unwrap_array();
            let collected_args_array = args.array_iterator().collect_vec();
            for (arg, type_) in collected_args_array.into_iter().zip(parsed_md.arg_types.iter()) {
                let arg = match type_ {
                    CompressedParsedDescriptorType::BooleanType => NewJavaValueHandle::Boolean(arg.as_njv().to_handle_discouraged().cast_boolean().inner_value(jvm)),
                    CompressedParsedDescriptorType::ByteType => NewJavaValueHandle::Byte(arg.as_njv().to_handle_discouraged().cast_byte().inner_value(jvm)),
                    CompressedParsedDescriptorType::ShortType => NewJavaValueHandle::Short(arg.as_njv().to_handle_discouraged().cast_short().inner_value(jvm)),
                    CompressedParsedDescriptorType::CharType => NewJavaValueHandle::Char(arg.as_njv().to_handle_discouraged().cast_char().inner_value(jvm)),
                    CompressedParsedDescriptorType::IntType => NewJavaValueHandle::Int(arg.as_njv().to_handle_discouraged().cast_int().inner_value(jvm)),
                    CompressedParsedDescriptorType::LongType => NewJavaValueHandle::Long(arg.as_njv().to_handle_discouraged().cast_long().inner_value(jvm)),
                    CompressedParsedDescriptorType::FloatType => NewJavaValueHandle::Float(arg.as_njv().to_handle_discouraged().cast_float().inner_value(jvm)),
                    CompressedParsedDescriptorType::DoubleType => NewJavaValueHandle::Double(arg.as_njv().to_handle_discouraged().cast_double().inner_value(jvm)),
                    _ => arg,
                };
                res_args.push(arg);
            }

            res_args
        }
        None => {
            res_args
        }
    };
    let res_args = res_args.iter().map(|handle|handle.as_njv()).collect_vec();
    let res = if is_virtual {
        match invoke_virtual(jvm, int_state, method_name, &parsed_md, res_args) {
            Ok(x) => x,
            Err(WasException { exception_obj }) => {
                *get_throw(env) = Some(WasException { exception_obj });
                return jobject::invalid_default();
            }
        }
    } else {
        match run_static_or_virtual(jvm, int_state, &target_runtime_class, method_name, &parsed_md, res_args) {
            Ok(x) => x,
            Err(WasException { exception_obj }) => {
                *get_throw(env) = Some(WasException { exception_obj });
                return jobject::invalid_default();
            }
        }
    };

    let res = match res {
        None => {
            None
        }
        Some(njv) => {
            if let NewJavaValue::AllocObject(obj) = njv.as_njv() {
                njv.unwrap_object()
            } else {

                match java_value_to_boxed_object(jvm, int_state, njv.as_njv(), parsed_md.return_type) {
                    Ok(obj) => {
                        obj
                    }
                    Err(WasException { exception_obj }) => {
                        *get_throw(env) = Some(WasException { exception_obj });
                        return jobject::invalid_default();
                    }
                }
            }
        }
    };

    new_local_ref_public_new(res.as_ref().map(|obj| obj.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor<'gc>(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let constructor_obj = match from_object_new(jvm, c) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state, get_throw(env));
        }
    };
    let temp_4 = constructor_obj.unwrap_normal_object_ref().get_var_top_level(jvm, FieldName::field_clazz());
    let clazz = match class_object_to_runtime_class(&temp_4.cast_class().expect("todo"), jvm) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state, get_throw(env));
        }
    };
    if let Err(WasException { exception_obj }) = check_loaded_class(jvm, int_state, clazz.cpdtype()) {
        todo!();
        return null_mut();
    };
    let parameter_types = constructor_obj.cast_constructor().parameter_types(jvm).iter().map(|paramater_type| paramater_type.as_type(jvm)).collect_vec();
    let args = if args0.is_null() {
        vec![]
    } else {
        let temp_1 = match from_object_new(jvm, args0) {
            Some(x) => x,
            None => {
                return throw_npe(jvm, int_state, get_throw(env));
            }
        };
        let elems_refcell = temp_1.unwrap_array();
        elems_refcell
            .array_iterator()
            .zip(parameter_types.iter())
            .map(|(arg, cpdtype)| {
                //todo dupe with standard method invoke
                match cpdtype {
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
        arg_types: parameter_types,
        return_type: CPDType::VoidType, //todo use from_leaacy instead
    };
    let obj = new_object(jvm, int_state, &clazz, false);
    let mut full_args = vec![obj.new_java_value()];
    full_args.extend(args.iter().map(|handle| handle.as_njv()));
    run_constructor(jvm, int_state, clazz, full_args, &signature);
    new_local_ref_public_new(Some(obj.as_allocated_obj()), int_state)
}