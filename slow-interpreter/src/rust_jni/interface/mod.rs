use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::fs::File;
use std::io::{Cursor, Write};
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock};

use by_address::ByAddress;
use itertools::Itertools;
use wtf8::Wtf8Buf;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use classfile_view::view::field_view::FieldView;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jfieldID, jint, jmethodID, JNI_ERR, JNI_OK, JNIEnv, JNINativeInterface_, jobject, jsize, jstring, jvalue};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
use rust_jvm_common::descriptor_parser::parse_field_descriptor;
use rust_jvm_common::loading::{ClassLoadingError, ClassWithLoader, LoaderName};
use verification::{VerifierContext, verify};
use verification::verifier::TypeSafetyError;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{check_initing_or_inited_class, create_class_object, get_field_numbers};
use crate::class_objects::get_or_create_class_object_force_loader;
use crate::field_table::FieldId;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter::WasException;
use crate::interpreter_util::new_object;
use crate::java::lang::class::JClass;
use crate::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::java::lang::reflect::field::Field;
use crate::java::lang::reflect::method::Method;
use crate::java::lang::string::JString;
use crate::java_values::{ByAddressGcManagedObject, JavaValue};
use crate::jvm_state::ClassStatus;
use crate::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassClass};
use crate::rust_jni::interface::array::*;
use crate::rust_jni::interface::array::array_region::*;
use crate::rust_jni::interface::array::new::*;
use crate::rust_jni::interface::call::call_nonstatic::*;
use crate::rust_jni::interface::call::call_nonvirtual::{
    call_nonvirtual_boolean_method, call_nonvirtual_boolean_method_a, call_nonvirtual_boolean_method_v, call_nonvirtual_byte_method, call_nonvirtual_byte_method_a, call_nonvirtual_byte_method_v, call_nonvirtual_char_method, call_nonvirtual_char_method_a, call_nonvirtual_char_method_v, call_nonvirtual_double_method, call_nonvirtual_double_method_a, call_nonvirtual_double_method_v, call_nonvirtual_float_method, call_nonvirtual_float_method_a, call_nonvirtual_float_method_v, call_nonvirtual_int_method,
    call_nonvirtual_int_method_a, call_nonvirtual_int_method_v, call_nonvirtual_long_method, call_nonvirtual_long_method_a, call_nonvirtual_long_method_v, call_nonvirtual_object_method, call_nonvirtual_object_method_a, call_nonvirtual_object_method_v, call_nonvirtual_short_method, call_nonvirtual_short_method_a, call_nonvirtual_short_method_v, call_nonvirtual_void_method, call_nonvirtual_void_method_a, call_nonvirtual_void_method_v,
};
use crate::rust_jni::interface::call::call_static::*;
use crate::rust_jni::interface::call::VarargProvider;
use crate::rust_jni::interface::exception::*;
use crate::rust_jni::interface::get_field::*;
use crate::rust_jni::interface::global_ref::*;
use crate::rust_jni::interface::instance_of::is_instance_of;
use crate::rust_jni::interface::local_frame::{delete_local_ref, new_local_ref, pop_local_frame, push_local_frame};
use crate::rust_jni::interface::method::get_method_id;
use crate::rust_jni::interface::misc::*;
use crate::rust_jni::interface::new_object::*;
use crate::rust_jni::interface::set_field::*;
use crate::rust_jni::interface::string::*;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_object_class, get_state, to_object};
use crate::utils::throw_npe;

//todo this should be in state impl
thread_local! {
    static JNI_INTERFACE: RefCell<*mut *const JNINativeInterface_> = RefCell::new(null_mut());
}

