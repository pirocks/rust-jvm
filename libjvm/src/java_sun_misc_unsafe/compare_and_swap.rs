use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::runtime_type::RuntimeType;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::{GcManagedObject, JavaValue, Object};
use slow_interpreter::new_java_values::{AllocatedObjectHandle, NewJavaValue, NewJavaValueHandle};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state, to_object};
use slow_interpreter::unsafe_move_test::get_obj_and_name;
use slow_interpreter::utils::{throw_npe, throw_npe_res};
use verification::verifier::codecorrectness::operand_stack_has_legal_length;


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jint, new: jint) -> jboolean {
    let jvm = get_state(env);
    let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
        Err(WasException {}) => return jboolean::MAX,
    };
    let to_jv = notnull.to_jv();
    let normal_obj = to_jv.unwrap_normal_object();
    let curval = normal_obj.get_var(jvm, rc.clone(), field_name);
    (if curval.unwrap_int() == old {
        normal_obj.set_var(rc, field_name, JavaValue::Int(new));
        1
    } else {
        0
    }) as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jlong, new: jlong) -> jboolean {
    let jvm = get_state(env);
    let (rc, notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((rc, notnull, field_name)) => (rc, notnull, field_name),
        Err(WasException {}) => return jboolean::MAX,
    };
    let jv = notnull.to_jv();
    let normal_obj = jv.unwrap_normal_object();
    let curval = normal_obj.get_var_top_level(jvm, field_name);
    (if curval.unwrap_long() == old {
        normal_obj.set_var(rc, field_name, JavaValue::Long(new));
        1
    } else {
        0
    }) as jboolean
}

