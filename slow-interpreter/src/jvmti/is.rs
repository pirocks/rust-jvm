use std::mem::transmute;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jboolean, jclass, jmethodID, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::java_values::JavaValue;
use crate::jvmti::get_state;
use crate::method_table::MethodId;
use crate::rust_jni::native_util::from_object;

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


pub unsafe extern "C" fn is_method_obsolete(env: *mut jvmtiEnv, _method: jmethodID, is_obsolete_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "IsMethodObsolete");
    is_obsolete_ptr.write(false as u8);//todo don't support retransform classes.
    jvm.tracing.trace_jdwp_function_exit(jvm, "IsMethodObsolete");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn is_method_native(
    env: *mut jvmtiEnv,
    method: jmethodID,
    is_native_ptr: *mut jboolean,
) -> jvmtiError {
    let jvm = get_state(env);
    let method_id: MethodId = transmute(method);
    let (rc, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let mv = rc.view().method_view_i(method_i as usize);
    dbg!(mv.name());
    dbg!(mv.is_native());
    is_native_ptr.write(mv.is_native() as jboolean);
    jvmtiError_JVMTI_ERROR_NONE
}