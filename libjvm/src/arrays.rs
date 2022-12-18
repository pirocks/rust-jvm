use std::ffi::{c_uchar, c_void};
use std::num::NonZeroU8;
use std::ptr::{NonNull};

use itertools::Itertools;

use array_memory_layout::accessor::Accessor;
use array_memory_layout::layout::ArrayMemoryLayout;
use gc_memory_layout_common::array_copy_no_validate;
use gc_memory_layout_common::memory_regions::MemoryRegions;
use jvmti_jni_bindings::{jclass, jint, jintArray, JNIEnv, jobject, jvalue, JVM_T_BOOLEAN, JVM_T_BYTE, JVM_T_CHAR, JVM_T_DOUBLE, JVM_T_FLOAT, JVM_T_INT, JVM_T_LONG, JVM_T_SHORT};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use slow_interpreter::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter::common::new::a_new_array_from_name;
use slow_interpreter::ir_to_java_layer::exit_impls::multi_allocate_array::multi_new_array_impl;
use slow_interpreter::java_values::{default_value, ExceptionReturn};
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object_new};
use slow_interpreter::stdlib::java::lang::index_out_of_bounds_exception::IndexOutOfBoundsException;
use slow_interpreter::stdlib::java::lang::null_pointer_exception::NullPointerException;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::{java_value_to_boxed_object, throw_array_out_of_bounds, throw_illegal_arg_res, throw_npe, throw_npe_res};
use crate::reflection::unwrap_boxed_java_value;

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewArray(_env: *mut JNIEnv, _obj: jobject, _currClass: jclass, _length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayLength(env: *mut JNIEnv, arr: jobject) -> jint {
    match get_array(env, arr) {
        Ok(jv) => jv.unwrap_object_nonnull().unwrap_array().len() as i32,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            jint::invalid_default()
        }
    }
}

unsafe fn get_array<'gc>(env: *mut JNIEnv, arr: jobject) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match from_object_new(jvm, arr) {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(possibly_arr) => {
            if possibly_arr.is_array(jvm) {
                Ok(NewJavaValueHandle::Object(possibly_arr))
            } else {
                return throw_illegal_arg_res(jvm, int_state);
            }
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    match get_array(env, arr) {
        Ok(jv) => {
            let nonnull = jv.unwrap_object_nonnull();
            let array = nonnull.unwrap_array();
            let elem_type = array.elem_cpdtype();
            let len = array.len() as i32;
            if index < 0 || index >= len {
                return throw_array_out_of_bounds(jvm, int_state, throw, index);
            }
            let java_value = array.get_i(index);
            let owned = if let NewJavaValue::AllocObject(_obj) = java_value.as_njv() {
                java_value.unwrap_object()
            } else {
                match java_value_to_boxed_object(jvm, int_state, java_value.as_njv(), elem_type) {
                    Ok(boxed) => boxed,
                    Err(WasException { exception_obj }) => {
                        *get_throw(env) = Some(WasException { exception_obj });
                        return jobject::invalid_default()
                    }
                }
            };
            new_local_ref_public_new(
                owned.as_ref().map(|boxed| boxed.as_allocated_obj()),
                int_state,
            )
        }
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jobject::invalid_default()
        }
    }
}