//GetFieldID
pub fn get_interface(state: &JVMState, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> *mut *const JNINativeInterface_ {
    // unsafe { state.set_int_state(int_state) };
    JNI_INTERFACE.with(|refcell| {
        if refcell.borrow().is_null() {
            let new = get_interface_impl(state, int_state);
            let jni_data_structure_ptr = Box::leak(box new) as *const JNINativeInterface_;
            refcell.replace(Box::leak(box (jni_data_structure_ptr)) as *mut *const JNINativeInterface_);
            //todo leak
        }
        let new_borrow = refcell.borrow();
        *new_borrow as *mut *const JNINativeInterface_
    })
}

fn get_interface_impl(state: &JVMState, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: unsafe { transmute(int_state) },
        reserved2: std::ptr::null_mut(),
        reserved3: std::ptr::null_mut(),
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

///MonitorEnter
//
// jint MonitorEnter(JNIEnv *env, jobject obj);
//
// Enters the monitor associated with the underlying Java object referred to by obj.
// Enters the monitor associated with the object referred to by obj. The obj reference must not be NULL.
//
// Each Java object has a monitor associated with it. If the current thread already owns the monitor associated with obj, it increments a counter in the monitor indicating the number of times this thread has entered the monitor. If the monitor associated with obj is not owned by any thread, the current thread becomes the owner of the monitor, setting the entry count of this monitor to 1. If another thread already owns the monitor associated with obj, the current thread waits until the monitor is released, then tries again to gain ownership.
//
// A monitor entered through a MonitorEnter JNI function call cannot be exited using the monitorexit Java virtual machine instruction or a synchronized method return. A MonitorEnter JNI function call and a monitorenter Java virtual machine instruction may race to enter the monitor associated with the same object.
//
// To avoid deadlocks, a monitor entered through a MonitorEnter JNI function call must be exited using the MonitorExit JNI call, unless the DetachCurrentThread call is used to implicitly release JNI monitors.
// LINKAGE:
// Index 217 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// obj: a normal Java object or class object.
// RETURNS:
//
// Returns “0” on success; returns a negative value on failure.
pub unsafe extern "C" fn monitor_enter(env: *mut JNIEnv, obj: jobject) -> jint {
    let jvm = get_state(env);
    match from_object(jvm, obj) {
        Some(x) => x,
        None => return JNI_ERR,
    }
        .monitor_lock(jvm, get_interpreter_state(env));
    JNI_OK as i32
}

///MonitorExit
//
// jint MonitorExit(JNIEnv *env, jobject obj);
//
// The current thread must be the owner of the monitor associated with the underlying Java object referred to by obj. The thread decrements the counter indicating the number of times it has entered this monitor. If the value of the counter becomes zero, the current thread releases the monitor.
//
// Native code must not use MonitorExit to exit a monitor entered through a synchronized method or a monitorenter Java virtual machine instruction.
// LINKAGE:
// Index 218 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// obj: a normal Java object or class object.
// RETURNS:
//
// Returns “0” on success; returns a negative value on failure.
// EXCEPTIONS:
//
// IllegalMonitorStateException: if the current thread does not own the monitor.
pub unsafe extern "C" fn monitor_exit(env: *mut JNIEnv, obj: jobject) -> jint {
    let jvm = get_state(env);
    match from_object(jvm, obj) {
        Some(x) => x,
        None => return JNI_ERR,
    }
        .monitor_unlock(jvm, get_interpreter_state(env));
    JNI_OK as i32
}

///GetStringChars
//
// const jchar * GetStringChars(JNIEnv *env, jstring string,
// jboolean *isCopy);
//
// Returns a pointer to the array of Unicode characters of the string. This pointer is valid until ReleaseStringChars() is called.
//
// If isCopy is not NULL, then *isCopy is set to JNI_TRUE if a copy is made; or it is set to JNI_FALSE if no copy is made.
// LINKAGE:
// Index 165 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// string: a Java string object.
//
// isCopy: a pointer to a boolean.
// RETURNS:
//
// Returns a pointer to a Unicode string, or NULL if the operation fails.
pub unsafe extern "C" fn get_string_chars(env: *mut JNIEnv, str: jstring, is_copy: *mut jboolean) -> *const jchar {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    *is_copy = u8::from(true);
    let string: JString = match JavaValue::Object(todo!() /*from_jclass(jvm,str)*/).cast_string() {
        None => return throw_npe(jvm, int_state),
        Some(string) => string,
    };
    let char_vec = string.value(jvm);
    let mut res = null_mut();
    jvm.native.native_interface_allocations.allocate_and_write_vec(char_vec, null_mut(), &mut res as *mut *mut jchar);
    res
}

///AllocObject
//
// jobject AllocObject(JNIEnv *env, jclass clazz);
//
// Allocates a new Java object without invoking any of the constructors for the object. Returns a reference to the object.
//
// The clazz argument must not refer to an array class.
// LINKAGE:
//
// Index 27 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// clazz: a Java class object.
// RETURNS:
//
// Returns a Java object, or NULL if the object cannot be constructed.
// THROWS:
//
// InstantiationException: if the class is an interface or an abstract class.
//
// OutOfMemoryError: if the system runs out of memory.
unsafe extern "C" fn alloc_object(env: *mut JNIEnv, clazz: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let res_object = new_object(jvm, int_state, &from_jclass(jvm, clazz).as_runtime_class(jvm)).unwrap_object();
    to_object(res_object)
}

///ToReflectedMethod
//
// jobject ToReflectedMethod(JNIEnv *env, jclass cls,
//    jmethodID methodID, jboolean isStatic);
//
// Converts a method ID derived from cls to a java.lang.reflect.Method or java.lang.reflect.Constructor object. isStatic must be set to JNI_TRUE if the method ID refers to a static field, and JNI_FALSE otherwise.
//
// Throws OutOfMemoryError and returns 0 if fails.
// LINKAGE:
//
// Index 9 in the JNIEnv interface function table.
// SINCE:
//
// JDK/JRE 1.2
unsafe extern "C" fn to_reflected_method(env: *mut JNIEnv, _cls: jclass, method_id: jmethodID, _is_static: jboolean) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let method_id: usize = transmute(method_id);
    let (runtime_class, index) = match jvm.method_table.read().unwrap().try_lookup(method_id) {
        Some(x) => x,
        None => return null_mut(),
    };
    let runtime_class_view = runtime_class.view();
    let method_view = runtime_class_view.method_view_i(index);
    let method_obj = match Method::method_object_from_method_view(jvm, int_state, &method_view) {
        Ok(method_obj) => method_obj,
        Err(_) => todo!(),
    };
    to_object(method_obj.object().into())
}

///ExceptionDescribe
//
// void ExceptionDescribe(JNIEnv *env);
//
// Prints an exception and a backtrace of the stack to a system error-reporting channel, such as stderr. This is a convenience routine provided for debugging.
// LINKAGE:
//
// Index 16 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
// ExceptionClear
//
// void ExceptionClear(JNIEnv *env);
//
// Clears any exception that is currently being thrown. If no exception is currently being thrown, this routine has no effect.
// LINKAGE:
//
// Index 17 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
unsafe extern "C" fn exception_describe(env: *mut JNIEnv) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    if let Some(throwing) = int_state.throw() {
        int_state.set_throw(None);
        match JavaValue::Object(todo!() /*throwing.into()*/).cast_throwable().print_stack_trace(jvm, int_state) {
            Ok(_) => {}
            Err(WasException {}) => {}
        };
    }
}

