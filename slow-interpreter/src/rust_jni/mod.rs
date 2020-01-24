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
use runtime_common::java_values::{JavaValue, Object, ObjectPointer};
use std::ffi::CStr;
use libffi::middle::Type;
use libffi::middle::Arg;
use std::mem::transmute;
use std::ops::Deref;
use libffi::middle::Cif;
use libffi::middle::CodePtr;
use jni::sys::jclass;
use std::cell::{RefCell, Ref};
use rust_jvm_common::utils::{method_name, extract_string_from_utf8, extract_class_from_constant_pool};
use std::collections::HashMap;
use rust_jvm_common::classfile::{CPIndex, MethodInfo, Classfile};
use rust_jvm_common::classnames::{class_name, ClassName};
use std::os::raw::c_char;
use std::borrow::Borrow;
use std::alloc::Layout;
use std::mem::size_of;
use std::convert::TryInto;
use runtime_common::{InterpreterState, LibJavaLoading, CallStackEntry};
use std::rc::Rc;
use jni::sys::*;
use crate::get_or_create_class_object;
use crate::rust_jni::value_conversion::to_native_type;
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classfile::InstructionInfo::ret;


pub mod value_conversion;
pub mod mangling;

pub fn new_java_loading(path: String) -> LibJavaLoading {
    trace!("Loading libjava.so from:`{}`", path);
    crate::rust_jni::libloading::os::unix::Library::open("libjvm.so".into(), dlopen::RTLD_LAZY.try_into().unwrap()).unwrap();
    let loaded = crate::rust_jni::libloading::os::unix::Library::open(path.clone().into(), dlopen::RTLD_LAZY.try_into().unwrap()).unwrap();
    let lib = Library::from(loaded);
    LibJavaLoading {
        lib,
        registered_natives: RefCell::new(HashMap::new()),
    }
}


pub fn call(state: &mut InterpreterState, current_frame: Rc<CallStackEntry>, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> Option<JavaValue> {
    let mangled = mangling::mangle(classfile.clone(), method_i);
    let symbol: Symbol<unsafe extern fn()> = unsafe { state.jni.lib.get(mangled.as_bytes()).unwrap() };
    let raw = symbol.deref();
    let mut args_type = vec![Type::pointer(), Type::pointer()];
    let jclass: jclass = unsafe { transmute(&classfile) };
    let env = &get_interface(state, current_frame);
    let mut c_args = vec![Arg::new(&&env), Arg::new(&jclass)];
    for j in args {
        args_type.push(to_native_type(j.clone()));
        c_args.push(match j {
            JavaValue::Long(l) => Arg::new(&l),
            JavaValue::Int(i) => Arg::new(&i),
            JavaValue::Short(s) => Arg::new(&s),
            JavaValue::Byte(b) => Arg::new(&b),
            JavaValue::Boolean(b) => Arg::new(&b),
            JavaValue::Char(c) => Arg::new(&c),
            JavaValue::Float(f) => Arg::new(&f),
            JavaValue::Double(d) => Arg::new(&d),
            JavaValue::Array(_) => unimplemented!(),
            JavaValue::Object(o) => match o {
                None => Arg::new(&(std::ptr::null() as *const Object)),
                Some(op) => {
                    let x = Arc::into_raw(op.object.clone());
//                    dbg!(&x);
                    Arg::new(x.borrow())
                }
            },
            JavaValue::Top => panic!()
        });
    }
    let cif = Cif::new(args_type.into_iter(), Type::f64());
    let fn_ptr = CodePtr::from_fun(*raw);
//    dbg!(&c_args);
//    dbg!(&fn_ptr);
    let cif_res: usize = unsafe {
        cif.call(fn_ptr, c_args.as_slice())
    };
    match return_type {
        ParsedType::VoidType => {
            None
        }
//            ParsedType::ByteType => {}
//            ParsedType::CharType => {}
        ParsedType::DoubleType => {
            Some(JavaValue::Double(unsafe { transmute(cif_res) }))
        }
//            ParsedType::FloatType => {}
        ParsedType::IntType => {
            Some(JavaValue::Int(cif_res as i32))
        }
        ParsedType::LongType => {
            Some(JavaValue::Long(cif_res as i64))
        }
        ParsedType::Class(_) => {
            unsafe {
                Some(JavaValue::Object(ObjectPointer { object: (Arc::from_raw(transmute(cif_res))) }.into()))
            }
        }
//            ParsedType::ShortType => {}
//            ParsedType::BooleanType => {}
//            ParsedType::ArrayReferenceType(_) => {}
//            ParsedType::TopType => {}
//            ParsedType::NullType => {}
//            ParsedType::Uninitialized(_) => {}
//            ParsedType::UninitializedThis => {}
        _ => {
            dbg!(return_type);
            panic!()
        }
    }
}


unsafe extern "system" fn register_natives(env: *mut JNIEnv,
                                           clazz: jclass,
                                           methods: *const JNINativeMethod,
                                           n_methods: jint) -> jint {
    trace!("Call to register_natives, n_methods: {}", n_methods);
    for to_register_i in 0..n_methods {
        let jni_context = &(*((**env).reserved0 as *mut InterpreterState)).jni;
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
                register_native_with_lib_java_loading(jni_context, &method, &runtime_class, i)
            }
        });
    }
    0
}

