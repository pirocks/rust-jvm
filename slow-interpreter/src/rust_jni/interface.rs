use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use jni_bindings::{JNINativeInterface_, JNIEnv, jobject, jmethodID, jthrowable, jint, jclass, __va_list_tag};
use std::mem::transmute;
use std::ffi::{c_void, CStr, VaList};
use crate::rust_jni::{exception_check, register_natives, release_string_utfchars, get_method_id, MethodId};
use crate::rust_jni::native_util::{get_object_class, get_frame, get_state, to_object, from_object};
use crate::rust_jni::string::{release_string_chars, new_string_utf, get_string_utfchars};
use crate::instructions::invoke::{invoke_virtual_method_i, invoke_static_impl};
use rust_jvm_common::classfile::ACC_STATIC;
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::unified_types::ParsedType;
use runtime_common::java_values::{JavaValue, Object};
use log::trace;
use rust_jvm_common::classnames::class_name;
use crate::instructions::ldc::load_class_constant_by_name;
use std::sync::Arc;

//CallObjectMethod
//ExceptionOccurred
//DeleteLocalRef
pub fn get_interface(state: &InterpreterState, frame: Rc<StackEntry>) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: {
            let boxed = Box::new(frame);
            Box::into_raw(boxed) as *mut c_void
        },
        reserved2: std::ptr::null_mut(),
        reserved3: std::ptr::null_mut(),
        GetVersion: None,
        DefineClass: None,
        FindClass: Some(find_class),
        FromReflectedMethod: None,
        FromReflectedField: None,
        ToReflectedMethod: None,
        GetSuperclass: None,
        IsAssignableFrom: None,
        ToReflectedField: None,
        Throw: None,
        ThrowNew: None,
        ExceptionOccurred: Some(exception_occured),
        ExceptionDescribe: None,
        ExceptionClear: None,
        FatalError: None,
        PushLocalFrame: None,
        PopLocalFrame: None,
        NewGlobalRef: Some(new_global_ref),
        DeleteGlobalRef: None,
        DeleteLocalRef: Some(delete_local_ref),
        IsSameObject: None,
        NewLocalRef: None,
        EnsureLocalCapacity: Some(ensure_local_capacity),
        AllocObject: None,
        NewObject: None,
        NewObjectV: None,
        NewObjectA: None,
        GetObjectClass: Some(get_object_class),
        IsInstanceOf: None,
        GetMethodID: Some(get_method_id),
        CallObjectMethod: Some(unsafe { transmute::<_, unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jobject>(call_object_method as *mut c_void) }),
        CallObjectMethodV: None,
        CallObjectMethodA: None,
        CallBooleanMethod: None,
        CallBooleanMethodV: None,
        CallBooleanMethodA: None,
        CallByteMethod: None,
        CallByteMethodV: None,
        CallByteMethodA: None,
        CallCharMethod: None,
        CallCharMethodV: None,
        CallCharMethodA: None,
        CallShortMethod: None,
        CallShortMethodV: None,
        CallShortMethodA: None,
        CallIntMethod: None,
        CallIntMethodV: None,
        CallIntMethodA: None,
        CallLongMethod: None,
        CallLongMethodV: None,
        CallLongMethodA: None,
        CallFloatMethod: None,
        CallFloatMethodV: None,
        CallFloatMethodA: None,
        CallDoubleMethod: None,
        CallDoubleMethodV: None,
        CallDoubleMethodA: None,
        CallVoidMethod: None,
        CallVoidMethodV: None,
        CallVoidMethodA: None,
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
        GetFieldID: None,
        GetObjectField: None,
        GetBooleanField: None,
        GetByteField: None,
        GetCharField: None,
        GetShortField: None,
        GetIntField: None,
        GetLongField: None,
        GetFloatField: None,
        GetDoubleField: None,
        SetObjectField: None,
        SetBooleanField: None,
        SetByteField: None,
        SetCharField: None,
        SetShortField: None,
        SetIntField: None,
        SetLongField: None,
        SetFloatField: None,
        SetDoubleField: None,
        GetStaticMethodID: Some(get_static_method_id),
        CallStaticObjectMethod: None,
        CallStaticObjectMethodV: Some(unsafe { transmute::<_, unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut __va_list_tag) -> jobject>(call_static_object_method_v as *mut c_void) }),
        CallStaticObjectMethodA: None,
        CallStaticBooleanMethod: None,
        CallStaticBooleanMethodV: None,
        CallStaticBooleanMethodA: None,
        CallStaticByteMethod: None,
        CallStaticByteMethodV: None,
        CallStaticByteMethodA: None,
        CallStaticCharMethod: None,
        CallStaticCharMethodV: None,
        CallStaticCharMethodA: None,
        CallStaticShortMethod: None,
        CallStaticShortMethodV: None,
        CallStaticShortMethodA: None,
        CallStaticIntMethod: None,
        CallStaticIntMethodV: None,
        CallStaticIntMethodA: None,
        CallStaticLongMethod: None,
        CallStaticLongMethodV: None,
        CallStaticLongMethodA: None,
        CallStaticFloatMethod: None,
        CallStaticFloatMethodV: None,
        CallStaticFloatMethodA: None,
        CallStaticDoubleMethod: None,
        CallStaticDoubleMethodV: None,
        CallStaticDoubleMethodA: None,
        CallStaticVoidMethod: None,
        CallStaticVoidMethodV: None,
        CallStaticVoidMethodA: None,
        GetStaticFieldID: None,
        GetStaticObjectField: None,
        GetStaticBooleanField: None,
        GetStaticByteField: None,
        GetStaticCharField: None,
        GetStaticShortField: None,
        GetStaticIntField: None,
        GetStaticLongField: None,
        GetStaticFloatField: None,
        GetStaticDoubleField: None,
        SetStaticObjectField: None,
        SetStaticBooleanField: None,
        SetStaticByteField: None,
        SetStaticCharField: None,
        SetStaticShortField: None,
        SetStaticIntField: None,
        SetStaticLongField: None,
        SetStaticFloatField: None,
        SetStaticDoubleField: None,
        NewString: None,
        GetStringLength: None,
        GetStringChars: None,
        ReleaseStringChars: Some(release_string_chars),
        NewStringUTF: Some(new_string_utf),
        GetStringUTFLength: None,
        GetStringUTFChars: Some(get_string_utfchars),
        ReleaseStringUTFChars: Some(release_string_utfchars),
        GetArrayLength: None,
        NewObjectArray: None,
        GetObjectArrayElement: None,
        SetObjectArrayElement: None,
        NewBooleanArray: None,
        NewByteArray: None,
        NewCharArray: None,
        NewShortArray: None,
        NewIntArray: None,
        NewLongArray: None,
        NewFloatArray: None,
        NewDoubleArray: None,
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
        GetByteArrayRegion: None,
        GetCharArrayRegion: None,
        GetShortArrayRegion: None,
        GetIntArrayRegion: None,
        GetLongArrayRegion: None,
        GetFloatArrayRegion: None,
        GetDoubleArrayRegion: None,
        SetBooleanArrayRegion: None,
        SetByteArrayRegion: None,
        SetCharArrayRegion: None,
        SetShortArrayRegion: None,
        SetIntArrayRegion: None,
        SetLongArrayRegion: None,
        SetFloatArrayRegion: None,
        SetDoubleArrayRegion: None,
        RegisterNatives: Some(register_natives),
        UnregisterNatives: None,
        MonitorEnter: None,
        MonitorExit: None,
        GetJavaVM: None,
        GetStringRegion: None,
        GetStringUTFRegion: None,
        GetPrimitiveArrayCritical: None,
        ReleasePrimitiveArrayCritical: None,
        GetStringCritical: None,
        ReleaseStringCritical: None,
        NewWeakGlobalRef: None,
        DeleteWeakGlobalRef: None,
        ExceptionCheck: Some(exception_check),
        NewDirectByteBuffer: None,
        GetDirectBufferAddress: None,
        GetDirectBufferCapacity: None,
        GetObjectRefType: None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
    let method_id = (method_id as *mut MethodId).as_ref().unwrap();
    let classfile = method_id.class.classfile.clone();
    let method = &classfile.methods[method_id.method_i];