///FatalError
//
// void FatalError(JNIEnv *env, const char *msg);
//
// Raises a fatal error and does not expect the VM to recover. This function does not return.
// LINKAGE:
//
// Index 18 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// msg: an error message. The string is encoded in modified UTF-8.
// ExceptionCheck
// We introduce a convenience function to check for pending exceptions without creating a local reference to the exception object.
//
// jboolean ExceptionCheck(JNIEnv *env);
//
// Returns JNI_TRUE when there is a pending exception; otherwise, returns JNI_FALSE.
// LINKAGE:
// Index 228 in the JNIEnv interface function table.
// SINCE:
//
// JDK/JRE 1.2
unsafe extern "C" fn fatal_error(_env: *mut JNIEnv, msg: *const ::std::os::raw::c_char) {
    panic!("JNI raised a fatal error.\n JNI MSG: {}", CStr::from_ptr(msg).to_string_lossy())
}

///ThrowNew
//
// jint ThrowNew(JNIEnv *env, jclass clazz,
// const char *message);
//
// Constructs an exception object from the specified class with the message specified by message and causes that exception to be thrown.
// LINKAGE:
//
// Index 14 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// clazz: a subclass of java.lang.Throwable.
//
// message: the message used to construct the java.lang.Throwable object. The string is encoded in modified UTF-8.
// RETURNS:
//
// Returns 0 on success; a negative value on failure.
// THROWS:
//
// the newly constructed java.lang.Throwable object.
unsafe extern "C" fn throw_new(env: *mut JNIEnv, clazz: jclass, msg: *const ::std::os::raw::c_char) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (constructor_method_id, java_string_object) = {
        let runtime_class = from_jclass(jvm, clazz).as_runtime_class(jvm);
        let class_view = runtime_class.view();
        let desc = CMethodDescriptor {
            arg_types: vec![CPDType::Ref(CPRefType::Class(CClassName::string()))],
            return_type: CPDType::VoidType,
        };
        let constructor_method_id = match class_view.lookup_method(MethodName::constructor_init(), &desc) {
            None => return -1,
            Some(constructor) => jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), constructor.method_i() as u16),
        };
        let rust_string = match CStr::from_ptr(msg).to_str() {
            Ok(string) => string,
            Err(_) => return -2,
        }
            .to_string();
        let java_string = match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(rust_string)) {
            Ok(java_string) => java_string,
            Err(WasException {}) => return -4,
        };
        (constructor_method_id, to_object(java_string.object().into()))
    };
    let new_object = (**env).NewObjectA.as_ref().unwrap();
    let jvalue_ = jvalue { l: java_string_object };
    let obj = new_object(env, clazz, transmute(constructor_method_id), &jvalue_ as *const jvalue);
    let int_state = get_interpreter_state(env);
    int_state.set_throw(
        match from_object(jvm, obj) {
            None => return -3,
            Some(res) => res,
        }
            .into(),
    );
    JNI_OK as i32
}