//todo shouldn't this be handled by a registered native
unsafe extern "system" fn get_string_utfchars(_env: *mut JNIEnv,
                                              name: jstring,
                                              is_copy: *mut jboolean) -> *const c_char {
    let str_obj: Arc<Object> = Arc::from_raw(transmute(name));
    let unwrapped = str_obj.fields.borrow().get("value").unwrap().clone().unwrap_array();
    let refcell: &RefCell<Vec<JavaValue>> = &unwrapped;
    let char_array: &Ref<Vec<JavaValue>> = &refcell.borrow();
    let chars_layout = Layout::from_size_align((char_array.len() + 1) * size_of::<c_char>(), size_of::<c_char>()).unwrap();
    let res = std::alloc::alloc(chars_layout) as *mut c_char;
    char_array.iter().enumerate().for_each(|(i, j)| {
        let cur = j.unwrap_char() as u8;
        res.offset(i as isize).write(transmute(cur))
    });
    res.offset(char_array.len() as isize).write(0);//null terminate
    if is_copy != std::ptr::null_mut() {
        unimplemented!()
    }
    return res;
}

fn register_native_with_lib_java_loading(jni_context: &LibJavaLoading, method: &JNINativeMethod, runtime_class: &Arc<RuntimeClass>, i: usize) -> () {
    if jni_context.registered_natives.borrow().contains_key(runtime_class) {
        unsafe {
            jni_context.registered_natives
                .borrow()
                .get(runtime_class)
                .unwrap()
                .borrow_mut()
                .insert(i as CPIndex, transmute(method.fnPtr));
        }
    } else {
        let mut map = HashMap::new();
        map.insert(i as CPIndex, unsafe { transmute(method.fnPtr) });
        jni_context.registered_natives.borrow_mut().insert(runtime_class.clone(), RefCell::new(map));
    }
}

unsafe extern "system" fn release_string_chars(_env: *mut JNIEnv, _str: jstring, _chars: *const jchar) {
    unimplemented!()
}

unsafe extern "system" fn release_string_utfchars(_env: *mut JNIEnv, _str: jstring, chars: *const c_char) {
    let len = libc::strlen(chars);
    let chars_layout = Layout::from_size_align((len + 1) * size_of::<c_char>(), size_of::<c_char>()).unwrap();
    std::alloc::dealloc(chars as *mut u8, chars_layout);
}

unsafe extern "system" fn exception_check(_env: *mut JNIEnv) -> jboolean {
    false as jboolean//todo exceptions are not needed for hello world so if we encounter an exception we just pretend it didn't happen
}

