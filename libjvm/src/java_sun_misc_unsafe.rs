use std::intrinsics::transmute;
use std::ops::Deref;
use std::ptr::null_mut;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jbyte, jclass, jint, jlong, JNIEnv, jobject, JVM_CALLER_DEPTH};
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::interface::get_field::new_field_id;
use slow_interpreter::rust_jni::native_util::{from_object, get_state};

use crate::introspection::JVM_GetCallerClass;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(
    env: *mut JNIEnv,
    cb: jclass) -> () {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    0//unimplemented but can't return nothing.
    //essentially the amount at the beginning of the array which is reserved
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    1//todo unimplemented but can't return nothing, and need to return a power of 2,1 counts as a power of two. This essentially reprs the size of an elem in java arrays
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(env: *mut JNIEnv,
                                                           obj: jobject) -> jint {
    64//officially speaking unimplemented but can't return nothing, and should maybe return something reasonable todo
}

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv,
                                                                     cb: jclass) -> jclass {
    return JVM_GetCallerClass(env, JVM_CALLER_DEPTH);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    obj: jobject,
    offset: jlong,
    obj1: jobject,
    obj2: jobject,
) -> jboolean {
//    if mangled == "Java_sun_misc_Unsafe_compareAndSwapObject".to_string() {
//        //todo do nothing for now and see what happens
//        Some(JavaValue::Boolean(true))
//    }
//    unimplemented!()
    true as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    src_obj: jobject,
    offset: jlong,
    dst_obj: jobject,
    address: jlong,
    len: jlong,
) {
    let nonnull = from_object(src_obj).unwrap();
    let as_array = nonnull.unwrap_array();//not defined for non-byte-array objects
    assert_eq!(as_array.elem_type, PTypeView::ByteType);
    let array_mut = as_array.elems.borrow_mut();
    let src_slice = &array_mut.deref()[offset as usize..((offset + len) as usize)];
    let mut src_buffer: Vec<i8> = vec![];
    for src_elem in src_slice {
        src_buffer.push(src_elem.unwrap_byte());
    }
    assert_eq!(dst_obj, null_mut());
    libc::memcpy(transmute(address), src_buffer.as_ptr() as *const libc::c_void, len as usize);
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(env: *mut JNIEnv,
                                                           the_unsafe: jobject,
                                                           address: jlong,
                                                           byte_: jbyte,
) {
    let byte_addr: *mut jbyte = transmute(address);
    byte_addr.write(byte_);
}

pub mod compare_and_swap {
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
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_objectFieldOffset(env: *mut JNIEnv, the_unsafe: jobject,
                                                                 field_obj: jobject,
) -> jlong {
    let jfield = JavaValue::Object(from_object(field_obj)).cast_field();
    let name = jfield.name().to_rust_string();
    let clazz = jfield.clazz().as_runtime_class();
    let class_view = clazz.view();
    let mut field_i = None;
    &class_view.fields().enumerate().for_each(|(i, f)| {
        if f.field_name() == name {
            field_i = Some(i);
        }
    });
    let jvm = get_state(env);
    let field_id = new_field_id(jvm, clazz, field_i.unwrap());
    transmute(field_id)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    obj: jobject,
    offset: jlong,
) -> jint {
    let jvm = get_state(env);
    let notnull = from_object(obj).unwrap();
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(offset));
    let field_name = rc.view().field(field_i as usize).field_name();
    let field_borrow = notnull.unwrap_normal_object().fields.borrow();
    field_borrow.get(&field_name).unwrap().unwrap_int()
}