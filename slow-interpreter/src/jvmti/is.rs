use jvmti_jni_bindings::{jvmtiEnv, jclass, jboolean, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jmethodID};
use crate::jvmti::get_state;
use crate::rust_jni::native_util::from_object;
use std::mem::transmute;
use classfile_view::view::HasAccessFlags;
use crate::java_values::JavaValue;

pub unsafe extern "C" fn is_array_class(env: *mut jvmtiEnv, klass: jclass, is_array_class_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "IsArrayClass");
    is_array_class_ptr.write(is_array_impl(klass));
    jvm.tracing.trace_jdwp_function_exit(jvm, "IsArrayClass");
    jvmtiError_JVMTI_ERROR_NONE
}

pub fn is_array_impl(cls: jclass) -> u8 {
    let object_non_null = unsafe { from_object(transmute(cls)).unwrap().clone() };
    let ptype = JavaValue::Object(object_non_null.into()).cast_class().as_type();
    let is_array = ptype.is_array();
    is_array as jboolean
}

pub unsafe extern "C" fn is_interface(env: *mut jvmtiEnv, klass: jclass, is_interface_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "IsInterface");
    let res = from_object(transmute(klass)).unwrap().unwrap_normal_object().class_pointer.view().is_interface();
    is_interface_ptr.write(res as u8);
    jvm.tracing.trace_jdwp_function_exit(jvm, "IsInterface");
    jvmtiError_JVMTI_ERROR_NONE
}



pub unsafe extern "C" fn is_method_obsolete(env: *mut jvmtiEnv, _method: jmethodID, is_obsolete_ptr: *mut jboolean ) -> jvmtiError{
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "IsMethodObsolete");
    is_obsolete_ptr.write(false as u8);//todo don't support retransform classes.
    jvm.tracing.trace_jdwp_function_exit(jvm, "IsMethodObsolete");
    jvmtiError_JVMTI_ERROR_NONE
}