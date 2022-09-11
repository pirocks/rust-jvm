use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use jvmti_jni_bindings::{JNIEnv, JNINativeInterface_};
use crate::better_java_stack::native_frame::NativeFrame;
use crate::{JVMState, WasException};
use crate::rust_jni::jni_interface::{alloc_object, define_class, exception_describe, fatal_error, from_reflected_field, from_reflected_method, get_string_chars, get_version, monitor_enter, monitor_exit, throw_new, to_reflected_field, to_reflected_method};
use crate::rust_jni::jni_interface::array::{get_array_length, get_boolean_array_elements, get_byte_array_elements, get_char_array_elements, get_double_array_elements, get_float_array_elements, get_int_array_elements, get_long_array_elements, get_object_array_element, get_primitive_array_critical, get_short_array_elements, release_boolean_array_elements, release_byte_array_elements, release_char_array_elements, release_double_array_elements, release_float_array_elements, release_int_array_elements, release_long_array_elements, release_primitive_array_critical, release_short_array_elements, set_object_array_element};
use crate::rust_jni::jni_interface::array::array_region::{get_boolean_array_region, get_byte_array_region, get_char_array_region, get_double_array_region, get_float_array_region, get_int_array_region, get_long_array_region, get_short_array_region, set_boolean_array_region, set_byte_array_region, set_char_array_region, set_double_array_region, set_float_array_region, set_int_array_region, set_long_array_region, set_short_array_region};
use crate::rust_jni::jni_interface::array::new::{new_boolean_array, new_byte_array, new_char_array, new_double_array, new_float_array, new_int_array, new_long_array, new_object_array, new_short_array};
use crate::rust_jni::jni_interface::call::call_nonstatic::{call_boolean_method, call_boolean_method_a, call_boolean_method_v, call_byte_method, call_byte_method_a, call_byte_method_v, call_char_method, call_char_method_a, call_char_method_v, call_double_method, call_double_method_a, call_double_method_v, call_float_method, call_float_method_a, call_float_method_v, call_int_method, call_int_method_a, call_int_method_v, call_long_method, call_long_method_a, call_long_method_v, call_object_method, call_object_method_a, call_object_method_v, call_short_method, call_short_method_a, call_short_method_v, call_void_method, call_void_method_a, call_void_method_v};
use crate::rust_jni::jni_interface::call::call_nonvirtual::{call_nonvirtual_boolean_method, call_nonvirtual_boolean_method_a, call_nonvirtual_boolean_method_v, call_nonvirtual_byte_method, call_nonvirtual_byte_method_a, call_nonvirtual_byte_method_v, call_nonvirtual_char_method, call_nonvirtual_char_method_a, call_nonvirtual_char_method_v, call_nonvirtual_double_method, call_nonvirtual_double_method_a, call_nonvirtual_double_method_v, call_nonvirtual_float_method, call_nonvirtual_float_method_a, call_nonvirtual_float_method_v, call_nonvirtual_int_method, call_nonvirtual_int_method_a, call_nonvirtual_int_method_v, call_nonvirtual_long_method, call_nonvirtual_long_method_a, call_nonvirtual_long_method_v, call_nonvirtual_object_method, call_nonvirtual_object_method_a, call_nonvirtual_object_method_v, call_nonvirtual_short_method, call_nonvirtual_short_method_a, call_nonvirtual_short_method_v, call_nonvirtual_void_method, call_nonvirtual_void_method_a, call_nonvirtual_void_method_v};
use crate::rust_jni::jni_interface::call::call_static::{call_static_boolean_method, call_static_boolean_method_a, call_static_boolean_method_v, call_static_byte_method, call_static_byte_method_a, call_static_byte_method_v, call_static_char_method, call_static_char_method_a, call_static_char_method_v, call_static_double_method, call_static_double_method_a, call_static_double_method_v, call_static_float_method, call_static_float_method_a, call_static_float_method_v, call_static_int_method, call_static_int_method_a, call_static_int_method_v, call_static_long_method, call_static_long_method_a, call_static_long_method_v, call_static_object_method, call_static_object_method_a, call_static_object_method_v, call_static_short_method, call_static_short_method_a, call_static_short_method_v, call_static_void_method, call_static_void_method_a, call_static_void_method_v};
use crate::rust_jni::jni_interface::exception::{exception_check, exception_clear, exception_occured, throw};
use crate::rust_jni::jni_interface::get_field::{get_boolean_field, get_byte_field, get_char_field, get_double_field, get_field_id, get_float_field, get_int_field, get_long_field, get_object_field, get_short_field, get_static_boolean_field, get_static_byte_field, get_static_char_field, get_static_double_field, get_static_field_id, get_static_float_field, get_static_int_field, get_static_long_field, get_static_method_id, get_static_object_field, get_static_short_field};
use crate::rust_jni::jni_interface::global_ref::{delete_global_ref, delete_weak_global_ref, new_global_ref, new_weak_global_ref};
use crate::rust_jni::jni_interface::instance_of::is_instance_of;
use crate::rust_jni::jni_interface::local_frame::{delete_local_ref, new_local_ref, pop_local_frame, push_local_frame};
use crate::rust_jni::jni_interface::method::get_method_id;
use crate::rust_jni::jni_interface::misc::{ensure_local_capacity, find_class, get_java_vm, get_superclass, is_assignable_from, is_same_object, register_natives, unregister_natives};
use crate::rust_jni::jni_interface::new_object::{jni_new_object, new_object_a, new_object_v};
use crate::rust_jni::jni_interface::set_field::{set_boolean_field, set_byte_field, set_char_field, set_double_field, set_float_field, set_int_field, set_long_field, set_object_field, set_short_field, set_static_boolean_field, set_static_byte_field, set_static_char_field, set_static_double_field, set_static_float_field, set_static_int_field, set_static_long_field, set_static_object_field, set_static_short_field};
use crate::rust_jni::jni_interface::string::{get_string_region, get_string_utfchars, get_string_utflength, get_string_utfregion, new_string, new_string_utf, release_string_chars, release_string_utfchars};
use crate::rust_jni::native_util::get_object_class;

