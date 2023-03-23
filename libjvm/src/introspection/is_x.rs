use std::ops::Deref;

use jvmti_interface::is::is_array_impl;
use jvmti_jni_bindings::{jboolean, jclass, jdouble, JNIEnv, jvmtiError_JVMTI_ERROR_INVALID_CLASS};
use runtime_class_stuff::RuntimeClass;
use slow_interpreter::new_java_values::{NewJavaValueHandle};


use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::rust_jni::jni_utils::{get_state};

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    u8::from(d.is_nan())
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let jvm = get_state(env);
    let obj = from_object_new(jvm, cls);
    let runtime_class = NewJavaValueHandle::from_optional_object(obj).cast_class().expect("todo").as_runtime_class(jvm);
    (match runtime_class.deref() {
        RuntimeClass::Primitive(_) => false,
        RuntimeClass::Array(_) => false,
        RuntimeClass::Object(_) => runtime_class.view().is_interface(),
    }) as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let jvm = get_state(env);
    match is_array_impl(jvm, cls) {
        Ok(res) => res,
        Err(error) => {
            if error == jvmtiError_JVMTI_ERROR_INVALID_CLASS {
                panic!("this should never happen since this is only called for valid classes")
            }
            panic!("Unexpected error from is_array_impl")
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let jvm = get_state(env);
    let type_ = NewJavaValueHandle::from_optional_object(from_object_new(jvm, cls)).cast_class().expect("todo").as_type(jvm);
    type_.is_primitive() as jboolean
}
