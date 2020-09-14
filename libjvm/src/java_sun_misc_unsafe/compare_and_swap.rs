use std::mem::transmute;

use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_state};

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
    let notnull = from_object(target_obj).unwrap();
    let normal_obj = notnull.unwrap_normal_object();
    let mut fields_borrow = normal_obj.fields.borrow_mut();
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
    let notnull = from_object(target_obj).unwrap();
    let normal_obj = notnull.unwrap_normal_object();
    let mut fields_borrow = normal_obj.fields.borrow_mut();
    let curval = fields_borrow.get(field_name.as_str()).unwrap();
    (if curval.unwrap_long() == old {
        fields_borrow.insert(field_name, JavaValue::Long(new));
        1
    } else {
        0
    }) as jboolean
}
