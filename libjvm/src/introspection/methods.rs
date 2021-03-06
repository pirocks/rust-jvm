use std::hint::unreachable_unchecked;
use std::os::raw::{c_char, c_int, c_uchar, c_ushort};
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jboolean, jbyteArray, jclass, jint, JNIEnv, jobject, jobjectArray, JVM_ExceptionTableEntryType};
use rust_jvm_common::classfile::Code;
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::reflect::method::Method;
use slow_interpreter::java_values::{ExceptionReturn, JavaValue, Object};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;
use slow_interpreter::utils::{throw_array_out_of_bounds, throw_illegal_arg, throw_illegal_arg_res, throw_npe};

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let method = JavaValue::Object(Some(match from_object(method) {
        None => {
            return throw_npe(jvm, int_state);
        }
        Some(method_obj) => method_obj
    })).cast_method();
    let clazz = method.get_clazz().as_runtime_class(jvm);
    let name = method.get_name().to_rust_string();
    let return_type_jclass: JClass = method.get_returnType();
    let return_type = return_type_jclass.as_type(jvm).to_ptype();
    let parameter_types = method.parameter_types().into_iter().map(|jclass_| jclass_.as_type(jvm).to_ptype()).collect::<Vec<_>>();
    let view = clazz.view();
    let res_method_view = match view.lookup_method(name.as_str(), &MethodDescriptor { parameter_types, return_type }) {
        None => {
            return throw_illegal_arg(jvm, int_state);
        }
        Some(res_method_view) => res_method_view
    };
    // todo!("{}", res_method_view);
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let jvm = get_state(env);
    if from_jclass(ofClass).as_type(jvm).is_primitive() {
        return std::ptr::null_mut();
    }
    let em = from_jclass(ofClass).as_runtime_class(jvm).view().enclosing_method_view();
    match em {
        None => std::ptr::null_mut(),
        Some(_) => unimplemented!(),
    }
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(cb).as_runtime_class(jvm);
    let view = rc.view();
    view.num_methods() as jint
}

unsafe fn get_method_view<T: ExceptionReturn>(env: *mut JNIEnv, cb: jclass, method_index: jint, and_then: impl Fn(&MethodView) -> Result<T, WasException>) -> Result<T, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(cb).as_runtime_class(jvm);
    let view = rc.view();
    let method_view = view.method_view_i(method_index as usize);
    and_then(&method_view)
}

//todo should just return T, no need to handle result
unsafe fn get_code_attr<T: ExceptionReturn>(env: *mut JNIEnv, cb: jclass, method_index: jint, and_then: impl Fn(&Code) -> Result<T, WasException>) -> Result<T, WasException> {
    get_method_view(env, cb, method_index, |method_view| {
        let jvm = get_state(env);
        let int_state = get_interpreter_state(env);
        let code_attr = match method_view.code_attribute() {
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
        Err(WasException {}) => jint::invalid_default()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCode(env: *mut JNIEnv, cb: jclass, method_index: jint, code_output: *mut c_uchar) {
    match get_code_attr(env, cb, method_index, |code| {
        for (i, x) in code.code_raw.iter().enumerate() {
            code_output.offset(i as isize).write(*x)
        }
        Ok(())
    }) {
        Ok(res) => res,
        Err(WasException {}) => return
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCodeLength(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    match get_code_attr(env, cb, method_index, |code| {
        Ok(code.code_raw.len() as jint)
    }) {
        Ok(res) => res,
        Err(WasException {}) => return jint::invalid_default()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_code_attr(env, cb, index, |code| {
        Ok(code.exception_table.len() as jint)
    }) {
        Ok(res) => res,
        Err(WasException {}) => return jint::invalid_default()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxModifiers(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_method_view(env, cb, index, |method_view| {
        Ok(method_view.access_flags() as jint)
    }) {
        Ok(res) => res,
        Err(WasException {}) => return jint::invalid_default()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxLocalsCount(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_code_attr(env, cb, index, |code| {
        Ok(code.max_locals as jint)
    }) {
        Ok(res) => res,
        Err(WasException {}) => return jint::invalid_default()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxArgsSize(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_method_view(env, cb, index, |method_view| {
        Ok(method_view.num_args() as jint)
    }) {
        Ok(res) => res,
        Err(WasException {}) => return jint::invalid_default()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxMaxStack(env: *mut JNIEnv, cb: jclass, index: c_int) -> jint {
    match get_code_attr(env, cb, index, |code| {
        Ok(code.max_stack as jint)
    }) {
        Ok(res) => res,
        Err(WasException {}) => return jint::invalid_default()
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
unsafe extern "system" fn JVM_GetMethodIxExceptionTableEntry(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    entry_index: jint,
    entry: *mut JVM_ExceptionTableEntryType,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionIndexes(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    exceptions: *mut c_ushort,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassTypeAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
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
    let rc = from_jclass(cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.num_methods() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    u8::from(view.method_view_i(index as usize).name() == "<init>")
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: c_int) -> jboolean {
    u8::from(false)//todo perhaps check invoke dynamic stuff
}


