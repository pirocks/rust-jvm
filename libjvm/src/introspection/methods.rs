use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::view::ClassView;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jboolean, jbyteArray, jclass, jint, JNIEnv, jobject, jobjectArray, JVM_ExceptionTableEntryType};
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::reflect::method::Method;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;
use slow_interpreter::utils::{throw_array_out_of_bounds, throw_illegal_arg, throw_npe};

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let method = JavaValue::Object(Some(match from_object(method) {
        None => {
            throw_npe(jvm, int_state);
            return null_mut();
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
            throw_illegal_arg(jvm, int_state);
            return null_mut();
        }
        Some(res_method_view) => res_method_view
    };
    todo!("{}", res_method_view);

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



#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionsCount(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(cb).as_runtime_class(jvm);
    let view = rc.view();
    let code_attr = match view.method_view_i(method_index as usize).code_attribute() {
        Some(x) => x,
        None => {
            throw_illegal_arg(jvm, int_state);
            return i32::MAX;
        }
    };
    code_attr.exception_table.len() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCode(env: *mut JNIEnv, cb: jclass, method_index: jint, code: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCodeLength(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxLocalsCount(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxArgsSize(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxMaxStack(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
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
    exceptions: *mut ::std::os::raw::c_ushort,
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
    unsafe extern "system" fn JVM_GetFieldIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
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
        throw_array_out_of_bounds(jvm, int_state, index);
        return u8::from(false);
    }
    u8::from(view.method_view_i(index as usize).name() == "<init>")
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: c_int) -> jboolean {
    u8::from(false)//todo perhaps check invoke dynamic stuff
}


