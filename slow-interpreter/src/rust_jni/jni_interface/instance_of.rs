use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};

use crate::interpreter::common::special::{instance_of_exit_impl};
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::jni_interface::{get_interpreter_state, get_state};
use crate::rust_jni::native_util::{from_object_new};
use crate::utils::throw_illegal_arg;

pub unsafe extern "C" fn is_instance_of(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let java_obj = from_object_new(jvm, obj);
    let class_object = from_object_new(jvm, clazz);
    let type_view = NewJavaValueHandle::from_optional_object(class_object).cast_class().expect("todo").as_type(jvm);
    let type_ = match type_view.try_unwrap_ref_type() {
        None => {
            return throw_illegal_arg(jvm, int_state);
        }
        Some(ref_type) => ref_type,
    };
    let res = instance_of_exit_impl(jvm,  type_.to_cpdtype(), java_obj.as_ref());
    (res != 0) as jboolean
}