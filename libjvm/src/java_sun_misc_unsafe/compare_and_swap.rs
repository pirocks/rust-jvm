use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::throw_npe;
use verification::verifier::codecorrectness::operand_stack_has_legal_length;

unsafe fn get_obj_and_name(env: *mut JNIEnv, the_unsafe: jobject, target_obj: jobject, offset: jlong) -> Result<(Arc<Object>, String), WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
    let view = rc.view();
    let field = view.field(field_i as usize);
    let field_name = field.field_name();
    let notnull = match from_object(target_obj) {
        None => {
            throw_npe(jvm, int_state);
            return Err(WasException);
        }
        Some(notnull) => notnull
    };
    Ok((notnull, field_name))
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(env: *mut JNIEnv, the_unsafe: jobject,
                                                                 target_obj: jobject,
                                                                 offset: jlong,
                                                                 old: jint,
                                                                 new: jint,
) -> jboolean {
    let (notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((notnull, field_name)) => (notnull, field_name),
        Err(WasException {}) => return jboolean::MAX
    };
    let normal_obj = notnull.unwrap_normal_object();
    let mut fields_borrow = normal_obj.fields_mut();
    let curval = fields_borrow.get(field_name.as_str()).unwrap();
    (if curval.unwrap_int() == old {
        fields_borrow.insert(field_name, JavaValue::Int(new));
        1
    } else {
        0
    }) as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(env: *mut JNIEnv, the_unsafe: jobject,
                                                                  target_obj: jobject,
                                                                  offset: jlong,
                                                                  old: jlong,
                                                                  new: jlong,
) -> jboolean {
    let (notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
        Ok((notnull, field_name)) => (notnull, field_name),
        Err(WasException {}) => return jboolean::MAX
    };
    let normal_obj = notnull.unwrap_normal_object();
    let mut fields_borrow = normal_obj.fields_mut();
    let curval = fields_borrow.get(field_name.as_str()).unwrap();
    (if curval.unwrap_long() == old {
        fields_borrow.insert(field_name, JavaValue::Long(new));
        1
    } else {
        0
    }) as jboolean
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    target_obj: jobject,
    offset: jlong,
    old: jobject,
    new: jobject,
) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let notnull = match from_object(target_obj) {
        None => {
            throw_npe(jvm, int_state);
            return jboolean::MAX;
        }
        Some(notnull) => notnull
    };
    let new = JavaValue::Object(from_object(new));
    match notnull.deref() {
        Object::Array(arr) => {
            let mut ref_mut = arr.unwrap_mut();
            let curval = ref_mut.get_mut((offset as usize)).unwrap();
            let old = from_object(old);
            do_swap(curval, old, new)
        }
        Object::Object(normal_obj) => {
            let (notnull, field_name) = match get_obj_and_name(env, the_unsafe, target_obj, offset) {
                Ok((notnull, field_name)) => (notnull, field_name),
                Err(WasException {}) => return jboolean::MAX
            };
            let mut fields_borrow = normal_obj.fields_mut();
            let curval = fields_borrow.get_mut(field_name.as_str()).unwrap();
            let old = from_object(old);
            do_swap(curval, old, new)
        }
    }
}


pub fn do_swap(curval: &mut JavaValue, old: Option<Arc<Object>>, new: JavaValue) -> jboolean {
    let should_replace = match curval.unwrap_object() {
        None => {
            match old {
                None => true,
                Some(_) => false
            }
        }
        Some(cur) => {
            match old {
                None => false,
                Some(old) => Arc::ptr_eq(&cur, &old)
            }
        }
    };
    if should_replace {
        *curval = new;
    }
    should_replace as jboolean
}