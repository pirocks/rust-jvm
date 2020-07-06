use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::Arc;

use jvmti_jni_bindings::{jboolean, JNI_FALSE, JNI_TRUE, JNIEnv, JNINativeInterface_, jobject};

use crate::{InterpreterStateGuard, JVMState};
use crate::rust_jni::{exception_check, get_method_id, register_natives, release_string_utfchars};
use crate::rust_jni::interface::array::*;
use crate::rust_jni::interface::array::array_region::*;
use crate::rust_jni::interface::array::new::*;
use crate::rust_jni::interface::call::call_nonstatic::*;
use crate::rust_jni::interface::call::call_static::*;
use crate::rust_jni::interface::exception::*;
use crate::rust_jni::interface::get_field::*;
use crate::rust_jni::interface::global_ref::*;
use crate::rust_jni::interface::instance_of::is_instance_of;
use crate::rust_jni::interface::local_frame::{delete_local_ref, new_local_ref, pop_local_frame, push_local_frame};
use crate::rust_jni::interface::misc::*;
use crate::rust_jni::interface::set_field::*;
use crate::rust_jni::interface::string::*;
use crate::rust_jni::native_util::{from_object, get_object_class};

//todo this should be in state impl
thread_local! {
    static JNI_INTERFACE: RefCell<*mut *const JNINativeInterface_> = RefCell::new(null_mut());
}

//GetFieldID
pub fn get_interface(state: &'static JVMState, int_state: &mut InterpreterStateGuard) -> *mut *const JNINativeInterface_ {
    JNI_INTERFACE.with(|refcell| {
        unsafe {
            let first_borrow = refcell.borrow_mut();
            match first_borrow.as_mut() {
                None => {}
                Some(interface) => {
                    (*((*interface) as *mut JNINativeInterface_)).reserved1 = transmute(int_state);//todo technically this is wrong, see "JNI Interface Functions and Pointers" in jni spec
                    return interface as *mut *const JNINativeInterface_;
                }
            }
        }
        let new = get_interface_impl(state, int_state);
        let jni_data_structure_ptr = Box::leak(box new) as *const JNINativeInterface_;
        refcell.replace(Box::leak(box (jni_data_structure_ptr)) as *mut *const JNINativeInterface_);//todo leak
        let new_borrow = refcell.borrow();
        *new_borrow as *mut *const JNINativeInterface_
    })
}