unsafe extern "C" fn to_reflected_field(env: *mut JNIEnv, _cls: jclass, field_id: jfieldID, _is_static: jboolean) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);

    let field_id: FieldId = transmute(field_id);
    let (rc, i) = jvm.field_table.write().unwrap().lookup(field_id);
    to_object(
        match field_object_from_view(jvm, int_state, rc.clone(), rc.view().field(i as usize)) {
            Ok(res) => res,
            Err(_) => todo!(),
        }
            .unwrap_object(),
    )
}

//shouldn't take class as arg and should be an impl method on Field
pub fn field_object_from_view(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class_obj: Arc<RuntimeClass<'gc_life>>, f: FieldView) -> Result<JavaValue<'gc_life>, WasException> {
    let field_class_name_ = class_obj.clone().cpdtype();
    let parent_runtime_class = load_class_constant_by_type(jvm, int_state, &field_class_name_)?;

    let field_name = f.field_name();

    let field_desc_str = f.field_desc();
    let field_type = parse_field_descriptor(field_desc_str.as_str()).unwrap().field_type;

    let modifiers = f.access_flags() as i32;
    let slot = f.field_i() as i32;
    let clazz = parent_runtime_class.cast_class().expect("todo");
    let field_name_str = field_name.0.to_str(&jvm.string_pool);
    let name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(field_name_str))?.intern(jvm, int_state)?;
    let type_ = JClass::from_type(jvm, int_state, CPDType::from_ptype(&field_type, &jvm.string_pool))?;
    let signature = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(field_desc_str))?;
    let annotations_ = vec![]; //todo impl annotations.

    Ok(Field::init(jvm, int_state, clazz, name, type_, modifiers, slot, signature, annotations_)?.java_value())
}