//    dbg!(method.access_flags & ACC_STATIC);
    if method.access_flags & ACC_STATIC > 0 {
        unimplemented!()
    }
    let state = get_state(env);
    let frame = get_frame(env);
    //todo simplify use of this.
    let exp_method_name = method.method_name(&classfile);
    let exp_descriptor_str = classfile.constant_pool[method.descriptor_index as usize].extract_string_from_utf8();
    let parsed = parse_method_descriptor(&method_id.class.loader, exp_descriptor_str.as_str()).unwrap();

    frame.push(JavaValue::Object(from_object(obj)));
    for type_ in &parsed.parameter_types {
        match type_ {
            ParsedType::ByteType => unimplemented!(),
            ParsedType::CharType => unimplemented!(),
            ParsedType::DoubleType => unimplemented!(),
            ParsedType::FloatType => unimplemented!(),
            ParsedType::IntType => unimplemented!(),
            ParsedType::LongType => unimplemented!(),
            ParsedType::Class(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            ParsedType::ShortType => unimplemented!(),
            ParsedType::BooleanType => unimplemented!(),
            ParsedType::ArrayReferenceType(_) => unimplemented!(),
            ParsedType::VoidType => unimplemented!(),
            ParsedType::TopType => unimplemented!(),
            ParsedType::NullType => unimplemented!(),
            ParsedType::Uninitialized(_) => unimplemented!(),
            ParsedType::UninitializedThis => unimplemented!(),
        }
    }
    //todo add params into operand stack;
    trace!("Call:{} {}", class_name(&from_object(obj).unwrap().class_pointer.classfile).get_referred_name(), exp_method_name);
    invoke_virtual_method_i(state, frame.clone(), exp_method_name, parsed, method_id.class.clone(), method_id.method_i, method);
    let res = frame.pop().unwrap_object();
    to_object(res)
}

