use std::borrow::Borrow;
use std::ffi::CStr;
use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use descriptor_parser::parse_method_descriptor;
use jvmti_jni_bindings::{_jfieldID, _jobject, jboolean, jbyte, jchar, jclass, jdouble, jfieldID, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort};

use crate::java_values::JavaValue;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::interface::util::class_object_to_runtime_class;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};

pub unsafe extern "C" fn get_boolean_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jboolean {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_boolean()
}

pub unsafe extern "C" fn get_byte_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jbyte {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_byte()
}

pub unsafe extern "C" fn get_short_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jshort {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_short()
}

pub unsafe extern "C" fn get_char_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jchar {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_char()
}

pub unsafe extern "C" fn get_int_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jint {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_int() as jint
}

pub unsafe extern "C" fn get_long_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jlong {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_long() as jlong
}

pub unsafe extern "C" fn get_float_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jfloat {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_float()
}

pub unsafe extern "C" fn get_double_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jdouble {
    let java_value = get_java_value_field(env, obj, field_id_raw);
    java_value.unwrap_double()
}

pub unsafe extern "C" fn get_object_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jobject {
    let int_state = get_interpreter_state(env);
    let java_value = get_java_value_field(env, obj, field_id_raw);

    new_local_ref_public(java_value.unwrap_object(), int_state)
}


unsafe fn get_java_value_field(env: *mut JNIEnv, obj: *mut _jobject, field_id_raw: *mut _jfieldID) -> JavaValue {
    let (rc, field_i) = get_state(env).field_table.read().unwrap().lookup(transmute(field_id_raw));
    let view = &rc.view();
    let name = view.field(field_i as usize).field_name();
    let notnull = from_object(obj).unwrap();
    let normal_obj = notnull.unwrap_normal_object();
    let fields_borrow = normal_obj.fields.borrow();
    fields_borrow.deref().get(&name).unwrap().clone()
}


pub unsafe extern "C" fn get_field_id(env: *mut JNIEnv, clazz: jclass, c_name: *const ::std::os::raw::c_char, _sig: *const ::std::os::raw::c_char) -> jfieldID {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let runtime_class = from_jclass(clazz).as_runtime_class();
    let view = &runtime_class.view();
    for field_i in 0..view.num_fields() {
        //todo check descriptor
        if view.field(field_i).field_name() == name {
            return new_field_id(get_state(env), runtime_class, field_i);
        }
    }
    panic!()
}

pub fn new_field_id(jvm: &'static JVMState, runtime_class: Arc<RuntimeClass>, field_i: usize) -> jfieldID {
    let id = jvm.field_table.write().unwrap().register_with_table(runtime_class, field_i as u16);
    unsafe { transmute(id) }
}


pub unsafe extern "C" fn get_static_method_id(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const ::std::os::raw::c_char,
    sig: *const ::std::os::raw::c_char,
) -> jmethodID {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    // let frame = int_state.current_frame_mut();
    let method_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj_o = from_object(clazz);
    //todo dup
    let runtime_class = class_object_to_runtime_class(&JavaValue::Object(class_obj_o).cast_class(), jvm, int_state).unwrap();
    let view = &runtime_class.view();
    let method = view.lookup_method(&method_name, &parse_method_descriptor(method_descriptor_str.as_str()).unwrap()).unwrap();
    assert!(method.is_static());
    let res = Box::into_raw(box jvm.method_table
        .write()
        .unwrap()
        .register_with_table(runtime_class.clone(), method.method_i() as u16));
    transmute(res)
}


pub unsafe extern "C" fn get_static_field_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jfieldID {
//    get_frame(&mut get_frames(env)).print_stack_trace();
    //todo should have its own impl
    get_field_id(env, clazz, name, sig)
}

unsafe fn get_static_field(env: *mut JNIEnv, klass: jclass, field_id_raw: jfieldID) -> JavaValue {
    let jvm = get_state(env);
    let (rc, field_i) = jvm.field_table.write().unwrap().lookup(transmute(field_id_raw));
    let view = rc.view();
    let name = view.field(field_i as usize).field_name();
    let jclass = from_jclass(klass);
    jclass.as_runtime_class().static_vars().borrow().get(&name).unwrap().clone()
}


pub unsafe extern "C" fn get_static_object_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jobject {
    let int_state = get_interpreter_state(env);
    new_local_ref_public(get_static_field(env, clazz, field_id).unwrap_object(), int_state)
}

pub unsafe extern "C" fn get_static_boolean_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jboolean {
    get_static_field(env, clazz, field_id).unwrap_boolean()
}

pub unsafe extern "C" fn get_static_byte_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jbyte {
    get_static_field(env, clazz, field_id).unwrap_byte()
}

pub unsafe extern "C" fn get_static_short_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jshort {
    get_static_field(env, clazz, field_id).unwrap_short()
}

pub unsafe extern "C" fn get_static_char_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jchar {
    get_static_field(env, clazz, field_id).unwrap_char()
}

pub unsafe extern "C" fn get_static_int_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jint {
    get_static_field(env, clazz, field_id).unwrap_int()
}

pub unsafe extern "C" fn get_static_long_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jlong {
    get_static_field(env, clazz, field_id).unwrap_long()
}

pub unsafe extern "C" fn get_static_float_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jfloat {
    get_static_field(env, clazz, field_id).unwrap_float()
}

pub unsafe extern "C" fn get_static_double_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jdouble {
    get_static_field(env, clazz, field_id).unwrap_double()
}


