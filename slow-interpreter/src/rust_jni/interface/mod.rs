use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::fs::File;
use std::io::{Cursor, Write};
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

use by_address::ByAddress;

use classfile_parser::parse_class_file;
use classfile_view::loading::LoaderName;
use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jbyte, jclass, jint, jio_vfprintf, jmethodID, JNIEnv, JNINativeInterface_, jobject, jsize, JVM_Available};
use rust_jvm_common::classfile::Classfile;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::create_class_object;
use crate::class_objects::get_or_create_class_object;
use crate::java_values::{default_value, JavaValue};
use crate::jvm_state::ClassStatus;
use crate::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassClass};
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
use crate::rust_jni::interface::method::get_method_id;
use crate::rust_jni::interface::misc::*;
use crate::rust_jni::interface::new_object::*;
use crate::rust_jni::interface::set_field::*;
use crate::rust_jni::interface::string::*;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_object_class, get_state, to_object};

//todo this should be in state impl
thread_local! {
    static JNI_INTERFACE: RefCell<*mut *const JNINativeInterface_> = RefCell::new(null_mut());
}

//GetFieldID
pub fn get_interface(state: &JVMState, int_state: &mut InterpreterStateGuard) -> *mut *const JNINativeInterface_ {
    // unsafe { state.set_int_state(int_state) };
    JNI_INTERFACE.with(|refcell| {
        let new = get_interface_impl(state, int_state);
        let jni_data_structure_ptr = Box::leak(box new) as *const JNINativeInterface_;
        refcell.replace(Box::leak(box (jni_data_structure_ptr)) as *mut *const JNINativeInterface_);//todo leak
        let new_borrow = refcell.borrow();
        *new_borrow as *mut *const JNINativeInterface_
    })
}

fn get_interface_impl(state: &JVMState, int_state: &mut InterpreterStateGuard) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: unsafe { transmute(int_state) },
        reserved2: std::ptr::null_mut(),
        reserved3: std::ptr::null_mut(),
        GetVersion: Some(get_version),
        DefineClass: Some(define_class),
        FindClass: Some(find_class),
        FromReflectedMethod: Some(from_reflected_method),
        FromReflectedField: None, //todo
        ToReflectedMethod: None, //todo
        GetSuperclass: Some(get_superclass),
        IsAssignableFrom: Some(is_assignable_from),
        ToReflectedField: None, //todo
        Throw: Some(throw),
        ThrowNew: None, //todo
        ExceptionOccurred: Some(exception_occured),
        ExceptionDescribe: None, //todo
        ExceptionClear: Some(exception_clear),
        FatalError: None, //todo
        PushLocalFrame: Some(push_local_frame),
        PopLocalFrame: Some(pop_local_frame),
        NewGlobalRef: Some(new_global_ref),
        DeleteGlobalRef: Some(delete_global_ref),
        DeleteLocalRef: Some(delete_local_ref),
        IsSameObject: Some(is_same_object),
        NewLocalRef: Some(new_local_ref),
        EnsureLocalCapacity: Some(ensure_local_capacity),
        AllocObject: None, //todo
        NewObject: Some(unsafe { transmute(new_object as *mut c_void) }),
        NewObjectV: Some(unsafe { transmute(new_object_v as *mut c_void) }),
        NewObjectA: None, //todo
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
        CallNonvirtualObjectMethod: None, //todo
        CallNonvirtualObjectMethodV: None, //todo
        CallNonvirtualObjectMethodA: None, //todo
        CallNonvirtualBooleanMethod: None, //todo
        CallNonvirtualBooleanMethodV: None, //todo
        CallNonvirtualBooleanMethodA: None, //todo
        CallNonvirtualByteMethod: None, //todo
        CallNonvirtualByteMethodV: None, //todo
        CallNonvirtualByteMethodA: None, //todo
        CallNonvirtualCharMethod: None, //todo
        CallNonvirtualCharMethodV: None, //todo
        CallNonvirtualCharMethodA: None, //todo
        CallNonvirtualShortMethod: None, //todo
        CallNonvirtualShortMethodV: None, //todo
        CallNonvirtualShortMethodA: None, //todo
        CallNonvirtualIntMethod: None, //todo
        CallNonvirtualIntMethodV: None, //todo
        CallNonvirtualIntMethodA: None, //todo
        CallNonvirtualLongMethod: None, //todo
        CallNonvirtualLongMethodV: None, //todo
        CallNonvirtualLongMethodA: None, //todo
        CallNonvirtualFloatMethod: None, //todo
        CallNonvirtualFloatMethodV: None, //todo
        CallNonvirtualFloatMethodA: None, //todo
        CallNonvirtualDoubleMethod: None, //todo
        CallNonvirtualDoubleMethodV: None, //todo
        CallNonvirtualDoubleMethodA: None, //todo
        CallNonvirtualVoidMethod: None, //todo
        CallNonvirtualVoidMethodV: None, //todo
        CallNonvirtualVoidMethodA: None, //todo
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
        GetStringChars: None, //todo
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
        GetBooleanArrayElements: None, //todo
        GetByteArrayElements: None, //todo
        GetCharArrayElements: None, //todo
        GetShortArrayElements: None, //todo
        GetIntArrayElements: None, //todo
        GetLongArrayElements: Some(get_long_array_elements),
        GetFloatArrayElements: None, //todo
        GetDoubleArrayElements: None, //todo
        ReleaseBooleanArrayElements: None, //todo
        ReleaseByteArrayElements: None, //todo
        ReleaseCharArrayElements: None, //todo
        ReleaseShortArrayElements: None, //todo
        ReleaseIntArrayElements: None, //todo
        ReleaseLongArrayElements: Some(release_long_array_elements),
        ReleaseFloatArrayElements: None, //todo
        ReleaseDoubleArrayElements: None, //todo
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
        UnregisterNatives: None, //todo
        MonitorEnter: None, //todo
        MonitorExit: None, //todo
        GetJavaVM: Some(get_java_vm),
        GetStringRegion: Some(get_string_region),
        GetStringUTFRegion: Some(get_string_utfregion),
        GetPrimitiveArrayCritical: Some(get_primitive_array_critical),
        ReleasePrimitiveArrayCritical: Some(release_primitive_array_critical),
        GetStringCritical: None, //todo
        ReleaseStringCritical: None, //todo
        NewWeakGlobalRef: Some(new_weak_global_ref),
        DeleteWeakGlobalRef: Some(delete_weak_global_ref),
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: None, //todo
        GetDirectBufferAddress: None, //todo
        GetDirectBufferCapacity: None, //todo
        GetObjectRefType: None, //todo
    }
}

