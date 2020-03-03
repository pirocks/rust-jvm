use crate::rust_jni::native_util::{from_object, get_state, get_frame};
use runtime_common::java_values::JavaValue;
use jni_bindings::{jboolean, jfieldID, jobject, JNIEnv, jlong, jint, jclass};
use std::ops::DerefMut;
use crate::rust_jni::interface::util::{FieldID, runtime_class_from_object};

pub unsafe extern "C" fn set_int_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jint) {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow_mut().deref_mut().insert(name, JavaValue::Int(val));
}

pub unsafe extern "C" fn set_long_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jlong) {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow_mut().deref_mut().insert(name, JavaValue::Long(val));
}

pub unsafe extern "C" fn set_boolean_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jboolean) {
    let field_id: &FieldID = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow_mut().deref_mut().insert(name, JavaValue::Boolean(val != 0));
}

pub unsafe extern "C" fn set_static_object_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jobject) {
//Box::into_raw(Box::new(FieldID { class: runtime_class.clone(), field_i })) as jfieldID;
    let state = get_state(env);
    let frame = get_frame(env);
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));//todo leak
    let value = from_object(value);
    let classfile = &field_id.class.classfile;
    let field_name = classfile.constant_pool[classfile.fields[field_id.field_i].name_index as usize].extract_string_from_utf8();
    let static_class = runtime_class_from_object(clazz,state,&frame).unwrap();
    static_class.static_vars.borrow_mut().insert(field_name, JavaValue::Object(value));
}