pub fn get_all_methods(classfile: Arc<Classfile>, loader: Arc<dyn Loader + Send + Sync>, bl: Arc<dyn Loader + Send + Sync>) -> Vec<(Arc<Classfile>,usize)> {
    let mut res = vec![];
    classfile.methods.iter().enumerate().for_each( |(i,m)|{
        res.push((classfile.clone(),i));
    });
    if classfile.super_class == 0 {
        let object = loader.clone().load_class(loader.clone(), &ClassName::Str("java/lang/Object".to_string()), bl).unwrap();
        object.methods.iter().enumerate().for_each( |(i,_)|{
            res.push((object.clone(),i));
        });
    } else {
        let class_entry = extract_class_from_constant_pool(classfile.super_class,&classfile);
        let name = extract_string_from_utf8(&classfile.constant_pool[class_entry.name_index as usize]);
        let super_classfile = loader.load_class(loader.clone(), &ClassName::Str(name), bl.clone()).unwrap();
        for (c, i) in get_all_methods(super_classfile, loader.clone(), bl.clone()) {
            res.push((c, i));//todo accidental O(n^2)
        }
    }

    return res;
}

//for now a method id is a pair of class pointers and i.
unsafe extern "system" fn get_method_id(env: *mut JNIEnv,
                                        clazz: jclass,
                                        name: *const c_char,
                                        sig: *const c_char)
                                        -> jmethodID {
    let mut method_name = String::new();
    let name_len = libc::strlen(name);
    for i in 0..name_len {
        method_name.push(name.offset(i as isize).read() as u8 as char);
    }

    let mut method_descriptor_str = String::new();
    //todo dup
    let desc_len = libc::strlen(sig);
    for i in 0..desc_len {
        method_descriptor_str.push(sig.offset(i as isize).read() as u8 as char);
    }

    let state = &mut (*((**env).reserved0 as *mut InterpreterState));
    let class_obj: Arc<Object> = Arc::from_raw(transmute(clazz));
    let object_option = class_obj.object_class_object_pointer.borrow();
    let classfile = &object_option.as_ref().unwrap().classfile;
    dbg!(class_name(classfile).get_referred_name());
    let (method_i, (c, m)) = get_all_methods(classfile.clone(),class_obj.class_pointer.loader.clone(),state.bootstrap_loader.clone()).iter().enumerate().find(|(_, (c,i))| {
//        let cur_desc = extract_string_from_utf8(&c.constant_pool[(*m).descriptor_index as usize]);
        dbg!(&rust_jvm_common::utils::method_name(c, m));
        dbg!(&method_name);
        dbg!(&method_descriptor_str);
        dbg!(&cur_desc);
        rust_jvm_common::utils::method_name(c, m) == method_name &&
            method_descriptor_str == cur_desc
    }).unwrap();
    transmute(&m)
}

unsafe extern "system" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    assert_ne!(obj, std::ptr::null_mut());
    let obj: Arc<Object> = Arc::from_raw(transmute(obj));
    let state = &mut (*((**env).reserved0 as *mut InterpreterState));
    let frame = Rc::from_raw((**env).reserved1 as *const CallStackEntry);
    let class_object = get_or_create_class_object(state, &class_name(&obj.class_pointer.classfile), frame, obj.class_pointer.loader.clone());
    Arc::into_raw(class_object) as jclass
}

fn get_interface(state: &InterpreterState, frame: Rc<CallStackEntry>) -> JNINativeInterface_ {
    JNINativeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: unsafe { transmute(Rc::into_raw(frame)) },
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
        GetObjectClass: Some(get_object_class),
        IsInstanceOf: None,
        GetMethodID: Some(get_method_id),
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
        ReleaseStringChars: Some(release_string_chars),
        NewStringUTF: None,
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

pub mod dlopen {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!("../../gen", "/dlopen.rs"));
}