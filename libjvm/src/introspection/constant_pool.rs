use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::loading::ClassLoadingError;
use classfile_view::view::constant_info_view::ConstantInfoView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jclass, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jobjectArray, jstring};
use slow_interpreter::class_loading::{check_initing_or_inited_class, check_loaded_class};
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::java::lang::reflect::constant_pool::ConstantPool;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::Object;
use slow_interpreter::rust_jni::native_util::{from_jclass, get_interpreter_state, get_state, to_object};

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
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    view.constant_pool_size() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(c) => {
            match get_or_create_class_object(jvm, PTypeView::Ref(c.class_name()), int_state) {
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
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Integer(int_) => int_.int,
        _ => todo!("unclear what to do here")
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jlong {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Long(long_) => long_.long,
        _ => todo!("unclear what to do here")//should throw illregal arg exception
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jfloat {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    //todo bounds check
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Float(float_) => float_.float,
        _ => todo!("unclear what to do here")
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jdouble {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Double(double_) => double_.double,
        _ => todo!("unclear what to do here")
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::String(string) => to_object(JString::from_rust(jvm, int_state, string.string()).object().into()),
        _ => todo!("unclear what to do here")
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let view = from_jclass(constantPoolOop).as_runtime_class(jvm).view();
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Utf8(utf8) => to_object(JString::from_rust(jvm, int_state, utf8.str.clone()).object().into()),
        _ => todo!("unclear what to do here")
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