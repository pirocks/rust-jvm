use crate::rust_jni::native_util::{from_object, to_object, get_state, get_frame, from_jclass};
use jvmti_jni_bindings::{jint, jfieldID, jobject, JNIEnv, jlong, jclass, jmethodID, _jfieldID, _jobject, jboolean, jshort, jbyte, jchar, jfloat, jdouble};
use std::ops::Deref;
use std::ffi::CStr;
use std::mem::transmute;
use crate::rust_jni::interface::util::{FieldID, class_object_to_runtime_class};
use descriptor_parser::parse_method_descriptor;
use classfile_view::view::HasAccessFlags;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;
use std::sync::Arc;

pub unsafe extern "C" fn get_boolean_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jboolean {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_boolean()
}

pub unsafe extern "C" fn get_byte_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jbyte {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_byte()
}

pub unsafe extern "C" fn get_short_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jshort {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_short()
}

pub unsafe extern "C" fn get_char_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jchar {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_char()
}

pub unsafe extern "C" fn get_int_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jint {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_int() as jint
}

pub unsafe extern "C" fn get_long_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jlong {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_long() as jlong
}

pub unsafe extern "C" fn get_float_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jfloat {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_float()
}

pub unsafe extern "C" fn get_double_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jdouble {
    let java_value = get_java_value_field(obj, field_id_raw);
    java_value.unwrap_double()
}

pub unsafe extern "C" fn get_object_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jobject {
    let java_value = get_java_value_field(obj, field_id_raw);
    to_object(java_value.unwrap_object())
}


unsafe fn get_java_value_field(obj: *mut _jobject, field_id_raw: *mut _jfieldID) -> JavaValue {
    let field_id: &FieldID = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let view = &field_id.class.view();
    let name = view.field(field_id.field_i as usize).field_name();
    let notnull = from_object(obj).unwrap();
    let normal_obj = notnull.unwrap_normal_object();
    let fields_borrow = normal_obj.fields.borrow();
    fields_borrow.deref().get(&name).unwrap().clone()
}


pub unsafe extern "C" fn get_field_id(_env: *mut JNIEnv, clazz: jclass, c_name: *const ::std::os::raw::c_char, _sig: *const ::std::os::raw::c_char) -> jfieldID {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let runtime_class = from_jclass(clazz).as_runtime_class();
    let view = &runtime_class.view();
    for field_i in 0..view.num_fields() {
        //todo check descriptor
        if view.field(field_i).field_name() == name {
            return new_field_id(runtime_class, field_i);
        }
    }
    panic!()
}

pub(crate) fn new_field_id(runtime_class: Arc<RuntimeClass>, field_i: usize) -> jfieldID {
    Box::into_raw(Box::new(FieldID { class: runtime_class.clone(), field_i })) as jfieldID
}


pub unsafe extern "C" fn get_static_method_id(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const ::std::os::raw::c_char,
    sig: *const ::std::os::raw::c_char,
) -> jmethodID {
    let jvm = get_state(env);
    let frame = get_frame(env);
    let method_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj_o = from_object(clazz);
    //todo dup
    let runtime_class = class_object_to_runtime_class(&JavaValue::Object(class_obj_o).cast_class(), jvm, &frame).unwrap();
    let view = &runtime_class.view();
    let method = view.method_index().lookup(&method_name, &parse_method_descriptor(method_descriptor_str.as_str()).unwrap()).unwrap();
    assert!(method.is_static());
    let res = Box::into_raw(box jvm.method_table
        .write()
        .unwrap()
        .register_with_table(runtime_class.clone(), method.method_i() as u16));
    transmute(res)
}


pub unsafe extern "C" fn get_static_field_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jfieldID {
//    get_frame(env).print_stack_trace();
    //todo should have its own impl
    get_field_id(env, clazz, name, sig)
}



