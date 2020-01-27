extern crate libloading;
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
use std::cell::RefCell;
use rust_jvm_common::utils::{method_name, extract_string_from_utf8, get_super_class_name};
use std::collections::HashMap;
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classnames::{class_name, ClassName};
use std::os::raw::{c_char, c_void};
use std::alloc::Layout;
use std::mem::size_of;
use std::convert::TryInto;
use runtime_common::{InterpreterState, LibJavaLoading, CallStackEntry};
use std::rc::Rc;
use crate::rust_jni::value_conversion::{to_native_type, to_native};
use crate::interpreter_util::check_inited_class;
use jni_bindings::{jclass, JNIEnv, JNINativeMethod, jint, jstring, jboolean, jmethodID};
use crate::rust_jni::native_util::{get_state, get_frame, from_object};
use crate::rust_jni::interface::get_interface;
use std::io::Error;


pub mod value_conversion;
pub mod mangling;

pub fn new_java_loading(path: String) -> LibJavaLoading {
    trace!("Loading libjava.so from:`{}`", path);
//    crate::rust_jni::libloading::os::unix::Library::open("libjvm.so".into(), (dlopen::RTLD_NOW | dlopen::RTLD_GLOBAL).try_into().unwrap()).unwrap();
    let loaded = crate::rust_jni::libloading::os::unix::Library::open(path.clone().into(), (dlopen::RTLD_NOW | dlopen::RTLD_GLOBAL).try_into().unwrap()).unwrap();
    let lib = Library::from(loaded);
    LibJavaLoading {
        lib,
        registered_natives: RefCell::new(HashMap::new()),
    }
}