unsafe extern "C" fn from_reflected_method(env: *mut JNIEnv, method: jobject) -> jmethodID {
    let jvm = get_state(env);
    let method_obj = JavaValue::Object(todo!() /*from_jclass(jvm,method)*/).cast_method();
    let runtime_class = method_obj.get_clazz(jvm).as_runtime_class(jvm);
    let param_types = method_obj.parameter_types(jvm).iter().map(|param| param.as_runtime_class(jvm).cpdtype()).collect_vec();
    let name_str = method_obj.get_name(jvm).to_rust_string(jvm);
    let name = MethodName(jvm.string_pool.add_name(name_str, false));
    runtime_class
        .clone()
        .view()
        .lookup_method_name(name)
        .iter()
        .find(|candiate_method| candiate_method.desc().arg_types == param_types.iter().map(|from| from.clone()).collect_vec())
        .map(|method| jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method.method_i() as u16) as jmethodID)
        .unwrap_or(transmute(-1isize))
}

unsafe extern "C" fn from_reflected_field(env: *mut JNIEnv, method: jobject) -> jfieldID {
    let jvm = get_state(env);
    let field_obj = JavaValue::Object(from_object(jvm, method)).cast_field();
    let runtime_class = field_obj.clazz(jvm).as_runtime_class(jvm);
    let field_name = FieldName(jvm.string_pool.add_name(field_obj.name(jvm).to_rust_string(jvm), false));
    runtime_class.view().fields().find(|candidate_field| candidate_field.field_name() == field_name).map(|field| field.field_i()).map(|field_i| jvm.field_table.write().unwrap().get_field_id(runtime_class, field_i as u16) as jfieldID).unwrap_or(transmute(-1isize))
}

unsafe extern "C" fn get_version(_env: *mut JNIEnv) -> jint {
    return 0x00010008;
}

pub fn define_class_safe(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, parsed: Arc<Classfile>, current_loader: LoaderName, class_view: ClassBackedView) -> Result<JavaValue<'gc_life>, WasException> {
    let class_name = class_view.name().unwrap_name();
    let class_view = Arc::new(class_view);
    let super_class = class_view.super_name().map(|name| check_initing_or_inited_class(jvm, int_state, name.into()).unwrap());
    let interfaces = class_view.interfaces().map(|interface| check_initing_or_inited_class(jvm, int_state, interface.interface_name().into()).unwrap()).collect_vec();
    let (recursive_num_fields, field_numbers) = get_field_numbers(&class_view, &super_class);
    let runtime_class = Arc::new(RuntimeClass::Object(RuntimeClassClass {
        class_view: class_view.clone(),
        field_numbers,
        recursive_num_fields,
        static_vars: Default::default(),
        parent: super_class,
        interfaces,
        status: RwLock::new(ClassStatus::UNPREPARED),
    }));
    let mut class_view_cache = HashMap::new();
    class_view_cache.insert(ClassWithLoader { class_name, loader: current_loader }, class_view.clone() as Arc<dyn ClassView>);
    let mut vf = VerifierContext {
        live_pool_getter: jvm.get_live_object_pool_getter(),
        classfile_getter: jvm.get_class_getter(int_state.current_loader()),
        string_pool: &jvm.string_pool,
        class_view_cache: Mutex::new(class_view_cache),
        current_loader: LoaderName::BootstrapLoader, //todo
        verification_types: Default::default(),
        debug: false,
    };
    match verify(&mut vf, class_name, LoaderName::BootstrapLoader /*todo*/) {
        Ok(_) => {}
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassNotFoundException)) => {
            let class = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("todo"))?;
            let to_throw = ClassNotFoundException::new(jvm, int_state, class)?.object().into();
            int_state.set_throw(to_throw);
            return Err(WasException {});
        }
        Err(TypeSafetyError::NotSafe(_)) => panic!(),
        Err(TypeSafetyError::Java5Maybe) => panic!(),
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassFileInvalid(_))) => panic!(),
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassVerificationError)) => panic!(),
    };
    let class_object = create_class_object(jvm, int_state, None, current_loader)?;
    let mut classes = jvm.classes.write().unwrap();
    classes.anon_classes.push(runtime_class.clone());
    classes.initiating_loaders.insert(class_name.clone().into(), (current_loader, runtime_class.clone()));
    classes.loaded_classes_by_type.entry(current_loader).or_insert(HashMap::new()).entry(class_name.clone().into()).insert(runtime_class.clone());
    classes.class_object_pool.insert(ByAddressGcManagedObject(class_object), ByAddress(runtime_class.clone()));
    drop(classes);
    jvm.sink_function_verification_date(&vf.verification_types, runtime_class.clone());
    prepare_class(jvm, int_state, Arc::new(ClassBackedView::from(parsed.clone(), &jvm.string_pool)), &mut *runtime_class.static_vars());
    runtime_class.set_status(ClassStatus::PREPARED);
    runtime_class.set_status(ClassStatus::INITIALIZING);
    initialize_class(runtime_class.clone(), jvm, int_state)?;
    runtime_class.set_status(ClassStatus::INITIALIZED);
    Ok(JavaValue::Object(get_or_create_class_object_force_loader(jvm, class_name.into(), int_state, current_loader).unwrap().into()))
}

