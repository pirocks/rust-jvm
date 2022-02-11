use std::borrow::Borrow;
use std::ops::Deref;
use std::panic::panic_any;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jclass, jint, jintArray, JNIEnv, jobject, jvalue};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::names::CClassName;
use slow_interpreter::instructions::new::a_new_array_from_name;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::java::lang::boolean::Boolean;
use slow_interpreter::java::lang::byte::Byte;
use slow_interpreter::java::lang::char::Char;
use slow_interpreter::java::lang::double::Double;
use slow_interpreter::java::lang::float::Float;
use slow_interpreter::java::lang::int::Int;
use slow_interpreter::java::lang::long::Long;
use slow_interpreter::java::lang::short::Short;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{java_value_to_boxed_object, throw_array_out_of_bounds, throw_array_out_of_bounds_res, throw_illegal_arg_res, throw_npe, throw_npe_res};

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewArray(env: *mut JNIEnv, obj: jobject, currClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayLength(env: *mut JNIEnv, arr: jobject) -> jint {
    match get_array(env, arr) {
        Ok(jv) => jv.unwrap_array().len() as i32,
        Err(WasException {}) => -1 as i32,
    }
}

unsafe fn get_array<'gc_life>(env: *mut JNIEnv, arr: jobject) -> Result<JavaValue<'gc_life>, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match from_object(jvm, arr) {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(possibly_arr) => {
            match possibly_arr.deref() {
                Object::Array(_) => {
                    Ok(JavaValue::Object(todo!() /*from_jclass(jvm,arr)*/))
                }
                Object::Object(obj) => {
                    return throw_illegal_arg_res(jvm, int_state);
                }
            }
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match get_array(env, arr) {
        Ok(jv) => {
            let len = jv.unwrap_array().len() as i32;
            if index < 0 || index >= len {
                return throw_array_out_of_bounds(jvm, int_state, index);
            }
            let java_value = jv.unwrap_array().get_i(jvm, index);
            new_local_ref_public(
                match java_value_to_boxed_object(jvm, int_state, java_value) {
                    Ok(boxed) => todo!()/*boxed*/,
                    Err(WasException {}) => None,
                },
                int_state,
            )
        }
        Err(WasException {}) => null_mut(),
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
    let array_type_name = from_jclass(jvm, eltClass).as_runtime_class(jvm).cpdtype();
    a_new_array_from_name(jvm, int_state, length, array_type_name);
    new_local_ref_public(int_state.pop_current_operand_stack(Some(CClassName::object().into())).unwrap_object(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewMultiArray(env: *mut JNIEnv, eltClass: jclass, dim: jintArray) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ArrayCopy(env: *mut JNIEnv, ignored: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let src_o = from_object_new(jvm, src);
    let src = match src_o {
        Some(x) => NewJavaValueHandle::Object(x),
        None => return throw_npe(jvm, int_state),
    };
    let src = src.unwrap_array(jvm);
    let mut dest_o = from_object_new(jvm, dst);
    let new_jv_handle = match dest_o {
        Some(x) => NewJavaValueHandle::Object(x),
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let dest = new_jv_handle.unwrap_array(jvm);
    if src_pos < 0 || dst_pos < 0 || length < 0 || src_pos + length > src.len() as i32 || dst_pos + length > dest.len() as i32 {
        unimplemented!()
    }
    let mut to_copy = vec![];
    for i in 0..(length) {
        let temp = src.get_i( ((src_pos + i) as usize));
        to_copy.push(temp);
    }
    for i in 0..(length) {
        dest.set_i((dst_pos + i) as usize, to_copy[i as usize].as_njv());
    }
}