pub fn initial_jni_interface() -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: null_mut(),
        reserved1: null_mut(),
        reserved2: null_mut(),
        reserved3: null_mut(),
        GetVersion: Some(get_version),
        DefineClass: Some(define_class),
        FindClass: Some(find_class),
        FromReflectedMethod: Some(from_reflected_method),
        FromReflectedField: Some(from_reflected_field),
        ToReflectedMethod: Some(to_reflected_method),
        GetSuperclass: Some(get_superclass),
        IsAssignableFrom: Some(is_assignable_from),
        ToReflectedField: Some(to_reflected_field),
        Throw: Some(throw),
        ThrowNew: Some(throw_new),
        ExceptionOccurred: Some(exception_occured),
        ExceptionDescribe: Some(exception_describe),
        ExceptionClear: Some(exception_clear),
        FatalError: Some(fatal_error),
        PushLocalFrame: Some(push_local_frame),
        PopLocalFrame: Some(pop_local_frame),
        NewGlobalRef: Some(new_global_ref),
        DeleteGlobalRef: Some(delete_global_ref),
        DeleteLocalRef: Some(delete_local_ref),
        IsSameObject: Some(is_same_object),
        NewLocalRef: Some(new_local_ref),
        EnsureLocalCapacity: Some(ensure_local_capacity),
        AllocObject: Some(alloc_object),
        NewObject: Some(jni_new_object),
        NewObjectV: Some(unsafe { transmute(new_object_v as *mut c_void) }),
        NewObjectA: Some(new_object_a),
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
        CallNonvirtualObjectMethod: Some(call_nonvirtual_object_method),
        CallNonvirtualObjectMethodV: Some(unsafe { transmute(call_nonvirtual_object_method_v as *mut c_void) }),
        CallNonvirtualObjectMethodA: Some(call_nonvirtual_object_method_a),
        CallNonvirtualBooleanMethod: Some(call_nonvirtual_boolean_method),
        CallNonvirtualBooleanMethodV: Some(unsafe { transmute(call_nonvirtual_boolean_method_v as *mut c_void) }),
        CallNonvirtualBooleanMethodA: Some(call_nonvirtual_boolean_method_a),
        CallNonvirtualByteMethod: Some(call_nonvirtual_byte_method),
        CallNonvirtualByteMethodV: Some(unsafe { transmute(call_nonvirtual_byte_method_v as *mut c_void) }),
        CallNonvirtualByteMethodA: Some(call_nonvirtual_byte_method_a),
        CallNonvirtualCharMethod: Some(call_nonvirtual_char_method),
        CallNonvirtualCharMethodV: Some(unsafe { transmute(call_nonvirtual_char_method_v as *mut c_void) }),
        CallNonvirtualCharMethodA: Some(call_nonvirtual_char_method_a),
        CallNonvirtualShortMethod: Some(call_nonvirtual_short_method),
        CallNonvirtualShortMethodV: Some(unsafe { transmute(call_nonvirtual_short_method_v as *mut c_void) }),
        CallNonvirtualShortMethodA: Some(call_nonvirtual_short_method_a),
        CallNonvirtualIntMethod: Some(call_nonvirtual_int_method),
        CallNonvirtualIntMethodV: Some(unsafe { transmute(call_nonvirtual_int_method_v as *mut c_void) }),
        CallNonvirtualIntMethodA: Some(call_nonvirtual_int_method_a),
        CallNonvirtualLongMethod: Some(call_nonvirtual_long_method),
        CallNonvirtualLongMethodV: Some(unsafe { transmute(call_nonvirtual_long_method_v as *mut c_void) }),
        CallNonvirtualLongMethodA: Some(call_nonvirtual_long_method_a),
        CallNonvirtualFloatMethod: Some(call_nonvirtual_float_method),
        CallNonvirtualFloatMethodV: Some(unsafe { transmute(call_nonvirtual_float_method_v as *mut c_void) }),
        CallNonvirtualFloatMethodA: Some(call_nonvirtual_float_method_a),
        CallNonvirtualDoubleMethod: Some(call_nonvirtual_double_method),
        CallNonvirtualDoubleMethodV: Some(unsafe { transmute(call_nonvirtual_double_method_v as *mut c_void) }),
        CallNonvirtualDoubleMethodA: Some(call_nonvirtual_double_method_a),
        CallNonvirtualVoidMethod: Some(call_nonvirtual_void_method),
        CallNonvirtualVoidMethodV: Some(unsafe { transmute(call_nonvirtual_void_method_v as *mut c_void) }),
        CallNonvirtualVoidMethodA: Some(call_nonvirtual_void_method_a),
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
        GetStaticObjectField: Some(get_static_object_field),
        GetStaticBooleanField: Some(get_static_boolean_field),
        GetStaticByteField: Some(get_static_byte_field),
        GetStaticCharField: Some(get_static_char_field),
        GetStaticShortField: Some(get_static_short_field),
        GetStaticIntField: Some(get_static_int_field),
        GetStaticLongField: Some(get_static_long_field),
        GetStaticFloatField: Some(get_static_float_field),
        GetStaticDoubleField: Some(get_static_double_field),
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
        GetStringChars: Some(get_string_chars),
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
        GetBooleanArrayElements: Some(get_boolean_array_elements),
        GetByteArrayElements: Some(get_byte_array_elements),
        GetCharArrayElements: Some(get_char_array_elements),
        GetShortArrayElements: Some(get_short_array_elements),
        GetIntArrayElements: Some(get_int_array_elements),
        GetLongArrayElements: Some(get_long_array_elements),
        GetFloatArrayElements: Some(get_float_array_elements),
        GetDoubleArrayElements: Some(get_double_array_elements),
        ReleaseBooleanArrayElements: Some(release_boolean_array_elements),
        ReleaseByteArrayElements: Some(release_byte_array_elements),
        ReleaseCharArrayElements: Some(release_char_array_elements),
        ReleaseShortArrayElements: Some(release_short_array_elements),
        ReleaseIntArrayElements: Some(release_int_array_elements),
        ReleaseLongArrayElements: Some(release_long_array_elements),
        ReleaseFloatArrayElements: Some(release_float_array_elements),
        ReleaseDoubleArrayElements: Some(release_double_array_elements),
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
        UnregisterNatives: Some(unregister_natives),
        MonitorEnter: Some(monitor_enter),
        MonitorExit: Some(monitor_exit),
        GetJavaVM: Some(get_java_vm),
        GetStringRegion: Some(get_string_region),
        GetStringUTFRegion: Some(get_string_utfregion),
        GetPrimitiveArrayCritical: Some(get_primitive_array_critical),
        ReleasePrimitiveArrayCritical: Some(release_primitive_array_critical),
        GetStringCritical: None,     //todo
        ReleaseStringCritical: None, //todo
        NewWeakGlobalRef: Some(new_weak_global_ref),
        DeleteWeakGlobalRef: Some(delete_weak_global_ref),
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: None,     //todo
        GetDirectBufferAddress: None,  //todo
        GetDirectBufferCapacity: None, //todo
        GetObjectRefType: None,        //todo
    }
}