pub fn v_code_to_cpdtype(code: u8) -> CPDType {
    match code as u32 {
        JVM_T_BOOLEAN => CPDType::BooleanType,
        JVM_T_CHAR => CPDType::CharType,
        JVM_T_FLOAT => CPDType::FloatType,
        JVM_T_DOUBLE => CPDType::DoubleType,
        JVM_T_BYTE => CPDType::ByteType,
        JVM_T_SHORT => CPDType::ShortType,
        JVM_T_INT => CPDType::IntType,
        JVM_T_LONG => CPDType::LongType,
        other => {
            dbg!(other);
            panic!()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, wCode: jint) -> jvalue {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let array_subtype = v_code_to_cpdtype(wCode as u8);
    let memory_layout = ArrayMemoryLayout::from_cpdtype(array_subtype);
    let accessor = memory_layout.calculate_index_address(match NonNull::new(arr as *mut c_void) {
        Some(x) => x,
        None => {
            let npe = NullPointerException::new(jvm, int_state).unwrap();
            *throw = Some(WasException { exception_obj: npe.full_object().cast_throwable() });
            return jvalue::invalid_default();
        }
    }, index);
    match array_subtype {
        CPDType::BooleanType => {
            jvalue { z: accessor.read_boolean() }
        }
        CPDType::ByteType => {
            jvalue { b: accessor.read_byte() }
        }
        CPDType::ShortType => {
            jvalue { s: accessor.read_short() }
        }
        CPDType::CharType => {
            jvalue { c: accessor.read_char() }
        }
        CPDType::IntType => {
            jvalue { i: accessor.read_int() }
        }
        CPDType::LongType => {
            jvalue { j: accessor.read_long() }
        }
        CPDType::FloatType => {
            jvalue { f: accessor.read_float() }
        }
        CPDType::DoubleType => {
            jvalue { d: accessor.read_double() }
        }
        _ => panic!()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_SetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, val: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let obj_to_set = NewJavaValueHandle::from_optional_object(from_object_new(jvm, val));
    match get_array(env, arr) {
        Ok(array) => {
            match array.unwrap_object(){
                None => {
                    return throw_npe(jvm, int_state, throw);
                }
                Some(array) => {
                    let array = array.unwrap_array();
                    let elem_subtype = array.elem_cpdtype();
                    if elem_subtype.is_primitive(){
                        array.set_i(index, unwrap_boxed_java_value(jvm, obj_to_set, &elem_subtype).as_njv());
                    }else {
                        //todo impl array store exception
                        array.set_i(index, obj_to_set.as_njv());
                    }
                }
            }
        }
        Err(WasException{ exception_obj }) => {
            *throw = Some(WasException{ exception_obj });
            return;
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_SetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, v: jvalue, vCode: c_uchar) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let array_subtype = v_code_to_cpdtype(vCode);
    let memory_layout = ArrayMemoryLayout::from_cpdtype(array_subtype);
    let accessor = memory_layout.calculate_index_address(match NonNull::new(arr as *mut c_void) {
        Some(x) => x,
        None => {
            let npe = NullPointerException::new(jvm, int_state).unwrap();
            *throw = Some(WasException { exception_obj: npe.full_object().cast_throwable() });
            return ;
        }
    }, index);
    match array_subtype {
        CPDType::BooleanType => {
            accessor.write_boolean(v.z);
        }
        CPDType::ByteType => {
            accessor.write_byte(v.b);
        }
        CPDType::ShortType => {
            accessor.write_short(v.s);
        }
        CPDType::CharType => {
            accessor.write_char(v.c);
        }
        CPDType::IntType => {
            accessor.write_int(v.i);
        }
        CPDType::LongType => {
            accessor.write_long(v.j);
        }
        CPDType::FloatType => {
            accessor.write_float(v.f);
        }
        CPDType::DoubleType => {
            accessor.write_double(v.d);
        }
        _ => panic!()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_NewArray(env: *mut JNIEnv, eltClass: jclass, length: jint) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let _ = assert_inited_or_initing_class(jvm, CPDType::class());
    from_jclass(jvm, eltClass).debug_assert(jvm);
    let array_type_name = from_jclass(jvm, eltClass).as_runtime_class(jvm).cpdtype();
    let res = a_new_array_from_name(jvm, int_state, length, array_type_name).unwrap();
    new_local_ref_public_new(res.unwrap_object().as_ref().map(|handle| handle.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewMultiArray(env: *mut JNIEnv, eltClass: jclass, dim: jintArray) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, eltClass).as_runtime_class(jvm);
    let dims = from_object_new(jvm, dim).unwrap().unwrap_array().array_iterator().map(|njv| njv.unwrap_int()).collect_vec();
    //todo dupe with the multi new array exit
    let num_arrays = dims.len();
    let elem_type = rc.cpdtype().unwrap_non_array();
    let array_type = CPDType::Array { base_type: elem_type, num_nested_arrs: NonZeroU8::new(num_arrays as u8).unwrap() };
    let _ = check_initing_or_inited_class(jvm, int_state, array_type).unwrap();
    let default = default_value(elem_type.to_cpdtype());
    let res = multi_new_array_impl(jvm, array_type, dims.as_slice(), default.as_njv());
    new_local_ref_public_new(res.unwrap_object().as_ref().map(|handle| handle.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_ArrayCopy(env: *mut JNIEnv, _ignored: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    let array_elem_type = MemoryRegions::find_object_region_header(NonNull::new(src).unwrap().cast()).array_elem_type.unwrap();
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_elem_type);
    let src_len = array_layout.calculate_len_address(NonNull::new(src).unwrap().cast()).as_ptr().read();
    let dest_len = array_layout.calculate_len_address(NonNull::new(dst).unwrap().cast()).as_ptr().read();

    if src_pos < 0 || dst_pos < 0 || length < 0 || src_pos + length > src_len as i32 || dst_pos + length > dest_len as i32 {
        let jvm = get_state(env);
        let int_state = get_interpreter_state(env);
        *get_throw(env) = Some(WasException { exception_obj: IndexOutOfBoundsException::new(jvm, int_state).unwrap().object().cast_throwable() });
        return;
    }

    array_copy_no_validate(src, src_pos, dst, dst_pos, length);
}

