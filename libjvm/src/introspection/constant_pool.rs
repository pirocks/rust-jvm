use std::hint::unreachable_unchecked;
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::loading::ClassLoadingError;
use classfile_view::view::ClassView;
use classfile_view::view::constant_info_view::ConstantInfoView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{_jobject, jclass, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jobjectArray, jstring};
use slow_interpreter::class_loading::{check_initing_or_inited_class, check_loaded_class};
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::reflect::constant_pool::ConstantPool;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::Object;
use slow_interpreter::rust_jni::native_util::{from_jclass, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{throw_array_out_of_bounds, throw_array_out_of_bounds_res, throw_illegal_arg, throw_illegal_arg_res};

#[no_mangle]
unsafe extern "system" fn JVM_GetClassConstantPool(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let constant_pool = ConstantPool::new(jvm, int_state, from_jclass(cls));
    to_object(constant_pool.object().into())
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetSize(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject) -> jint {
    let jvm = get_state(env);
    let runtimec_lass = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = runtimec_lass.view();
    view.constant_pool_size() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds(jvm, int_state, index);
        return null_mut();
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(c) => {
            match get_or_create_class_object(jvm, PTypeView::Ref(c.class_ref_type()), int_state) {
                Ok(class_obj) => to_object(class_obj.into()),
                Err(_) => null_mut()
            }
        }
        _ => null_mut()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetIntAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds(jvm, int_state, index);
        return 0;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Integer(int_) => int_.int,
        _ => {
            throw_illegal_arg(jvm, int_state);
            return -1;
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jlong {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds(jvm, int_state, index);
        return 0;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Long(long_) => long_.long,
        _ => {
            throw_illegal_arg(jvm, int_state);
            return -1;
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jfloat {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds(jvm, int_state, index);
        return -1f32;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Float(float_) => float_.float,
        _ => {
            throw_illegal_arg(jvm, int_state);
            return -1f32;
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jdouble {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds(jvm, int_state, index);
        return -1f64;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Double(double_) => double_.double,
        _ => {
            throw_illegal_arg(jvm, int_state);
            return -1f64;
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    match ConstantPoolGetStringAt_impl(env, constantPoolOop, index) {
        Ok(res) => res,
        Err(_) => null_mut()
    }
}

unsafe fn ConstantPoolGetStringAt_impl(env: *mut JNIEnv, constantPoolOop: *mut _jobject, index: i32) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::String(string) => Ok(to_object(JString::from_rust(jvm, int_state, string.string())?.object().into())),
        _ => {
            throw_illegal_arg_res(jvm, int_state)?;
            Ok(unreachable!())
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    match ConstantPoolGetUTF8At_impl(env, constantPoolOop, index) {
        Ok(res) => res,
        Err(WasException {}) => null_mut()
    }
}

unsafe fn ConstantPoolGetUTF8At_impl(env: *mut JNIEnv, constantPoolOop: jobject, index: i32) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Utf8(utf8) => Ok(to_object(JString::from_rust(jvm, int_state, utf8.str.clone())?.object().into())),
        _ => {
            throw_illegal_arg_res(jvm, int_state)?;
            Ok(unreachable!())
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPTypes(env: *mut JNIEnv, cb: jclass, types: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPEntriesCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int, calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int, calledClass: jclass) -> jint {
    unimplemented!()
}