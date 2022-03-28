use std::borrow::Borrow;
use std::ffi::CStr;
use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{_jfieldID, _jobject, jboolean, jbyte, jchar, jclass, jdouble, jfieldID, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort};
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::descriptor_parser::parse_method_descriptor;

use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::WasException;
use crate::java_values::{ExceptionReturn, JavaValue};
use crate::{JavaValueCommon, JVMState};
use crate::new_java_values::NewJavaValueHandle;
use runtime_class_stuff::RuntimeClass;
use crate::runtime_class::static_vars;
use crate::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use crate::rust_jni::interface::misc::get_all_fields;
use crate::rust_jni::interface::util::class_object_to_runtime_class;
use crate::rust_jni::native_util::{from_jclass, from_object_new, get_interpreter_state, get_state};
use crate::utils::{throw_npe, throw_npe_res};

pub unsafe extern "C" fn get_boolean_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jboolean {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_bool_strict()
}

pub unsafe extern "C" fn get_byte_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jbyte {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_byte_strict()
}

pub unsafe extern "C" fn get_short_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jshort {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_short_strict()
}

pub unsafe extern "C" fn get_char_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jchar {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_char_strict()
}

pub unsafe extern "C" fn get_int_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jint {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_int_strict() as jint
}

pub unsafe extern "C" fn get_long_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jlong {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_long_strict() as jlong
}

pub unsafe extern "C" fn get_float_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jfloat {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_float_strict()
}

pub unsafe extern "C" fn get_double_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jdouble {
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };
    java_value.unwrap_double_strict()
}

pub unsafe extern "C" fn get_object_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID) -> jobject {
    let int_state = get_interpreter_state(env);
    let java_value = match get_java_value_field(env, obj, field_id_raw) {
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
        Ok(res) => res,
    };

    new_local_ref_public_new(java_value.unwrap_object().as_ref().map(|handle|handle.as_allocated_obj()), int_state)
}

unsafe fn get_java_value_field<'gc>(env: *mut JNIEnv, obj: *mut _jobject, field_id_raw: *mut _jfieldID) -> Result<NewJavaValueHandle<'gc>, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(field_id_raw as usize);
    let view = &rc.view();
    let name = view.field(field_i as usize).field_name();
    let notnull = match from_object_new(jvm, obj) {
        Some(x) => x,
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
    };
    Ok(notnull.unwrap_normal_object().get_var(jvm, &rc, name))
}

pub unsafe extern "C" fn get_field_id(env: *mut JNIEnv, clazz: jclass, c_name: *const ::std::os::raw::c_char, _sig: *const ::std::os::raw::c_char) -> jfieldID {
    let jvm = get_state(env);
    let name = jvm.string_pool.add_name(CStr::from_ptr(&*c_name).to_str().unwrap().to_string(), false); //todo handle utf8
    let runtime_class = from_jclass(jvm, clazz).as_runtime_class(jvm);
    let int_state = get_interpreter_state(env);
    let (field_rc, field_i) = match get_all_fields(jvm, int_state, runtime_class, true) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .into_iter()
        .find(|(rc, i)| name == rc.view().field(*i).field_name().0)
        .unwrap(); //unwrap is prob okay, spec doesn't say what to do

    new_field_id(jvm, field_rc, field_i)
}

pub fn new_field_id<'gc>(jvm: &'gc JVMState<'gc>, runtime_class: Arc<RuntimeClass<'gc>>, field_i: usize) -> jfieldID {
    let id = jvm.field_table.write().unwrap().register_with_table(runtime_class, field_i as u16);
    unsafe { transmute(id) }
}

pub unsafe extern "C" fn get_static_method_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jmethodID {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let method_name_string = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_name = MethodName(jvm.string_pool.add_name(method_name_string, false));
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj_o = match from_object_new(jvm, clazz) {
        None => return throw_npe(jvm, int_state),
        Some(class_obj_o) => Some(class_obj_o),
    };
    let runtime_class = match class_object_to_runtime_class(&NewJavaValueHandle::from_optional_object(class_obj_o).cast_class().unwrap(), jvm, int_state) {
        Some(x) => x,
        None => return throw_npe(jvm, int_state),
    };
    let view = &runtime_class.view();
    let c_method_desc = CMethodDescriptor::from_legacy(parse_method_descriptor(method_descriptor_str.as_str()).unwrap(), &jvm.string_pool);
    let method = view.lookup_method(method_name, &c_method_desc).unwrap();
    assert!(method.is_static());
    let res = Box::into_raw(box jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method.method_i() as u16));
    res as jmethodID
}

pub unsafe extern "C" fn get_static_field_id(env: *mut JNIEnv, clazz: jclass, name: *const ::std::os::raw::c_char, sig: *const ::std::os::raw::c_char) -> jfieldID {
    get_field_id(env, clazz, name, sig)
}

unsafe fn get_static_field<'gc>(env: *mut JNIEnv, klass: jclass, field_id_raw: jfieldID) -> Result<JavaValue<'gc>, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.write().unwrap().lookup(field_id_raw as usize);
    let view = rc.view();
    let name = view.field(field_i as usize).field_name();
    let jclass = from_jclass(jvm, klass);
    let rc = jclass.as_runtime_class(jvm);
    check_initing_or_inited_class(jvm, int_state, rc.cpdtype())?;
    let guard = static_vars(rc.deref(),jvm);
    Ok(guard.borrow().get(name).to_jv())
}

pub unsafe extern "C" fn get_static_object_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jobject {
    let int_state = get_interpreter_state(env);
    new_local_ref_public(
        match get_static_field(env, clazz, field_id) {
            Ok(res) => res,
            Err(WasException {}) => return ExceptionReturn::invalid_default(),
        }
            .unwrap_object(),
        int_state,
    )
}

pub unsafe extern "C" fn get_static_boolean_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jboolean {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_boolean()
}

pub unsafe extern "C" fn get_static_byte_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jbyte {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_byte()
}

pub unsafe extern "C" fn get_static_short_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jshort {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_short()
}

pub unsafe extern "C" fn get_static_char_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jchar {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_char()
}

pub unsafe extern "C" fn get_static_int_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jint {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_int()
}

pub unsafe extern "C" fn get_static_long_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jlong {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_long()
}

pub unsafe extern "C" fn get_static_float_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jfloat {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_float()
}

pub unsafe extern "C" fn get_static_double_field(env: *mut JNIEnv, clazz: jclass, field_id: jfieldID) -> jdouble {
    match get_static_field(env, clazz, field_id) {
        Ok(res) => res,
        Err(WasException {}) => return ExceptionReturn::invalid_default(),
    }
        .unwrap_double()
}