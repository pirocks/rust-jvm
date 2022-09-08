use jvmti_jni_bindings::{jclass, JNIEnv, jobject, JVM_CALLER_DEPTH};
use slow_interpreter::better_java_stack::opaque_frame::OpaqueFrame;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::interpreter_util::new_object;
use slow_interpreter::rust_jni::interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_jclass, to_object_new};
use crate::JVM_GetCallerClass;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_allocateInstance<'gc>(env: *mut JNIEnv, the_unsafe: jobject, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let jclass = from_jclass(jvm, cls);
    let rc = check_initing_or_inited_class(jvm, int_state, jclass.as_type(jvm)).unwrap();
    let obj_handle = new_object(jvm, int_state, &rc);
    to_object_new(Some(obj_handle.as_allocated_obj()))
}

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv, cb: jclass) -> jclass {
    JVM_GetCallerClass(env, JVM_CALLER_DEPTH)
}