fn get_interface_impl(state: &'static JVMState, int_state: &mut InterpreterStateGuard) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: unsafe { transmute(int_state) },
        reserved2: std::ptr::null_mut(),
        reserved3: std::ptr::null_mut(),
        GetVersion: None,
        DefineClass: None,
        FindClass: Some(find_class),
        FromReflectedMethod: None,
        FromReflectedField: None,
        ToReflectedMethod: None,
        GetSuperclass: Some(get_superclass),
        IsAssignableFrom: Some(is_assignable_from),
        ToReflectedField: None,
        Throw: Some(throw),
        ThrowNew: None,
        ExceptionOccurred: Some(exception_occured),
        ExceptionDescribe: None,
        ExceptionClear: Some(exception_clear),
        FatalError: None,
        PushLocalFrame: Some(push_local_frame),
        PopLocalFrame: Some(pop_local_frame),
        NewGlobalRef: Some(new_global_ref),
        DeleteGlobalRef: Some(delete_global_ref),
        DeleteLocalRef: Some(delete_local_ref),
        IsSameObject: Some(is_same_object),
        NewLocalRef: Some(new_local_ref),
        EnsureLocalCapacity: Some(ensure_local_capacity),
        AllocObject: None,
        NewObject: Some(unsafe { transmute(new_object as *mut c_void) }),
        NewObjectV: Some(unsafe { transmute(new_object_v as *mut c_void) }),
        NewObjectA: None,
        GetObjectClass: Some(get_object_class),
        IsInstanceOf: Some(is_instance_of),
        GetMethodID: Some(get_method_id),
        CallObjectMethod: Some(call_object_method),
        CallObjectMethodV: Some(unsafe { transmute(call_object_method_v as *mut c_void) }),
        CallObjectMethodA: Some(call_object_method_a),
        CallBooleanMethod: Some(call_boolean_method),
        CallBooleanMethodV: Some(unsafe { transmute(call_boolean_method_v as *mut c_void) }),
        CallBooleanMethodA: Some(call_boolean_method_a),
        CallByteMethod: Some(call_byte_method),
        CallByteMethodV: Some(unsafe { transmute(call_byte_method_v as *mut c_void) }),
        CallByteMethodA: Some(call_byte_method_a),
        CallCharMethod: Some(call_char_method),
        CallCharMethodV: Some(unsafe { transmute(call_char_method_v as *mut c_void) }),
        CallCharMethodA: Some(call_char_method_a),
        CallShortMethod: Some(call_short_method),
        CallShortMethodV: Some(unsafe { transmute(call_short_method_v as *mut c_void) }),
        CallShortMethodA: Some(call_short_method_a),
        CallIntMethod: Some(call_int_method),
        CallIntMethodV: Some(unsafe { transmute(call_int_method_v as *mut c_void) }),
        CallIntMethodA: Some(call_int_method_a),
        CallLongMethod: Some(call_long_method),
        CallLongMethodV: Some(unsafe { transmute(call_long_method_v as *mut c_void) }),
        CallLongMethodA: Some(call_long_method_a),
        CallFloatMethod: Some(call_float_method),
        CallFloatMethodV: Some(unsafe { transmute(call_float_method_v as *mut c_void) }),
        CallFloatMethodA: Some(call_float_method_a),
        CallDoubleMethod: Some(call_double_method),
        CallDoubleMethodV: Some(unsafe { transmute(call_double_method_v as *mut c_void) }),
        CallDoubleMethodA: Some(call_double_method_a),
        CallVoidMethod: Some(call_void_method),
        CallVoidMethodV: Some(unsafe { transmute(call_void_method_v as *mut c_void) }),
        CallVoidMethodA: Some(call_void_method_a),
        CallNonvirtualObjectMethod: None,
        CallNonvirtualObjectMethodV: None,
        CallNonvirtualObjectMethodA: None,
        CallNonvirtualBooleanMethod: None,
        CallNonvirtualBooleanMethodV: None,
        CallNonvirtualBooleanMethodA: None,
        CallNonvirtualByteMethod: None,
        CallNonvirtualByteMethodV: None,
        CallNonvirtualByteMethodA: None,
        CallNonvirtualCharMethod: None,
        CallNonvirtualCharMethodV: None,
        CallNonvirtualCharMethodA: None,
        CallNonvirtualShortMethod: None,
        CallNonvirtualShortMethodV: None,
        CallNonvirtualShortMethodA: None,
        CallNonvirtualIntMethod: None,
        CallNonvirtualIntMethodV: None,
        CallNonvirtualIntMethodA: None,
        CallNonvirtualLongMethod: None,
        CallNonvirtualLongMethodV: None,
        CallNonvirtualLongMethodA: None,
        CallNonvirtualFloatMethod: None,
        CallNonvirtualFloatMethodV: None,
        CallNonvirtualFloatMethodA: None,
        CallNonvirtualDoubleMethod: None,
        CallNonvirtualDoubleMethodV: None,
        CallNonvirtualDoubleMethodA: None,
        CallNonvirtualVoidMethod: None,
        CallNonvirtualVoidMethodV: None,
        CallNonvirtualVoidMethodA: None,
        GetFieldID: Some(get_field_id),
        GetObjectField: Some(get_object_field),
        GetBooleanField: Some(get_boolean_field),
        GetByteField: Some(get_byte_field),
        GetCharField: Some(get_char_field),
        GetShortField: Some(get_short_field),
        GetIntField: Some(get_int_field),
        GetLongField: Some(get_long_field),
        GetFloatField: Some(get_float_field),
        GetDoubleField: Some(get_double_field),
        SetObjectField: Some(set_object_field),
        SetBooleanField: Some(set_boolean_field),
        SetByteField: Some(set_byte_field),
        SetCharField: Some(set_char_field),
        SetShortField: Some(set_short_field),
        SetIntField: Some(set_int_field),
        SetLongField: Some(set_long_field),
        SetFloatField: Some(set_float_field),
        SetDoubleField: Some(set_double_field),
        GetStaticMethodID: Some(get_static_method_id),
        CallStaticObjectMethod: Some(call_static_object_method),
        CallStaticObjectMethodV: Some(unsafe { transmute(call_static_object_method_v as *mut c_void) }),
        CallStaticObjectMethodA: Some(call_static_object_method_a),
        CallStaticBooleanMethod: Some(call_static_boolean_method),
        CallStaticBooleanMethodV: Some(unsafe { transmute(call_static_boolean_method_v as *mut c_void) }),
        CallStaticBooleanMethodA: Some(call_static_boolean_method_a),
        CallStaticByteMethod: Some(call_static_byte_method),
        CallStaticByteMethodV: Some(unsafe { transmute(call_static_byte_method_v as *mut c_void) }),
        CallStaticByteMethodA: Some(call_static_byte_method_a),
        CallStaticCharMethod: Some(call_static_char_method),
        CallStaticCharMethodV: Some(unsafe { transmute(call_static_char_method_v as *mut c_void) }),
        CallStaticCharMethodA: Some(call_static_char_method_a),
        CallStaticShortMethod: Some(call_static_short_method),
        CallStaticShortMethodV: Some(unsafe { transmute(call_static_short_method_v as *mut c_void) }),
        CallStaticShortMethodA: Some(call_static_short_method_a),
        CallStaticIntMethod: Some(call_static_int_method),
        CallStaticIntMethodV: Some(unsafe { transmute(call_static_int_method_v as *mut c_void) }),
        CallStaticIntMethodA: Some(call_static_int_method_a),
        CallStaticLongMethod: Some(call_static_long_method),
        CallStaticLongMethodV: Some(unsafe { transmute(call_static_long_method_v as *mut c_void) }),
        CallStaticLongMethodA: Some(call_static_long_method_a),
        CallStaticFloatMethod: Some(call_static_float_method),
        CallStaticFloatMethodV: Some(unsafe { transmute(call_static_float_method_v as *mut c_void) }),
        CallStaticFloatMethodA: Some(call_static_float_method_a),
        CallStaticDoubleMethod: Some(call_static_double_method),
        CallStaticDoubleMethodV: Some(unsafe { transmute(call_static_double_method_v as *mut c_void) }),
        CallStaticDoubleMethodA: Some(call_static_double_method_a),
        CallStaticVoidMethod: Some(call_static_void_method),
        CallStaticVoidMethodV: Some(unsafe { transmute(call_static_void_method_v as *mut c_void) }),
        CallStaticVoidMethodA: Some(call_static_void_method_a),
        GetStaticFieldID: Some(get_static_field_id),
        GetStaticObjectField: None,
        GetStaticBooleanField: None,
        GetStaticByteField: None,
        GetStaticCharField: None,
        GetStaticShortField: None,
        GetStaticIntField: None,
        GetStaticLongField: None,
        GetStaticFloatField: None,
        GetStaticDoubleField: None,
        SetStaticObjectField: Some(set_static_object_field),
        SetStaticBooleanField: Some(set_static_boolean_field),
        SetStaticByteField: Some(set_static_byte_field),
        SetStaticCharField: Some(set_static_char_field),
        SetStaticShortField: Some(set_static_short_field),
        SetStaticIntField: Some(set_static_int_field),
        SetStaticLongField: Some(set_static_long_field),
        SetStaticFloatField: Some(set_static_float_field),
        SetStaticDoubleField: Some(set_static_double_field),
        NewString: Some(new_string),
        GetStringLength: Some(get_string_utflength),
        GetStringChars: None,
        ReleaseStringChars: Some(release_string_chars),
        NewStringUTF: Some(new_string_utf),
        GetStringUTFLength: Some(get_string_utflength),
        GetStringUTFChars: Some(get_string_utfchars),
        ReleaseStringUTFChars: Some(release_string_utfchars),
        GetArrayLength: Some(get_array_length),
        NewObjectArray: Some(new_object_array),
        GetObjectArrayElement: Some(get_object_array_element),
        SetObjectArrayElement: Some(set_object_array_element),
        NewBooleanArray: Some(new_boolean_array),
        NewByteArray: Some(new_byte_array),
        NewCharArray: Some(new_char_array),
        NewShortArray: Some(new_short_array),
        NewIntArray: Some(new_int_array),
        NewLongArray: Some(new_long_array),
        NewFloatArray: Some(new_float_array),
        NewDoubleArray: Some(new_double_array),
        GetBooleanArrayElements: None,
        GetByteArrayElements: None,
        GetCharArrayElements: None,
        GetShortArrayElements: None,
        GetIntArrayElements: None,
        GetLongArrayElements: None,
        GetFloatArrayElements: None,
        GetDoubleArrayElements: None,
        ReleaseBooleanArrayElements: None,
        ReleaseByteArrayElements: None,
        ReleaseCharArrayElements: None,
        ReleaseShortArrayElements: None,
        ReleaseIntArrayElements: None,
        ReleaseLongArrayElements: None,
        ReleaseFloatArrayElements: None,
        ReleaseDoubleArrayElements: None,
        GetBooleanArrayRegion: Some(get_boolean_array_region),
        GetByteArrayRegion: Some(get_byte_array_region),
        GetCharArrayRegion: Some(get_char_array_region),
        GetShortArrayRegion: Some(get_short_array_region),
        GetIntArrayRegion: Some(get_int_array_region),
        GetLongArrayRegion: Some(get_long_array_region),
        GetFloatArrayRegion: Some(get_float_array_region),
        GetDoubleArrayRegion: Some(get_double_array_region),
        SetBooleanArrayRegion: Some(set_boolean_array_region),
        SetByteArrayRegion: Some(set_byte_array_region),
        SetCharArrayRegion: Some(set_char_array_region),
        SetShortArrayRegion: Some(set_short_array_region),
        SetIntArrayRegion: Some(set_int_array_region),
        SetLongArrayRegion: Some(set_long_array_region),
        SetFloatArrayRegion: Some(set_float_array_region),
        SetDoubleArrayRegion: Some(set_double_array_region),
        RegisterNatives: Some(register_natives),
        UnregisterNatives: None,
        MonitorEnter: None,
        MonitorExit: None,
        GetJavaVM: Some(get_java_vm),
        GetStringRegion: Some(get_string_region),
        GetStringUTFRegion: Some(get_string_utfregion),
        GetPrimitiveArrayCritical: None,
        ReleasePrimitiveArrayCritical: None,
        GetStringCritical: None,
        ReleaseStringCritical: None,
        NewWeakGlobalRef: Some(new_weak_global_ref),
        DeleteWeakGlobalRef: Some(delete_weak_global_ref),
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: None,
        GetDirectBufferAddress: None,
        GetDirectBufferCapacity: None,
        GetObjectRefType: None,
    }
}

pub unsafe extern "C" fn is_same_object(_env: *mut JNIEnv, obj1: jobject, obj2: jobject) -> jboolean {
    let _1 = from_object(obj1);
    let _2 = from_object(obj2);
    (match _1 {
        None => {
            match _2 {
                None => JNI_TRUE,
                Some(_) => JNI_FALSE,
            }
        }
        Some(_1_) => {
            match _2 {
                None => JNI_FALSE,
                Some(_2_) => Arc::ptr_eq(&_1_, &_2_) as u32,
            }
        }
    }) as u8
}

pub mod instance_of;
pub mod local_frame;
pub mod call;
pub mod array;
pub mod global_ref;
pub mod exception;
pub mod util;
pub mod misc;
pub mod set_field;
pub mod string;
pub mod get_field;