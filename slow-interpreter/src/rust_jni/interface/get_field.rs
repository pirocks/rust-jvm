use crate::rust_jni::native_util::{from_object, to_object, get_state, get_frame};
use jvmti_jni_bindings::{jint, jfieldID, jobject, JNIEnv, jlong, jclass, jmethodID};
use std::ops::Deref;
use std::ffi::CStr;
use std::mem::transmute;
use crate::rust_jni::MethodId;
use crate::rust_jni::interface::util::{FieldID, runtime_class_from_object, class_object_to_runtime_class};
use descriptor_parser::parse_method_descriptor;
use classfile_view::view::HasAccessFlags;

pub unsafe extern "C" fn get_long_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jlong {
    let field_id: &FieldID = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let view = &field_id.class.view();
    let name = view.field(field_id.field_i as usize).field_name();
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow().deref().get(&name).unwrap().unwrap_long() as jlong
}


pub unsafe extern "C" fn get_int_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jint {
    let field_id: &FieldID = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let view = &field_id.class.view();
    let name = view.field(field_id.field_i as usize).field_name();
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow().deref().get(&name).unwrap().unwrap_int() as jint
}


pub unsafe extern "C" fn get_object_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jobject {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let nonnull = from_object(obj).unwrap();
    let field_borrow = nonnull.unwrap_normal_object().fields.borrow();
    let fields = field_borrow.deref();
    let view = &field_id.class.view();
    let field_name = view.field(field_id.field_i).field_name();
    to_object(fields.get(&field_name).unwrap().unwrap_object())
}


pub unsafe extern "C" fn get_field_id(env: *mut JNIEnv, clazz: jclass, c_name: *const ::std::os::raw::c_char, _sig: *const ::std::os::raw::c_char) -> jfieldID {
    let state = get_state(env);
    let frame = get_frame(env);
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let runtime_class = runtime_class_from_object(clazz,state,&frame).unwrap();
    let view = &runtime_class.view();
    for field_i in 0..view.num_fields() {
        //todo check descriptor
        if view.field(field_i).field_name() == name {
            return Box::into_raw(Box::new(FieldID { class: runtime_class.clone(), field_i })) as jfieldID;
        }
    }
    panic!()
}


pub unsafe extern "C" fn get_static_method_id(
    env: *mut JNIEnv,
    clazz: jclass,
    name: *const ::std::os::raw::c_char,
    sig: *const ::std::os::raw::c_char,
) -> jmethodID {
    let state = get_state(env);
    let frame = get_frame(env);
    let method_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj_o = from_object(clazz).unwrap();
    //todo dup
    let runtime_class = class_object_to_runtime_class(class_obj_o.unwrap_normal_object(),state,&frame).unwrap();
    let view = &runtime_class.view();
    let method = view.method_index().lookup(&method_name, &parse_method_descriptor(method_descriptor_str.as_str()).unwrap()).unwrap();
    assert!(method.is_static());
    let res = Box::into_raw(Box::new(MethodId { class: runtime_class.clone(), method_i: method.method_i() }));
    transmute(res)
}


pub unsafe extern "C" fn get_static_field_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jfieldID {
//    get_frame(env).print_stack_trace();
    //todo should have its own impl
    get_field_id(env, clazz, name, sig)
}