pub fn with_jni_interface<'gc, 'l, T>(jvm: &'gc JVMState<'gc>, int_state: &mut NativeFrame<'gc, 'l>, was_exception: &mut Option<WasException<'gc>>, with_interface: impl FnOnce(*mut *const JNINativeInterface_) -> T) -> T {
    let jvm_ptr = jvm as *const JVMState<'gc> as *const c_void as *mut c_void; //todo this is mut/const thing is annoying
    let int_state_ptr = int_state as *mut NativeFrame<'gc, 'l> as *mut c_void;
    let exception_pointer = was_exception as *mut Option<WasException<'gc>> as *mut c_void;
    let interface = int_state.stack_jni_interface().jni_inner_mut();
    let reserved0_save = interface.reserved0;
    let reserved1_save = interface.reserved1;
    let reserved2_save = interface.reserved2;
    interface.reserved0 = jvm_ptr;
    interface.reserved1 = int_state_ptr;
    interface.reserved2 = exception_pointer;
    let mut as_ptr = interface as *const JNINativeInterface_;
    let as_ptr2 = (&mut as_ptr) as *mut *const JNINativeInterface_;
    let res = with_interface(as_ptr2);
    interface.reserved0 = reserved0_save;
    interface.reserved1 = reserved1_save;
    interface.reserved2 = reserved2_save;
    res
}

pub unsafe fn get_state<'gc>(env: *mut JNIEnv) -> &'gc JVMState<'gc> {
    &(*((**env).reserved0 as *const JVMState))
}

pub unsafe fn get_interpreter_state<'gc, 'k, 'any>(env: *mut JNIEnv) -> &'any mut NativeFrame<'gc, 'k> {
    (**env).reserved1.cast::<NativeFrame<'gc, 'k>>().as_mut().unwrap()
}

pub unsafe fn get_throw<'any, 'gc>(env: *mut JNIEnv) -> &'any mut Option<WasException<'gc>> {
    (**env).reserved2.cast::<Option<WasException<'gc>>>().as_mut().unwrap()
}
