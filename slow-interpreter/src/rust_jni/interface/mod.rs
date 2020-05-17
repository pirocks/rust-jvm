use jvmti_jni_bindings::{JNINativeInterface_, JNIEnv, jobject,  jboolean, JNI_FALSE, JNI_TRUE};
use std::mem::transmute;
use std::ffi::c_void;
use crate::rust_jni::{exception_check, register_natives, release_string_utfchars, get_method_id};
use crate::rust_jni::native_util::{get_object_class, from_object};
use crate::rust_jni::interface::string::*;
use crate::rust_jni::interface::misc::*;
use crate::rust_jni::interface::get_field::*;
use crate::rust_jni::interface::set_field::*;
use crate::rust_jni::interface::exception::*;
use crate::rust_jni::interface::global_ref::*;
use crate::rust_jni::interface::array::*;
use crate::JVMState;
use std::cell::RefCell;
use crate::rust_jni::interface::local_frame::{pop_local_frame, push_local_frame};
use std::sync::Arc;
use crate::rust_jni::interface::local_ref::new_local_ref;
use crate::rust_jni::interface::instance_of::is_instance_of;
use crate::rust_jni::interface::array::array_region::*;
use crate::rust_jni::interface::array::new::*;
use crate::rust_jni::interface::call::call_static::*;
use crate::rust_jni::interface::call::call_nonstatic::*;

//todo this should be in state impl
thread_local! {
    static JNI_INTERFACE: RefCell<Option<JNINativeInterface_>> = RefCell::new(None);
}

//GetFieldID
pub fn get_interface(state: &JVMState) -> *const JNINativeInterface_ {
    JNI_INTERFACE.with(|refcell| {
        {
            let first_borrow = refcell.borrow();
            match first_borrow.as_ref() {
                None => {}
                Some(interface) => {
                    return interface as *const JNINativeInterface_;
                }
            }
        }
        let new = get_interface_impl(state);
        refcell.replace(new.into());
        let new_borrow = refcell.borrow();
        new_borrow.as_ref().unwrap() as *const JNINativeInterface_
    })
}

fn get_interface_impl(state: &JVMState) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: std::ptr::null_mut(),
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
        CallObjectMethodV: Some(unsafe{ transmute(call_object_method_v as *mut c_void)}),
        CallObjectMethodA: Some(call_object_method_a),
        CallBooleanMethod: Some(call_boolean_method),
        CallBooleanMethodV: Some(unsafe{ transmute(call_boolean_method_v as *mut c_void)}),
        CallBooleanMethodA: Some(call_boolean_method_a),
        CallByteMethod: Some(call_byte_method),
        CallByteMethodV: Some(unsafe{ transmute(call_byte_method_v as *mut c_void)}),
        CallByteMethodA: Some(call_byte_method_a),
        CallCharMethod: Some(call_char_method),
        CallCharMethodV: Some(unsafe{ transmute(call_char_method_v as *mut c_void)}),
        CallCharMethodA: Some(call_char_method_a),
        CallShortMethod: Some(call_short_method),
        CallShortMethodV: Some(unsafe{ transmute(call_short_method_v as *mut c_void)}),
        CallShortMethodA: Some(call_short_method_a),
        CallIntMethod: Some(call_int_method),
        CallIntMethodV: Some(unsafe{ transmute(call_int_method_v as *mut c_void)}),
        CallIntMethodA: Some(call_int_method_a),
        CallLongMethod: Some(call_long_method),
        CallLongMethodV: Some(unsafe{ transmute(call_long_method_v as *mut c_void)}),
        CallLongMethodA: Some(call_long_method_a),
        CallFloatMethod: Some(call_float_method),
        CallFloatMethodV: Some(unsafe{ transmute(call_float_method_v as *mut c_void)}),
        CallFloatMethodA: Some(call_float_method_a),
        CallDoubleMethod: Some(call_double_method),
        CallDoubleMethodV: Some(unsafe{ transmute(call_double_method_v as *mut c_void)}),
        CallDoubleMethodA: Some(call_double_method_a),
        CallVoidMethod: Some(call_void_method),
        CallVoidMethodV: Some(unsafe{ transmute(call_void_method_v as *mut c_void)}),
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
        GetBooleanField: None,
        GetByteField: None,
        GetCharField: None,
        GetShortField: None,
        GetIntField: Some(get_int_field),
        GetLongField: Some(get_long_field),
        GetFloatField: None,
        GetDoubleField: None,
        SetObjectField: None,
        SetBooleanField: Some(set_boolean_field),
        SetByteField: None,
        SetCharField: None,
        SetShortField: None,
        SetIntField: Some(set_int_field),
        SetLongField: Some(set_long_field),
        SetFloatField: None,
        SetDoubleField: None,
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
        SetStaticBooleanField: None,
        SetStaticByteField: None,
        SetStaticCharField: None,
        SetStaticShortField: None,
        SetStaticIntField: None,
        SetStaticLongField: None,
        SetStaticFloatField: None,
        SetStaticDoubleField: None,
        NewString: Some(new_string),
        GetStringLength: Some(get_string_utflength),
        GetStringChars: None,
        ReleaseStringChars: Some(release_string_chars),
        NewStringUTF: Some(new_string_utf),
        GetStringUTFLength: Some(get_string_utflength),
        GetStringUTFChars: Some(get_string_utfchars),
        ReleaseStringUTFChars: Some(release_string_utfchars),
        GetArrayLength: Some(get_array_length),
        NewObjectArray: None,
        GetObjectArrayElement: None,
        SetObjectArrayElement: None,
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
        GetBooleanArrayRegion: None,
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
        DeleteWeakGlobalRef: None,
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

pub mod instance_of {
    use jvmti_jni_bindings::{JNIEnv, jobject, jclass, jboolean};
    use crate::rust_jni::native_util::{get_state, from_object, get_frame};
    use crate::instructions::special::instance_of_impl;
    use std::ops::Deref;
    use crate::java_values::JavaValue;

    pub unsafe extern "C" fn is_instance_of(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean {
        let jvm = get_state(env);
        let java_obj = from_object(obj);
        let class_object = from_object(clazz);
        let type_view = JavaValue::Object(class_object).cast_class().as_type();
        let type_ = match type_view.try_unwrap_ref_type(){
            None => unimplemented!(),
            Some(ref_type) => ref_type,
        };
        let frame = get_frame(env);
        instance_of_impl(jvm, frame.deref(), java_obj.unwrap(), type_.clone());
        (frame.pop().unwrap_int() != 0) as jboolean
    }
}

pub mod local_ref {
    use jvmti_jni_bindings::{JNIEnv, jobject};
    use crate::rust_jni::native_util::from_object;

    pub unsafe extern "C" fn new_local_ref(_env: *mut JNIEnv, ref_: jobject) -> jobject {
        //todo blocking on actually having gc
        std::mem::forget(from_object(ref_).unwrap());
        ref_
    }
}

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