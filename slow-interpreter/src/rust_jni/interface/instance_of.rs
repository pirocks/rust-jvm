use jvmti_jni_bindings::{JNIEnv, jobject, jclass, jboolean};
use crate::rust_jni::native_util::{get_state, from_object, get_frame};
use crate::instructions::special::instance_of_impl;
use std::ops::Deref;
use crate::java_values::JavaValue;

pub unsafe extern "C" fn is_instance_of(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean {
    let jvm = get_state(env);
    let java_obj = from_object(obj);
    let class_object = from_object(clazz);
    let type_view = JavaValue::Object(class_object).cast_class().as_type();
    let type_ = match type_view.try_unwrap_ref_type(){
        None => unimplemented!(),
        Some(ref_type) => ref_type,
    };
    let frame = get_frame(env);
    instance_of_impl(jvm, frame, java_obj.unwrap(), type_.clone());
    (frame.pop().unwrap_int() != 0) as jboolean
}
