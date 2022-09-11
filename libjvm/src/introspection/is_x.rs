use std::intrinsics::transmute;
use std::ops::Deref;
use std::os::raw::c_int;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, jdouble, JNIEnv, JVM_Available, jvmtiError_JVMTI_ERROR_CLASS_LOADER_UNSUPPORTED, jvmtiError_JVMTI_ERROR_INVALID_CLASS};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::classfile::ACC_INTERFACE;
use rust_jvm_common::classnames::class_name;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::jvmti_interface::is::is_array_impl;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::rust_jni::jni_interface::jni::get_state;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new};
use slow_interpreter::utils::throw_array_out_of_bounds;

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
        RuntimeClass::Byte => false,
        RuntimeClass::Boolean => false,
        RuntimeClass::Short => false,
        RuntimeClass::Char => false,
        RuntimeClass::Int => false,
        RuntimeClass::Long => false,
        RuntimeClass::Float => false,
        RuntimeClass::Double => false,
        RuntimeClass::Void => false,
        RuntimeClass::Array(_) => false,
        RuntimeClass::Object(_) => runtime_class.view().is_interface(),
        _ => panic!(),
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