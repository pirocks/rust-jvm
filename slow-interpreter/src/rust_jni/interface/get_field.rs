use crate::rust_jni::native_util::{from_object, to_object};
use jni_bindings::{jint, jfieldID, jobject, JNIEnv, jlong, jclass, jmethodID};
use std::ops::Deref;
use std::ffi::CStr;
use std::mem::transmute;
use crate::rust_jni::MethodId;
use crate::rust_jni::interface::util::{FieldID, runtime_class_from_object};

pub unsafe extern "C" fn get_long_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jlong {
    let field_id: &FieldID = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow().deref().get(&name).unwrap().unwrap_long() as jlong
}


pub unsafe extern "C" fn get_int_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jint {
    let field_id: &FieldID = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let classfile = &field_id.class.classfile;
    let name = classfile.fields[field_id.field_i as usize].name(classfile);
    from_object(obj).unwrap().unwrap_normal_object().fields.borrow().deref().get(&name).unwrap().unwrap_int() as jint
}


pub unsafe extern "C" fn get_object_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jobject {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let nonnull = from_object(obj).unwrap();
    let field_borrow = nonnull.unwrap_normal_object().fields.borrow();
    let fields = field_borrow.deref();
    let classfile = &field_id.class.classfile;
    let field_name = classfile.fields[field_id.field_i].name(classfile);
    to_object(fields.get(&field_name).unwrap().unwrap_object())
}


pub unsafe extern "C" fn get_field_id(_env: *mut JNIEnv, clazz: jclass, c_name: *const ::std::os::raw::c_char, _sig: *const ::std::os::raw::c_char) -> jfieldID {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let runtime_class = runtime_class_from_object(clazz).unwrap();
    let fields = &runtime_class.classfile.fields;
    for field_i in 0..fields.len() {
        //todo check descriptor
        if fields[field_i].name(&runtime_class.classfile) == name {
            return Box::into_raw(Box::new(FieldID { class: runtime_class.clone(), field_i })) as jfieldID;
        }
    }
    panic!()
}


pub unsafe extern "C" fn get_static_method_id(
    _env: *mut JNIEnv,
    clazz: jclass,
    name: *const ::std::os::raw::c_char,
    sig: *const ::std::os::raw::c_char,
) -> jmethodID {
    let method_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj_o = from_object(clazz).unwrap();
    let class_obj = class_obj_o.unwrap_normal_object();
    //todo dup
    let runtime_class = class_obj.object_class_object_pointer.borrow().as_ref().unwrap().clone();
    let classfile = &runtime_class.classfile;
    let (method_i, method) = classfile.lookup_method(method_name, method_descriptor_str).unwrap();
    assert!(method.is_static());
    let res = Box::into_raw(Box::new(MethodId { class: runtime_class.clone(), method_i }));
    transmute(res)
}


pub unsafe extern "C" fn get_static_field_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jfieldID {
//    get_frame(env).print_stack_trace();
    //todo should have its own impl
    get_field_id(env, clazz, name, sig)
}