pub unsafe extern "C" fn define_class(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let _name_string = CStr::from_ptr(name).to_str().unwrap(); //todo unused?
    let loader_name = JavaValue::Object(from_object(jvm, loader)).cast_class_loader().to_jvm_loader(jvm);
    let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
    if jvm.config.store_generated_classes {
        File::create("unsafe_define_class").unwrap().write_all(slice).unwrap();
    }
    let parsed = Arc::new(parse_class_file(&mut Cursor::new(slice)).expect("todo handle invalid"));
    //todo dupe with JVM_DefineClass and JVM_DefineClassWithSource
    to_object(
        match define_class_safe(jvm, int_state, parsed.clone(), loader_name, ClassBackedView::from(parsed, &jvm.string_pool)) {
            Ok(class_) => class_,
            Err(_) => todo!(),
        }
            .unwrap_object(),
    )
}

pub(crate) unsafe fn push_type_to_operand_stack(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, type_: &CPDType, l: &mut VarargProvider) {
    match type_ {
        CPDType::ByteType => {
            let byte_ = l.arg_byte();
            int_state.push_current_operand_stack(JavaValue::Byte(byte_))
        }
        CPDType::CharType => {
            let char_ = l.arg_char();
            int_state.push_current_operand_stack(JavaValue::Char(char_))
        }
        CPDType::DoubleType => {
            let double_ = l.arg_double();
            int_state.push_current_operand_stack(JavaValue::Double(double_))
        }
        CPDType::FloatType => {
            let float_ = l.arg_float();
            int_state.push_current_operand_stack(JavaValue::Float(float_))
        }
        CPDType::IntType => {
            let int: i32 = l.arg_int();
            int_state.push_current_operand_stack(JavaValue::Int(int))
        }
        CPDType::LongType => {
            let long: i64 = l.arg_long();
            int_state.push_current_operand_stack(JavaValue::Long(long))
        }
        CPDType::Ref(_) => {
            let native_object: jobject = l.arg_ptr();
            let o = from_object(jvm, native_object);
            int_state.push_current_operand_stack(JavaValue::Object(o));
        }
        CPDType::ShortType => {
            let short = l.arg_short();
            int_state.push_current_operand_stack(JavaValue::Short(short))
        }
        CPDType::BooleanType => {
            let boolean_ = l.arg_bool();
            int_state.push_current_operand_stack(JavaValue::Boolean(boolean_))
        }
        _ => panic!(),
    }
}

pub mod array;
pub mod call;
pub mod exception;
pub mod get_field;
pub mod global_ref;
pub mod instance_of;
pub mod local_frame;
pub mod method;
pub mod misc;
pub mod new_object;
pub mod set_field;
pub mod string;
pub mod util;