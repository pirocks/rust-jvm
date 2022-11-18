use std::os::raw::c_void;
use std::ptr::NonNull;
use array_memory_layout::accessor::Accessor;
use array_memory_layout::layout::ArrayMemoryLayout;

use jvmti_jni_bindings::{jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jdouble, jdoubleArray, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, JNI_ABORT, JNI_COMMIT, JNIEnv, jobject, jobjectArray, jshort, jshortArray, jsize};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use slow_interpreter::new_java_values::allocated_objects::AllocatedObject;
use slow_interpreter::new_java_values::{NewJavaValueHandle};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::utils::throw_npe;

pub unsafe extern "C" fn get_array_length(env: *mut JNIEnv, array: jarray) -> jsize {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let temp = match from_object_new(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state, get_throw(env));
        }
    };
    return temp.unwrap_array().len() as jsize;
    // let non_null_array: &Object = temp.deref();
    /*let len = match non_null_array {
        Object::Array(a) => a.len(),
        Object::Object(_o) => {
            return throw_illegal_arg(jvm, int_state);
        }
    };
    len as jsize*/
}

pub unsafe extern "C" fn get_object_array_element(env: *mut JNIEnv, array: jobjectArray, index: jsize) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let notnull = match from_object_new(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state, get_throw(env));
        }
    };
    let int_state = get_interpreter_state(env);
    let array = notnull.unwrap_array();
    new_local_ref_public_new(array.get_i(index).unwrap_object().as_ref().map(|handle| AllocatedObject::Handle(handle)), int_state)
}

pub unsafe extern "C" fn set_object_array_element(env: *mut JNIEnv, array: jobjectArray, index: jsize, val: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let notnull = match from_object_new(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state, get_throw(env));
        }
    };
    let array = notnull.unwrap_array();
    array.set_i(index, NewJavaValueHandle::from_optional_object(from_object_new(jvm, val)).as_njv());
}

pub mod array_region;
pub mod new;

pub fn array_fast_copy_set<T>(carray: *const T, array_layout: ArrayMemoryLayout, raw_array: NonNull<c_void>, len: i32) {
    for i in 0..len {
        let accessor = array_layout.calculate_index_address(raw_array,i);
        let to_write = unsafe { carray.offset(i as isize).read() };
        accessor.write_impl(to_write);
    }
}

pub unsafe extern "C" fn release_primitive_array_critical(env: *mut JNIEnv, raw_array: jarray, carray: *mut c_void, mode: jint) {
    // assert_eq!(mode, 0);
    if mode == JNI_ABORT as i32 {
        return;
    }
    //todo handle JNI_COMMIT
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let not_null = match from_object_new(jvm, raw_array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
    };
    let array = not_null.unwrap_array();
    let array_subtype = &array.elem_cpdtype();
    // for i in 0..array.len() {
    let array_memory_layout = ArrayMemoryLayout::from_cpdtype(*array_subtype);
    match array_subtype {
        CPDType::ByteType => {
            array_fast_copy_set::<jbyte>(carray as *const jbyte, array_memory_layout, NonNull::new(raw_array as *mut c_void).unwrap(), array.len())
            // array.set_i(i, NewJavaValue::Byte((carray as *const jbyte).offset(i as isize).read()));
        }
        CPDType::CharType => {
            // array.set_i(jvm, i, JavaValue::Char((carray as *const jchar).offset(i as isize).read()));
            todo!()
        }
        CPDType::DoubleType => {
            // array.set_i(jvm, i, JavaValue::Double((carray as *const jdouble).offset(i as isize).read()));
            todo!()
        }
        CPDType::FloatType => {
            // array.set_i(jvm, i, JavaValue::Float((carray as *const jfloat).offset(i as isize).read()));
            todo!()
        }
        CPDType::IntType => {
            array_fast_copy_set::<jint>(carray as *const jint, array_memory_layout, NonNull::new(raw_array as *mut c_void).unwrap(), array.len())
            /*array_fast_copy_set::<jint>(carray as *const jint, array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *mut jvalue,array.len())*/
            // array.set_i(i, NewJavaValue::Int((carray as *const jint).offset(i as isize).read()));
        }
        CPDType::LongType => {
            // array.set_i(jvm, i, JavaValue::Long((carray as *const jlong).offset(i as isize).read()));
            todo!()
        }
        CPDType::Class(_) | CPDType::Array { .. } => {
            // array.set_i(jvm, i, JavaValue::Object(from_object(jvm, (carray as *const jobject).offset(i as isize).read())));
            todo!()
        }
        CPDType::ShortType => {
            // array.set_i(jvm, i, JavaValue::Short((carray as *const jshort).offset(i as isize).read()));
            todo!()
        }
        CPDType::BooleanType => {
            // let boolean = (carray as *const jboolean).offset(i as isize).read();
            // assert!(boolean == 1 || boolean == 0);
            // array.set_i(jvm, i, JavaValue::Boolean(boolean));
            todo!()
        }
        _ => panic!(),
    }
    // }
    if mode != JNI_COMMIT as jint {
        //todo free carray
    }
}

