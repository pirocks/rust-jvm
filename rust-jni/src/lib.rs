extern crate libloading;
extern crate jni;
extern crate libc;
extern crate log;
extern crate simple_logger;

use log::trace;
use libloading::Library;
use libloading::Symbol;
use std::sync::Arc;
use rust_jvm_common::unified_types::ParsedType;
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::java_values::JavaValue;
use std::ffi::CStr;
use libffi::middle::Type;
use crate::value_conversion::to_native;
use libffi::middle::Arg;
use std::mem::transmute;
use crate::value_conversion::to_native_type;
use std::ops::Deref;
use libffi::middle::Cif;
use libffi::middle::CodePtr;
use jni::sys;
use jni::sys::jclass;


pub mod value_conversion;
pub mod mangling;

pub trait JNIContext {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> Option<JavaValue>;
}

#[derive(Debug)]
pub struct LibJavaLoading {
    pub lib: Library,
    pub registered_natives: RefCell<HashMap<Arc<RuntimeClass>, RefCell<HashMap<CPIndex, unsafe extern fn()>>>>,
}

impl LibJavaLoading {
    pub fn new(path: String) -> LibJavaLoading {
        trace!("Loading libjava.so from:`{}`", path);
        let lib = Library::new(path).unwrap();
        LibJavaLoading {
            lib,
            registered_natives: RefCell::new(HashMap::new()),
        }
    }
}

impl JNIContext for LibJavaLoading {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> Option<JavaValue> {
        let mangled = mangling::mangle(classfile.clone(), method_i);
        let symbol: Symbol<unsafe extern fn()> = unsafe { self.lib.get(mangled.as_bytes()).unwrap() };
        let raw = symbol.deref();
        let mut args_type = vec![Type::pointer(), Type::pointer()];
        let jclass: jclass = unsafe { transmute(&classfile) };
        let env = &get_interface(self);
        let mut c_args = vec![Arg::new(&&env), Arg::new(&jclass)];
        for x in args {
            args_type.push(to_native_type(x.clone()));
            c_args.push(to_native(x));//todo don't forget to free. and/or stack allocate
        }
        let cif = Cif::new(args_type.into_iter(), Type::f64());
        let fn_ptr = CodePtr::from_fun(*raw);
        let cif_res = unsafe {
            cif.call(fn_ptr, c_args.as_slice())
        };
        match return_type {
            ParsedType::VoidType => {
                None
            }
//            ParsedType::ByteType => {}
//            ParsedType::CharType => {}
//            ParsedType::DoubleType => {}
//            ParsedType::FloatType => {}
            ParsedType::IntType => {
                Some(JavaValue::Int(cif_res))
            }
//            ParsedType::LongType => {}
//            ParsedType::Class(_) => {}
//            ParsedType::ShortType => {}
//            ParsedType::BooleanType => {}
//            ParsedType::ArrayReferenceType(_) => {}
//            ParsedType::TopType => {}
//            ParsedType::NullType => {}
//            ParsedType::Uninitialized(_) => {}
//            ParsedType::UninitializedThis => {}
            _ => panic!()
        }
    }
}

use jni::sys::JNINativeMethod;
use jni::sys::jint;

unsafe extern "system" fn register_natives(env: *mut sys::JNIEnv,
                                           clazz: jclass,
                                           methods: *const JNINativeMethod,
                                           n_methods: jint) -> jint {
    trace!("Call to register_natives, n_methods: {}",n_methods);
    for to_register_i in 0..n_methods {
        let jni_context = &*((**env).reserved0 as *mut LibJavaLoading);
        let method = *methods.offset(to_register_i as isize);
        let expected_name: String = CStr::from_ptr(method.name).to_str().unwrap().to_string();
        let descriptor: String = CStr::from_ptr(method.signature).to_str().unwrap().to_string();
        let runtime_class: &Arc<RuntimeClass> = transmute(clazz);
        let classfile = &runtime_class.classfile;
        &classfile.methods.iter().enumerate().for_each(|(i, method_info)| {
            let descriptor_str = extract_string_from_utf8(&classfile.constant_pool[method_info.descriptor_index as usize]);
            let current_name = method_name(classfile, method_info);
            if current_name == expected_name && descriptor == descriptor_str {
                trace!("Registering method:{},{},{}", class_name(classfile).get_referred_name(), expected_name, descriptor_str);
                register_native_with_lib_java_loading(&jni_context, &method, &runtime_class, i)
            }
        });
    }
    0
}

fn register_native_with_lib_java_loading(jni_context: &LibJavaLoading, method: &JNINativeMethod, runtime_class: &Arc<RuntimeClass>, i: usize) -> () {
    if jni_context.registered_natives.borrow().contains_key(runtime_class) {

            unsafe { jni_context.registered_natives
                .borrow()
                .get(runtime_class)
                .unwrap()
                .borrow_mut()
                .insert(i as CPIndex, transmute(method.fnPtr)); }

    }else {
        let mut map = HashMap::new();
        map.insert(i as CPIndex, unsafe { transmute(method.fnPtr) });
        jni_context.registered_natives.borrow_mut().insert(runtime_class.clone(), RefCell::new(map));
    }
}

use std::cell::RefCell;
use rust_jvm_common::utils::{method_name, extract_string_from_utf8};
use std::collections::HashMap;
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classnames::class_name;

fn get_interface(l: &LibJavaLoading) -> sys::JNINativeInterface_ {
    sys::JNINativeInterface_ {
        reserved0: unsafe {transmute(l)},
        reserved1: std::ptr::null_mut(),
        reserved2: std::ptr::null_mut(),
        reserved3: std::ptr::null_mut(),
        GetVersion: None,
        DefineClass: None,
        FindClass: None,
        FromReflectedMethod: None,
        FromReflectedField: None,
        ToReflectedMethod: None,
        GetSuperclass: None,
        IsAssignableFrom: None,
        ToReflectedField: None,
        Throw: None,
        ThrowNew: None,
        ExceptionOccurred: None,
        ExceptionDescribe: None,
        ExceptionClear: None,
        FatalError: None,
        PushLocalFrame: None,
        PopLocalFrame: None,
        NewGlobalRef: None,
        DeleteGlobalRef: None,
        DeleteLocalRef: None,
        IsSameObject: None,
        NewLocalRef: None,
        EnsureLocalCapacity: None,
        AllocObject: None,
        NewObject: None,
        NewObjectV: None,
        NewObjectA: None,
        GetObjectClass: None,
        IsInstanceOf: None,
        GetMethodID: None,
        CallObjectMethod: None,
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
        GetStaticMethodID: None,
        CallStaticObjectMethod: None,
        CallStaticObjectMethodV: None,
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
        ReleaseStringChars: None,
        NewStringUTF: None,
        GetStringUTFLength: None,
        GetStringUTFChars: None,
        ReleaseStringUTFChars: None,
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
        ExceptionCheck: None,
        NewDirectByteBuffer: None,
        GetDirectBufferAddress: None,
        GetDirectBufferCapacity: None,
        GetObjectRefType: None,
    }
}