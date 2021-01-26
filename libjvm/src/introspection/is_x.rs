use std::intrinsics::transmute;
use std::ops::Deref;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, jdouble, JNIEnv, JVM_Available, jvmtiError_JVMTI_ERROR_CLASS_LOADER_UNSUPPORTED};
use rust_jvm_common::classfile::ACC_INTERFACE;
use rust_jvm_common::classnames::class_name;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::jvmti::is::is_array_impl;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_object, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let jvm = get_state(env);
    let obj = from_object(cls);
    let runtime_class = JavaValue::Object(obj).cast_class().as_runtime_class(jvm);
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
        RuntimeClass::Object(_) => {
            runtime_class.view().is_interface()
        }
    }) as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let jvm = get_state(env);
    is_array_impl(jvm, cls).unwrap()
}


#[no_mangle]
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let jvm = get_state(env);
    let type_ = JavaValue::Object(from_object(cls)).cast_class().as_type(jvm);
    type_.is_primitive() as jboolean
}


#[no_mangle]
unsafe extern "system" fn JVM_IsConstructorIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

