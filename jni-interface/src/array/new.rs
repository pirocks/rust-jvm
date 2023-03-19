use jvmti_jni_bindings::{jarray, jbooleanArray, jbyteArray, jcharArray, jclass, jdoubleArray, jfloatArray, jintArray, jlongArray, JNIEnv, jobject, jobjectArray, jshortArray, jsize};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use slow_interpreter::java_values::default_value_njv;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::unallocated_objects::UnAllocatedObject;
use slow_interpreter::rust_jni::jni_utils::{get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object_new};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::throw_utils::throw_npe;

pub unsafe extern "C" fn new_object_array(env: *mut JNIEnv, len: jsize, clazz: jclass, init: jobject) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let type_ = from_jclass(jvm, clazz).as_type(jvm);
    let res = new_array(env, len, type_);
    let res_safe = match from_object_new(jvm, res) {
        Some(x) => x,
        None => return throw_npe(jvm, int_state,get_throw(env)),
    };
    let array = res_safe.unwrap_array();
    for i in 0..array.len() {
        array.set_i(i, NewJavaValueHandle::from_optional_object(from_object_new(jvm, init)).as_njv());
    }
    res
}

pub unsafe extern "C" fn new_boolean_array(env: *mut JNIEnv, len: jsize) -> jbooleanArray {
    new_array(env, len, CPDType::BooleanType)
}

pub unsafe extern "C" fn new_byte_array(env: *mut JNIEnv, len: jsize) -> jbyteArray {
    new_array(env, len, CPDType::ByteType)
}

pub unsafe extern "C" fn new_short_array(env: *mut JNIEnv, len: jsize) -> jshortArray {
    new_array(env, len, CPDType::ShortType)
}

pub unsafe extern "C" fn new_char_array(env: *mut JNIEnv, len: jsize) -> jcharArray {
    new_array(env, len, CPDType::CharType)
}

pub unsafe extern "C" fn new_int_array(env: *mut JNIEnv, len: jsize) -> jintArray {
    new_array(env, len, CPDType::IntType)
}

pub unsafe extern "C" fn new_long_array(env: *mut JNIEnv, len: jsize) -> jlongArray {
    new_array(env, len, CPDType::LongType)
}

pub unsafe extern "C" fn new_float_array(env: *mut JNIEnv, len: jsize) -> jfloatArray {
    new_array(env, len, CPDType::FloatType)
}

pub unsafe extern "C" fn new_double_array(env: *mut JNIEnv, len: jsize) -> jdoubleArray {
    new_array(env, len, CPDType::DoubleType)
}

unsafe fn new_array<'gc, 'l>(env: *mut JNIEnv, len: i32, elem_type: CPDType) -> jarray {
    let jvm: &'gc JVMState<'gc> = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut the_vec = vec![];
    for _ in 0..len {
        the_vec.push(default_value_njv(&elem_type))
    }
    let rc = check_initing_or_inited_class(jvm, int_state, CPDType::array(elem_type)).unwrap();
    let object_array = UnAllocatedObject::new_array(rc, the_vec);
    new_local_ref_public_new(
        Some(jvm.allocate_object(object_array).as_allocated_obj()),
        int_state,
    )
}