unsafe extern "C" fn exception_occured(_env: *mut JNIEnv) -> jthrowable {
    //exceptions don't happen yet todo
    std::ptr::null_mut()
}


unsafe extern "C" fn delete_local_ref(_env: *mut JNIEnv, _obj: jobject) {
    //todo no gc, just leak
}

unsafe extern "C" fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    //we always have ram. todo
    return 0;
}

unsafe extern "C" fn find_class(env: *mut JNIEnv, c_name: *const ::std::os::raw::c_char) -> jclass {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let state = get_state(env);
    let frame = get_frame(env);
    load_class_constant_by_name(state, &frame, name);
    let obj = frame.pop().unwrap_object();
    to_object(obj)
}

unsafe extern "C" fn new_global_ref(_env: *mut JNIEnv, lobj: jobject) -> jobject {
    let obj = from_object(lobj);
    match &obj {
        None => {}
        Some(o) => {
            Box::leak(Box::new(o.clone()));
        }
    }
    to_object(obj)
}

unsafe extern "C" fn get_static_method_id(
    _env: *mut JNIEnv,
    clazz: jclass,
    name: *const ::std::os::raw::c_char,
    sig: *const ::std::os::raw::c_char,
) -> jmethodID {
    let method_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let method_descriptor_str = CStr::from_ptr(sig).to_str().unwrap().to_string();
    let class_obj: Arc<Object> = from_object(clazz).unwrap();
    //todo dup
    let runtime_class = class_obj.object_class_object_pointer.borrow().as_ref().unwrap().clone();
    let classfile = &runtime_class.classfile;
    let all_methods = &classfile.methods;
    let (method_i, _) = all_methods.iter().enumerate().find(|(_, m)| {
        let cur_desc = classfile.constant_pool[m.descriptor_index as usize].extract_string_from_utf8();
        let cur_method_name = m.method_name(classfile);
        cur_method_name == method_name &&
            method_descriptor_str == cur_desc &&
            m.access_flags & ACC_STATIC > 0
    }).unwrap();
    let res = Box::into_raw(Box::new(MethodId { class: runtime_class.clone(), method_i }));
    transmute(res)
}

unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: VaList) -> jobject {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
    let classfile = &method_id.class.classfile;
    let constant_pool = &classfile.constant_pool;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = constant_pool[method.descriptor_index as usize].extract_string_from_utf8();
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(&method_id.class.loader, method_descriptor_str.as_str()).unwrap();
    //todo dup
    for type_ in &parsed.parameter_types {
        match type_ {
            ParsedType::ByteType => unimplemented!(),
            ParsedType::CharType => unimplemented!(),
            ParsedType::DoubleType => unimplemented!(),
            ParsedType::FloatType => unimplemented!(),
            ParsedType::IntType => unimplemented!(),
            ParsedType::LongType => unimplemented!(),
            ParsedType::Class(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            ParsedType::ShortType => unimplemented!(),
            ParsedType::BooleanType => unimplemented!(),
            ParsedType::ArrayReferenceType(_) => unimplemented!(),
            ParsedType::VoidType => unimplemented!(),
            ParsedType::TopType => unimplemented!(),
            ParsedType::NullType => unimplemented!(),
            ParsedType::Uninitialized(_) => unimplemented!(),
            ParsedType::UninitializedThis => unimplemented!(),
        }
    }
    invoke_static_impl(state, frame.clone(), parsed, method_id.class.clone(), method_id.method_i, method);
    let res = frame.pop().unwrap_object();
    to_object(res)
}