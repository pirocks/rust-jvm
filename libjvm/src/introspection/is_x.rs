use jvmti_jni_bindings::{jdouble, jboolean, JNIEnv, jclass, JVM_Available};
use rust_jvm_common::classfile::ACC_INTERFACE;
use rust_jvm_common::classnames::class_name;
use slow_interpreter::rust_jni::native_util::{from_object, get_state, get_frame};
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use std::intrinsics::transmute;
use slow_interpreter::jvmti::is::is_array_impl;
use slow_interpreter::java_values::JavaValue;
use std::ops::Deref;
use slow_interpreter::runtime_class::RuntimeClass;

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let state = get_state(env);
    let frame = get_frame(env);
    let obj = from_object(cls);
    let runtime_class = JavaValue::Object(obj).cast_class().as_runtime_class();
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
        },
    }) as jboolean

}

#[no_mangle]
unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    is_array_impl(transmute(cls))
}



#[no_mangle]
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let type_ = JavaValue::Object(from_object(cls)).cast_class().as_type();
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