unsafe extern "C" fn from_reflected_method(env: *mut JNIEnv, method: jobject) -> jmethodID {
    let jvm = get_state(env);
    //todo support java.lang.Constructor as well
    let method_obj = JavaValue::Object(from_object(method)).cast_method();
    let runtime_class = method_obj.get_clazz().as_runtime_class(jvm);
    let param_types = method_obj.parameter_types().iter().map(|param| param.as_runtime_class(jvm).ptypeview()).collect::<Vec<_>>();
    let name = method_obj.get_name().to_rust_string();
    runtime_class.view().lookup_method_name(&name).iter().find(|candiate_method| {
        candiate_method.desc().parameter_types == param_types
    }).map(|method| jvm.method_table.write().unwrap().get_method_id(runtime_class, method.method_i() as u16) as jmethodID)
        .unwrap_or(transmute(-1))
}

unsafe extern "C" fn get_version(env: *mut JNIEnv) -> jint {
    return 0x00010008;
}

pub fn define_class_safe(jvm: &JVMState, int_state: &mut InterpreterStateGuard, parsed: Arc<Classfile>, current_loader: LoaderName, class_view: ClassView) -> JavaValue {
    let class_name = class_view.name();
    let runtime_class = Arc::new(RuntimeClass::Object(RuntimeClassClass {
        class_view: Arc::new(class_view),
        static_vars: Default::default(),
        status: RwLock::new(ClassStatus::UNPREPARED),
    }));
    let class_object = create_class_object(jvm, int_state, None, current_loader);
    let mut classes = jvm.classes.write().unwrap();
    let current_loader = int_state.current_loader();
    classes.anon_classes.write().unwrap().push(runtime_class.clone());
    classes.initiating_loaders.insert(class_name.clone().into(), (current_loader, runtime_class.clone()));
    classes.loaded_classes_by_type.entry(current_loader).or_insert(HashMap::new()).entry(class_name.clone().into()).insert(runtime_class.clone());
    classes.class_object_pool.insert(ByAddress(class_object), ByAddress(runtime_class.clone()));
    drop(classes);
    prepare_class(jvm, int_state, parsed.clone(), &mut *runtime_class.static_vars());
    runtime_class.set_status(ClassStatus::PREPARED);
    runtime_class.set_status(ClassStatus::INITIALIZING);
    initialize_class(runtime_class.clone(), jvm, int_state).unwrap();
    runtime_class.set_status(ClassStatus::INITIALIZED);
    JavaValue::Object(get_or_create_class_object(jvm, class_name.into(), int_state).unwrap().into())
}

unsafe extern "C" fn define_class(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let _name_string = CStr::from_ptr(name).to_str().unwrap();//todo unused?
    let loader_name = JavaValue::Object(from_object(loader)).cast_class_loader().to_jvm_loader(jvm);
    let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
    if jvm.store_generated_classes { File::create("unsafe_define_class").unwrap().write_all(slice).unwrap(); }
    let parsed = Arc::new(parse_class_file(&mut Cursor::new(slice)).expect("todo handle invalid"));
    //todo dupe with JVM_DefineClass and JVM_DefineClassWithSource
    to_object(define_class_safe(jvm, int_state, parsed.clone(), loader_name, ClassView::from(parsed)).unwrap_object())
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
pub mod new_object;
pub mod method;