pub fn array_fast_copy_get<T>(memory_layout: ArrayMemoryLayout, array: NonNull<c_void>, len: i32) -> Vec<T> {
    let mut vec = Vec::with_capacity(len as usize);
    for i in 0..len {
        let val: T = memory_layout.calculate_index_address(array, i).read_impl();
        vec.push(val);
    }
    vec
}

pub unsafe extern "C" fn get_primitive_array_critical(env: *mut JNIEnv, array_raw: jarray, is_copy: *mut jboolean) -> *mut c_void {
    //todo this is slow for some reason?
    // todo fast path copy for non-object arrays?
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let not_null = match from_object_new(jvm, array_raw) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
    };
    let array = not_null.unwrap_array();
    if !is_copy.is_null() {
        is_copy.write(true as jboolean);
    }
    //dup but difficult to make into template so ehh
    match &array.elem_cpdtype() {
        CPDType::ByteType => {
            let res: Vec<jbyte> = array_fast_copy_get::<jbyte>(ArrayMemoryLayout::from_cpdtype(array.elem_cpdtype()), NonNull::new(array_raw as *mut c_void).unwrap(), array.len());
            // let res = array.array_iterator().map(|elem| elem.unwrap_byte_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::CharType => {
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jchar>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            // let res = array.array_iterator().map(|elem| elem.unwrap_char_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::DoubleType => {
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jdouble>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            // let res = array.array_iterator().map(|elem| elem.unwrap_double_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::FloatType => {
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jfloat>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            // let res = array.array_iterator().map(|elem| elem.unwrap_float_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::IntType => {
            let res: Vec<jint> = array_fast_copy_get::<jint>(ArrayMemoryLayout::from_cpdtype(array.elem_cpdtype()), NonNull::new(array_raw as *mut c_void).unwrap(), array.len());
            // let res = array.array_iterator().map(|elem| elem.unwrap_int_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::LongType => {
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jlong>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            // let res = array.array_iterator().map(|elem| elem.unwrap_long_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::ShortType => {
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jshort>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            // let res = array.array_iterator().map(|elem| elem.unwrap_short_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::BooleanType => {
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jboolean>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            // let res = array.array_iterator().map(|elem| elem.unwrap_bool_strict()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::Class(_) | CPDType::Array { .. } => {
            // let res = array.array_iterator().map(|elem| to_object_new(elem.unwrap_object().as_ref().map(|handle| handle.as_allocated_obj()))).collect::<Vec<_>>();
            let res: Vec<()> = todo!("array fast copy should use array layout or maybe be part of array layout");/*array_fast_copy_get::<jboolean>(array.ptr.as_ptr().offset(size_of::<jlong>() as isize) as *const jvalue, array.len());*/
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        _ => panic!(),
    }
}

pub unsafe extern "C" fn get_byte_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jbyte {
    get_primitive_array_critical(env, array, is_copy) as *mut jbyte
}

pub unsafe extern "C" fn get_char_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jchar {
    get_primitive_array_critical(env, array, is_copy) as *mut jchar
}

pub unsafe extern "C" fn get_double_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jdouble {
    get_primitive_array_critical(env, array, is_copy) as *mut jdouble
}

pub unsafe extern "C" fn get_float_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jfloat {
    get_primitive_array_critical(env, array, is_copy) as *mut jfloat
}

pub unsafe extern "C" fn get_int_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jint {
    get_primitive_array_critical(env, array, is_copy) as *mut jint
}

pub unsafe extern "C" fn get_short_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jshort {
    get_primitive_array_critical(env, array, is_copy) as *mut jshort
}

pub unsafe extern "C" fn get_boolean_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jboolean {
    get_primitive_array_critical(env, array, is_copy) as *mut jboolean
}

pub unsafe extern "C" fn get_object_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jobject {
    get_primitive_array_critical(env, array, is_copy) as *mut jobject
}

pub unsafe extern "C" fn get_long_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jlong {
    get_primitive_array_critical(env, array, is_copy) as *mut jlong
}

pub unsafe extern "C" fn release_byte_array_elements(env: *mut JNIEnv, array: jbyteArray, elems: *mut jbyte, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_char_array_elements(env: *mut JNIEnv, array: jcharArray, elems: *mut jchar, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_double_array_elements(env: *mut JNIEnv, array: jdoubleArray, elems: *mut jdouble, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_float_array_elements(env: *mut JNIEnv, array: jfloatArray, elems: *mut jfloat, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_int_array_elements(env: *mut JNIEnv, array: jintArray, elems: *mut jint, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_short_array_elements(env: *mut JNIEnv, array: jshortArray, elems: *mut jshort, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_boolean_array_elements(env: *mut JNIEnv, array: jbooleanArray, elems: *mut jboolean, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_object_array_elements(env: *mut JNIEnv, array: jobjectArray, elems: *mut jobject, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_long_array_elements(env: *mut JNIEnv, array: jlongArray, elems: *mut jlong, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}