use std::hint::unreachable_unchecked;
use std::os::raw::{c_char, c_int, c_uchar, c_ushort};
use std::ptr::null_mut;

use itertools::Itertools;
use wtf8::Wtf8Buf;

use classfile_parser::parse_validation::ClassfileError::Java9FeatureNotSupported;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jboolean, jbyteArray, jclass, jint, JNIEnv, jobject, jobjectArray, JVM_ExceptionTableEntryType, lchmod};
use rust_jvm_common::classfile::Code;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::compressed_classfile::string_pool::CCString;
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::{ExceptionReturn, JavaValue, Object};
use slow_interpreter::new_java_values::NewJavaValue;
use slow_interpreter::rust_jni::jni_utils::{get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, to_object};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;
use slow_interpreter::stdlib::java::lang::class::JClass;
use slow_interpreter::stdlib::java::lang::reflect::method::Method;
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::{throw_array_out_of_bounds, throw_illegal_arg, throw_illegal_arg_res, throw_npe};

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters<'gc>(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let method = JavaValue::Object(
        todo!(), /*Some(match from_jclass(jvm,method) {
                     None => {
                         return throw_npe(jvm, int_state);
                     }
                     Some(method_obj) => method_obj
                 })*/
    )
        .cast_method();
    let clazz = method.get_clazz(jvm).as_runtime_class(jvm);
    let name = MethodName(jvm.string_pool.add_name(method.get_name(jvm).to_rust_string(jvm), true));
    let return_type_jclass: JClass<'gc> = method.get_return_type(jvm);
    let return_type = return_type_jclass.as_type(jvm);
    let parameter_types = method.parameter_types(jvm).into_iter().map(|jclass_| jclass_.as_type(jvm)).collect::<Vec<_>>();
    let view = clazz.view();
    let res_method_view = match view.lookup_method(name, &CMethodDescriptor { arg_types: parameter_types, return_type }) {
        None => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
        Some(res_method_view) => res_method_view,
    };
    // todo!("{}", res_method_view);
    todo!()
}

// returns 3 object array, class, name, descriptor string
#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    if from_jclass(jvm, ofClass).as_type(jvm).is_primitive() {
        return std::ptr::null_mut();
    }
    let view = from_jclass(jvm, ofClass).as_runtime_class(jvm).view();
    let em = view.enclosing_method_view();
    match em {
        None => std::ptr::null_mut(),
        Some(em) => {
            match (|| {
                let ptype_name = em.class_name(&jvm.string_pool);
                let jclass = JClass::from_type(jvm, int_state, ptype_name.to_cpdtype())?;
                let method_desc = match em.method_desc(&jvm.string_pool) {
                    None => {
                        return Ok(null_mut());
                    }
                    Some(method_desc) => method_desc
                };
                let method_desc = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(method_desc.to_str(&jvm.string_pool)))?;
                let method_name = match em.method_name(&jvm.string_pool) {
                    None => { return Ok(null_mut()); }
                    Some(method_name) => method_name,
                };
                let method_name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(method_name.0.to_str(&jvm.string_pool)))?;
                let array_obj = JavaValue::new_vec_from_vec(jvm, vec![jclass.new_java_value(), method_desc.new_java_value(), method_name.new_java_value()], CPDType::object());
                Ok(new_local_ref_public_new(Some(array_obj.as_allocated_obj()), int_state))
            })() {
                Err(WasException { exception_obj }) => {
                    *get_throw(env) = Some(WasException { exception_obj });
                    jobjectArray::invalid_default()
                }
                Ok(res) => res
            }
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    view.num_methods() as jint
}

unsafe fn get_method_view<'gc, T: ExceptionReturn>(env: *mut JNIEnv, cb: jclass, method_index: jint, and_then: impl Fn(&MethodView) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    let method_view = view.method_view_i(method_index as u16);
    and_then(&method_view)
}

unsafe fn get_code_attr<'gc, T: ExceptionReturn>(env: *mut JNIEnv, cb: jclass, method_index: jint, and_then: impl Fn(&Code) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
    get_method_view(env, cb, method_index, |method_view| {
        let jvm = get_state(env);
        let int_state = get_interpreter_state(env);
        let code_attr = match method_view.real_code_attribute() {
            Some(x) => x,
            None => {
                return throw_illegal_arg_res(jvm, int_state);
            }
        };
        and_then(code_attr)
    })
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionsCount(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    match get_code_attr(env, cb, method_index, |code| {
        Ok(code.exception_table.len() as i32) //todo this wrong, this should be in exception table length
    }) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            jint::invalid_default()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCode(env: *mut JNIEnv, cb: jclass, method_index: jint, code_output: *mut c_uchar) {
    if let Err(WasException { exception_obj }) = get_code_attr(env, cb, method_index, |code| {
        for (i, x) in code.code_raw.iter().enumerate() {
            code_output.offset(i as isize).write(*x)
        }
        Ok(())
    }) {
        *get_throw(env) = Some(WasException { exception_obj });
        return;
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCodeLength(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    match get_code_attr(env, cb, method_index, |code| Ok(code.code_raw.len() as jint)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jint::invalid_default();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_code_attr(env, cb, index, |code| Ok(code.exception_table.len() as jint)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jint::invalid_default();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxModifiers(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_method_view(env, cb, index, |method_view| Ok(method_view.access_flags() as jint)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jint::invalid_default();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxLocalsCount(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_code_attr(env, cb, index, |code| Ok(code.max_locals as jint)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jint::invalid_default();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxArgsSize(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_method_view(env, cb, index, |method_view| Ok(method_view.num_args() as jint)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jint::invalid_default();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxMaxStack(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_code_attr(env, cb, index, |code| Ok(code.max_stack as jint)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jint::invalid_default();
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableEntry(env: *mut JNIEnv, cb: jclass, method_index: jint, entry_index: jint, entry: *mut JVM_ExceptionTableEntryType) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionIndexes(env: *mut JNIEnv, cb: jclass, method_index: jint, exceptions: *mut c_ushort) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cls).as_runtime_class(jvm);
    let bytes_vec = match rc.unwrap_class_class().class_view.annotations() {
        Some(x) => x,
        None => {
            return null_mut();
        }
    };
    let java_bytes_vec = bytes_vec
        .into_iter()
        .map(|byte| NewJavaValue::Byte(byte as i8))
        .collect_vec();
    let res = JavaValue::new_vec_from_vec(jvm, java_bytes_vec, CPDType::ByteType);
    new_local_ref_public_new(Some(res.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassTypeAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    int_state.debug_print_stack_trace(jvm);
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldIxModifiers(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldTypeAnnotations(env: *mut JNIEnv, field: jobject) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodTypeAnnotations(env: *mut JNIEnv, method: jobject) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsConstructorIx(env: *mut JNIEnv, cb: jclass, index: c_int) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.num_methods() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    u8::from(view.method_view_i(index as u16).name() == MethodName::constructor_init())
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: c_int) -> jboolean {
    u8::from(false)
    //todo perhaps check invoke dynamic stuff
}