use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};

use slow_interpreter::interpreter::common::special::instance_of_exit_impl;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::utils::throw_illegal_arg;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};

pub unsafe extern "C" fn is_instance_of(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let java_obj = from_object_new(jvm, obj);
    let class_object = from_object_new(jvm, clazz);
    let type_view = NewJavaValueHandle::from_optional_object(class_object).cast_class().expect("todo").as_type(jvm);
    let type_ = match type_view.try_unwrap_ref_type() {
        None => {
            return throw_illegal_arg(jvm, int_state, get_throw(env));
        }
        Some(ref_type) => ref_type,
    };
    let res = instance_of_exit_impl(jvm, type_.to_cpdtype(), java_obj.as_ref());
    (res != 0) as jboolean
}