pub fn call(state: &mut InterpreterState, current_frame: Rc<CallStackEntry>, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> Result<Option<JavaValue>, Error> {
    let mangled = mangling::mangle(classfile.clone(), method_i);
    let raw = {
        let symbol: Symbol<unsafe extern fn()> = unsafe {
            match state.jni.lib.get(mangled.as_bytes()) {
                Ok(o) => o,
                Err(e) => return Result::Err(e),
            }
        };
        symbol.deref().clone()
    };
    call_impl(state, current_frame, classfile, args, return_type, &raw)
}

pub fn call_impl(state: &mut InterpreterState, current_frame: Rc<CallStackEntry>, classfile: Arc<RuntimeClass>, args: Vec<JavaValue>, return_type: ParsedType, raw: &unsafe extern "C" fn()) -> Result<Option<JavaValue>, Error> {
    let mut args_type = vec![Type::pointer(), Type::pointer()];
    let jclass: jclass = unsafe { transmute(&classfile) };
    let env = &get_interface(state, current_frame);
    let mut c_args = vec![Arg::new(&&env), Arg::new(&jclass)];
//todo inconsistent
//    dbg!(&args);
    for j in args {
        args_type.push(to_native_type(j.clone()));
        c_args.push(to_native(j));
    }
    let cif = Cif::new(args_type.into_iter(), Type::usize());
//todo what if float
    let fn_ptr = CodePtr::from_fun(*raw);
    let cif_res: *mut c_void = unsafe {
        cif.call(fn_ptr, c_args.as_slice())
    };
    Result::Ok(match return_type {
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
                Some(JavaValue::Object(ObjectPointer { object: from_object(transmute(cif_res)).unwrap() }.into()))
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
    })
}


unsafe extern "C" fn register_natives(env: *mut JNIEnv,
                                      clazz: jclass,
                                      methods: *const JNINativeMethod,
                                      n_methods: jint) -> jint {
    trace!("Call to register_natives, n_methods: {}", n_methods);
    for to_register_i in 0..n_methods {
        let jni_context = &get_state(env).jni;
        let method = *methods.offset(to_register_i as isize);
        let expected_name: String = CStr::from_ptr(method.name).to_str().unwrap().to_string().clone();
        let descriptor: String = CStr::from_ptr(method.signature).to_str().unwrap().to_string().clone();
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


fn register_native_with_lib_java_loading(jni_context: &LibJavaLoading, method: &JNINativeMethod, runtime_class: &Arc<RuntimeClass>, method_i: usize) -> () {
    if jni_context.registered_natives.borrow().contains_key(runtime_class) {
        unsafe {
            jni_context.registered_natives
                .borrow()
                .get(runtime_class)
                .unwrap()
                .borrow_mut()
                .insert(method_i as CPIndex, transmute(method.fnPtr));
        }
    } else {
        let mut map = HashMap::new();
        map.insert(method_i as CPIndex, unsafe { transmute(method.fnPtr) });
        jni_context.registered_natives.borrow_mut().insert(runtime_class.clone(), RefCell::new(map));
    }
}


unsafe extern "C" fn release_string_utfchars(_env: *mut JNIEnv, _str: jstring, chars: *const c_char) {
    let len = libc::strlen(chars);
    let chars_layout = Layout::from_size_align((len + 1) * size_of::<c_char>(), size_of::<c_char>()).unwrap();
    std::alloc::dealloc(chars as *mut u8, chars_layout);
}

unsafe extern "C" fn exception_check(_env: *mut JNIEnv) -> jboolean {
    false as jboolean//todo exceptions are not needed for hello world so if we encounter an exception we just pretend it didn't happen
}

pub fn get_all_methods(state: &mut InterpreterState, frame: Rc<CallStackEntry>, class: Arc<RuntimeClass>) -> Vec<(Arc<RuntimeClass>, usize)> {
    let mut res = vec![];
    class.classfile.methods.iter().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });
    if class.classfile.super_class == 0 {
        let object = check_inited_class(state, &ClassName::Str("java/lang/Object".to_string()), frame.clone().into(), class.loader.clone());
        object.classfile.methods.iter().enumerate().for_each(|(i, _)| {
            res.push((object.clone(), i));
        });
    } else {
        let name = get_super_class_name(&class.classfile);
        let super_ = check_inited_class(state, &name, frame.clone().into(), class.loader.clone());
        for (c, i) in get_all_methods(state, frame, super_) {
            res.push((c, i));//todo accidental O(n^2)
        }
    }

    return res;
}

//for now a method id is a pair of class pointers and i.
//turns out this is for member functions only
//see also get_static_method_id
unsafe extern "C" fn get_method_id(env: *mut JNIEnv,
                                   clazz: jclass,
                                   name: *const c_char,
                                   sig: *const c_char)
                                   -> jmethodID {
    let name_len = libc::strlen(name);
    let mut method_name = String::with_capacity(name_len);
    for i in 0..name_len {
        method_name.push(name.offset(i as isize).read() as u8 as char);
    }

    let desc_len = libc::strlen(sig);
    //todo dup
    let mut method_descriptor_str = String::with_capacity(desc_len);
    for i in 0..desc_len {
        method_descriptor_str.push(sig.offset(i as isize).read() as u8 as char);
    }

    let state = get_state(env);
    let frame = get_frame(env);//todo leak hazard
    let class_obj: Arc<Object> = from_object(clazz).unwrap();
    let all_methods = get_all_methods(state, frame, class_obj.object_class_object_pointer.borrow().as_ref().unwrap().clone());
    let (_method_i, (c, m)) = all_methods.iter().enumerate().find(|(_, (c, i))| {
        let method_info = &c.classfile.methods[*i];
        let cur_desc = extract_string_from_utf8(&c.classfile.constant_pool[method_info.descriptor_index as usize]);
        let cur_method_name = rust_jvm_common::utils::method_name(&c.classfile, method_info);
//        dbg!(&method_name);
//        dbg!(&cur_method_name);
        cur_method_name == method_name &&
            method_descriptor_str == cur_desc
    }).unwrap();
    let res = Box::into_raw(Box::new(MethodId { class: c.clone(), method_i: *m }));
    transmute(res)
}

pub struct MethodId {
    pub class: Arc<RuntimeClass>,
    pub method_i: usize,
}

pub mod native_util;
pub mod interface;
pub mod string;
pub mod dlopen;