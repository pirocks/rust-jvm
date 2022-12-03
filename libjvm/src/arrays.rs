use std::borrow::Borrow;
use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroU8;
use std::ops::Deref;
use std::panic::panic_any;
use std::ptr::{NonNull, null_mut};

use itertools::Itertools;
use libc::{size_t, timer_delete};
use array_memory_layout::layout::ArrayMemoryLayout;
use gc_memory_layout_common::memory_regions::MemoryRegions;

use jvmti_jni_bindings::{jclass, jint, jintArray, JNIEnv, jobject, jvalue};
use runtime_class_stuff::hidden_fields::HiddenJVMField;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use rust_jvm_common::cpdtype_table::CPDTypeID;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter::common::new::a_new_array_from_name;
use slow_interpreter::ir_to_java_layer::exit_impls::multi_allocate_array::multi_new_array_impl;
use slow_interpreter::java_values::{default_value, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::allocated_objects::AllocatedHandle;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw, new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, to_object};
use slow_interpreter::stdlib::java::lang::boolean::Boolean;
use slow_interpreter::stdlib::java::lang::byte::Byte;
use slow_interpreter::stdlib::java::lang::char::Char;
use slow_interpreter::stdlib::java::lang::double::Double;
use slow_interpreter::stdlib::java::lang::float::Float;
use slow_interpreter::stdlib::java::lang::int::Int;
use slow_interpreter::stdlib::java::lang::long::Long;
use slow_interpreter::stdlib::java::lang::short::Short;
use slow_interpreter::utils::{java_value_to_boxed_object, throw_array_out_of_bounds, throw_array_out_of_bounds_res, throw_illegal_arg_res, throw_npe, throw_npe_res};

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewArray(env: *mut JNIEnv, obj: jobject, currClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayLength(env: *mut JNIEnv, arr: jobject) -> jint {
    let jvm = get_state(env);
    match get_array(env, arr) {
        Ok(jv) => jv.unwrap_object_nonnull().unwrap_array().len() as i32,
        Err(WasException { exception_obj }) => {
            todo!();
            -1 as i32
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
            let len = nonnull.unwrap_array().len() as i32;
            if index < 0 || index >= len {
                return throw_array_out_of_bounds(jvm, int_state, throw, index);
            }
            let java_value = nonnull.unwrap_array().get_i(index);
            let owned = match java_value_to_boxed_object(jvm, int_state, java_value.as_njv()) {
                Ok(boxed) => boxed.map(|boxed|AllocatedHandle::NormalObject(boxed)),
                Err(WasException { exception_obj }) => {
                    todo!();
                    None
                }
            };
            new_local_ref_public_new(
                owned.as_ref().map(|boxed|boxed.as_allocated_obj()),
                int_state,
            )
        }
        Err(WasException { exception_obj }) => {
            todo!();
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, wCode: jint) -> jvalue {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, val: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, v: jvalue, vCode: ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewArray(env: *mut JNIEnv, eltClass: jclass, length: jint) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let class_rc = assert_inited_or_initing_class(jvm, CPDType::class());
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
unsafe extern "system" fn JVM_ArrayCopy(env: *mut JNIEnv, ignored: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let array_elem_type = MemoryRegions::find_object_region_header(NonNull::new(src).unwrap().cast()).array_elem_type.unwrap();
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_elem_type);
    let elem_size = array_layout.elem_size().get() as i32;
    let src_len = array_layout.calculate_len_address(NonNull::new(src).unwrap().cast()).as_ptr().read();
    let dest_len = array_layout.calculate_len_address(NonNull::new(dst).unwrap().cast()).as_ptr().read();
    if src_pos < 0 || dst_pos < 0 || length < 0 || src_pos + length > src_len as i32 || dst_pos + length > dest_len as i32 {
        dbg!(src_pos);
        dbg!(dst_pos);
        dbg!(length);
        dbg!(src_len);
        dbg!(dest_len);
        dbg!(src_pos < 0);
        dbg!(dst_pos < 0);
        dbg!(length < 0);
        dbg!(src_pos + length > src_len as i32);
        dbg!(dst_pos + length > dest_len as i32);
        int_state.debug_print_stack_trace(jvm);
        todo!("maybe easy to debug?")
    }

    let dst = NonNull::new(dst.cast()).unwrap();
    let src = NonNull::new(src.cast()).unwrap();

    let dst_raw = array_layout.calculate_index_address(dst,dst_pos).inner();
    let src_raw = array_layout.calculate_index_address(src,src_pos).inner();

    libc::memmove(dst_raw.as_ptr(),
                  src_raw.as_ptr(), (length * elem_size) as size_t);

    //slow:
    /*let src_o = from_object_new(jvm, src);
    let src = match src_o {
        Some(x) => NewJavaValueHandle::Object(x),
        None => return throw_npe(jvm, int_state),
    };
    let nonnull = src.unwrap_object_nonnull();
    let src = nonnull.unwrap_array();
    let mut dest_o = from_object_new(jvm, dst);
    let new_jv_handle = match dest_o {
        Some(x) => NewJavaValueHandle::Object(x),
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let nonnull = new_jv_handle.unwrap_object_nonnull();
    let dest = nonnull.unwrap_array();
    if src_pos < 0 || dst_pos < 0 || length < 0 || src_pos + length > src.len() as i32 || dst_pos + length > dest.len() as i32 {
        unimplemented!()
    }
    let mut to_copy = vec![];
    for i in 0..length {
        let temp = src.get_i((src_pos + i));
        to_copy.push(temp);
    }
    for i in 0..length {
        dest.set_i((dst_pos + i), to_copy[i as usize].as_njv());
    }*/
}