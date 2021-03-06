use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::rust_jni::native_util::{from_object, get_state, to_object};
use verification::verifier::codecorrectness::operand_stack_has_legal_length;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(env: *mut JNIEnv, the_unsafe: jobject,
                                                                 target_obj: jobject,
                                                                 offset: jlong,
                                                                 old: jint,
                                                                 new: jint,
) -> jboolean {
    let jvm = get_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
    let view = rc.view();
    let field = view.field(field_i as usize);
    let field_name = field.field_name();
    let notnull = from_object(target_obj).unwrap(); //todo handle npe
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
    //TODO MAJOR DUP
    let jvm = get_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
    let view = rc.view();
    let field = view.field(field_i as usize);
    let field_name = field.field_name();
    let notnull = from_object(target_obj).unwrap();//todo handle npe
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
//TODO MAJOR DUP
    //and even more b/c array v. object
    let jvm = get_state(env);
    let notnull = from_object(target_obj).unwrap();//todo handle npe
    match notnull.deref() {
        Object::Array(arr) => {
            //todo there is somewhere else where unwrap_mut isn't done todo
            let mut ref_mut = arr.unwrap_mut();
            let curval = ref_mut.get_mut((offset as usize)).unwrap();
            let old = from_object(old);
            ((if (curval.unwrap_object().is_none() && old.is_none()) || (
                curval.unwrap_object().is_some() &&
                    old.is_some() &&
                    Arc::ptr_eq(&curval.unwrap_object_nonnull(), &old.unwrap())) {//todo handle npe
                *curval = JavaValue::Object(from_object(new));
                1
            } else {
                0
            }) as jboolean)
        }
        Object::Object(normal_obj) => {
            let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
            let view = rc.view();
            let field = view.field(field_i as usize);
            let field_name = field.field_name();
            let mut fields_borrow = normal_obj.fields_mut();
            let curval = fields_borrow.get(field_name.as_str()).unwrap();
            let old = from_object(old);
            ((if (curval.unwrap_object().is_none() && old.is_none()) || (curval.unwrap_object().is_some() && old.is_some() && Arc::ptr_eq(&curval.unwrap_object().unwrap(), &old.unwrap())) {//todo handle npe
                fields_borrow.insert(field_name, JavaValue::Object(from_object(new)));
                1
            } else {
                0
            }) as jboolean)
        }
    }
}
