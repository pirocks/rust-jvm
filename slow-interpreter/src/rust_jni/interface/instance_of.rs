use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};

use crate::instructions::special::instance_of_impl;
use crate::interpreter::WasException;
use crate::java_values::JavaValue;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

pub unsafe extern "C" fn is_instance_of(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let java_obj = from_object(obj);
    let class_object = from_object(clazz);
    let type_view = JavaValue::Object(class_object).cast_class().as_type(jvm);
    let type_ = match type_view.try_unwrap_ref_type() {
        None => unimplemented!(),
        Some(ref_type) => ref_type,
    };
    match instance_of_impl(jvm, int_state, java_obj.unwrap(), type_.clone()) {
        Ok(_) => {}
        Err(WasException {}) => todo!()
    };//todo handle npe
    (int_state.pop_current_operand_stack().unwrap_int() != 0) as jboolean
}
