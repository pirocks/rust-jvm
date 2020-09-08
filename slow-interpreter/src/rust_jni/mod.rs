extern crate libc;
extern crate libloading;

use std::alloc::Layout;
use std::collections::{HashMap, HashSet};
use std::ffi::CStr;
use std::io::Error;
use std::mem::size_of;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::{c_char, c_void};
use std::sync::{Arc, RwLock};

use libffi::middle::Arg;
use libffi::middle::Cif;
use libffi::middle::CodePtr;
use libffi::middle::Type;
use libloading::Library;
use libloading::Symbol;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use jvmti_jni_bindings::{jboolean, jclass, jint, jmethodID, JNIEnv, JNINativeMethod, jstring};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState, LibJavaLoading};
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::interface::util::class_object_to_runtime_class;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use crate::rust_jni::value_conversion::{to_native, to_native_type};

pub mod value_conversion;
pub mod mangling;

impl LibJavaLoading {
    pub fn new_java_loading(path: String) -> LibJavaLoading {
        // trace!("Loading libjava.so from:`{}`", path);
//    crate::rust_jni::libloading::os::unix::Library::open("libjvm.so".into(), (dlopen::RTLD_NOW | dlopen::RTLD_GLOBAL).try_into().unwrap()).unwrap();
//    let loaded = crate::rust_jni::libloading::os::unix::Library::open(path.clone().into(), (dlopen::RTLD_NOW /*| dlopen::RTLD_GLOBAL*/).try_into().unwrap()).unwrap();
        let lib = Library::new(path.clone()).unwrap();
        let nio_path = path.replace("libjava.so", "libnio.so");
        let nio_lib = Library::new(nio_path).unwrap();
//    let lib = Library::from(loaded);
        LibJavaLoading {
            libjava: lib,
            libnio: nio_lib,
            registered_natives: RwLock::new(HashMap::new()),
        }
    }
}


pub fn call<'l>(
    state: &'static JVMState,
    int_state: &mut InterpreterStateGuard,
    classfile: Arc<RuntimeClass>,
    method_i: usize,
    args: Vec<JavaValue>,
    md: MethodDescriptor,
) -> Result<Option<JavaValue>, Error> {
    let mangled = mangling::mangle(classfile.clone(), method_i);
    let raw = {
        let symbol: Symbol<unsafe extern fn()> = unsafe {
            match state.libjava.libjava.get(mangled.clone().as_bytes()) {
                Ok(o) => o,
                Err(_) => {
                    match state.libjava.libnio.get(mangled.clone().as_bytes()) {
                        Ok(o) => o,
                        Err(e) => {
                            return Result::Err(e);
                        }
                    }
                }
            }
        };
        symbol.deref().clone()
    };
    if classfile.view().method_view_i(method_i).is_static() {
        Result::Ok(call_impl(state, int_state, classfile, args, md, &raw, false, false))
    } else {
        Result::Ok(call_impl(state, int_state, classfile, args, md, &raw, true, false))
    }
}

pub fn call_impl<'l>(
    jvm: &'static JVMState,
    int_state: &mut InterpreterStateGuard,
    classfile: Arc<RuntimeClass>,
    args: Vec<JavaValue>,
    md: MethodDescriptor,
    raw: &unsafe extern "C" fn(),
    suppress_runtime_class: bool,
    _debug: bool,
) -> Option<JavaValue> {
    let mut args_type = if suppress_runtime_class {
        vec![Type::pointer()]
    } else {
        vec![Type::pointer(), Type::pointer()]
    };
    let env = get_interface(jvm, int_state);
    let mut c_args = if suppress_runtime_class {
        vec![Arg::new(&env)]
    } else {
        load_class_constant_by_type(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Class(classfile.view().name())));
        let res = vec![Arg::new(&env), to_native(int_state.pop_current_operand_stack(), &PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())).to_ptype())];
        res
    };
//todo inconsistent use of class and/pr arc<RuntimeClass>

    if suppress_runtime_class {
        for (j, t) in args
            .iter()
            .zip(vec![PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())).to_ptype()]
                .iter()
                .chain(md.parameter_types.iter())) {
            args_type.push(to_native_type(&t));
            c_args.push(to_native(j.clone(), &t));
        }
    } else {
        for (j, t) in args.iter().zip(md.parameter_types.iter()) {
            args_type.push(to_native_type(&t));
            c_args.push(to_native(j.clone(), &t));
        }
    }
    let cif = Cif::new(args_type.into_iter(), Type::usize());
//todo what if float
    let fn_ptr = CodePtr::from_fun(*raw);
    // trace!("----NATIVE ENTER----");
    let cif_res: *mut c_void = unsafe {
        cif.call(fn_ptr, c_args.as_slice())
    };
    // trace!("----NATIVE EXIT ----");
    match PTypeView::from_ptype(&md.return_type) {
        PTypeView::VoidType => {
            None
        }
//            ParsedType::ByteType => {}
//            ParsedType::CharType => {}
        PTypeView::DoubleType => {
            Some(JavaValue::Double(unsafe { transmute(cif_res) }))
        }
//            ParsedType::FloatType => {}
        PTypeView::IntType => {
            Some(JavaValue::Int(cif_res as i32))
        }
        PTypeView::LongType => {
            Some(JavaValue::Long(cif_res as i64))
        }
//            ParsedType::ShortType => {}
        PTypeView::BooleanType => {
            Some(JavaValue::Boolean(cif_res as u8))
        }
        PTypeView::Ref(_) => {
            unsafe {
                Some(JavaValue::Object(from_object(transmute(cif_res))))
            }
        }
//            ParsedType::TopType => {}
//            ParsedType::NullType => {}
//            ParsedType::Uninitialized(_) => {}
//            ParsedType::UninitializedThis => {}
        _ => {
            dbg!(md.return_type);//todo
            panic!()
        }
    }
}


unsafe extern "C" fn register_natives(env: *mut JNIEnv,
                                      clazz: jclass,
                                      methods: *const JNINativeMethod,
                                      n_methods: jint) -> jint {
    // println!("Call to register_natives, n_methods: {}", n_methods);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    int_state.print_stack_trace();
    for to_register_i in 0..n_methods {
        let method = *methods.offset(to_register_i as isize);
        let expected_name: String = CStr::from_ptr(method.name).to_str().unwrap().to_string().clone();
        let descriptor: String = CStr::from_ptr(method.signature).to_str().unwrap().to_string().clone();
        let runtime_class: Arc<RuntimeClass> = from_jclass(clazz).as_runtime_class();
        let jni_context = &jvm.libjava;
        let view = &runtime_class.view();
        &view.methods().enumerate().for_each(|(i, method_info)| {
            let descriptor_str = method_info.desc_str();
            let current_name = method_info.name();
            if current_name == expected_name && descriptor == descriptor_str {
                jvm.tracing.trace_jni_register(&view.name(), expected_name.as_str());
                register_native_with_lib_java_loading(jni_context, &method, &runtime_class, i)
            }
        });
    }
    0
}


fn register_native_with_lib_java_loading(jni_context: &LibJavaLoading, method: &JNINativeMethod, runtime_class: &Arc<RuntimeClass>, method_i: usize) -> () {
    if jni_context.registered_natives.read().unwrap().contains_key(runtime_class) {
        unsafe {
            jni_context.registered_natives
                .read().unwrap()
                .get(runtime_class)
                .unwrap()
                .write().unwrap()
                .insert(method_i as CPIndex, transmute(method.fnPtr));
        }
    } else {
        let mut map = HashMap::new();
        map.insert(method_i as CPIndex, unsafe { transmute(method.fnPtr) });
        jni_context.registered_natives.write().unwrap().insert(runtime_class.clone(), RwLock::new(map));
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

pub fn get_all_methods<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>) -> Vec<(Arc<RuntimeClass>, usize)> {
    let mut res = vec![];
    // dbg!(&class.class_view.name());
    class.view().methods().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });
    if class.view().super_name().is_none() {
        let object = check_inited_class(jvm, int_state, &ClassName::object().into(), class.loader(jvm).clone());
        object.view().methods().enumerate().for_each(|(i, _)| {
            res.push((object.clone(), i));
        });
    } else {
        let name = class.view().super_name().unwrap();
        let super_ = check_inited_class(jvm, int_state, &name.into(), class.loader(jvm).clone());
        for (c, i) in get_all_methods(jvm, int_state, super_) {
            res.push((c, i));
        }
    }

    res
}

//todo duplication with methods
pub fn get_all_fields<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>) -> Vec<(Arc<RuntimeClass>, usize)> {
    let mut res = vec![];
    class.view().fields().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });
    if class.view().super_name().is_none() {
        let object = check_inited_class(jvm, int_state, &ClassName::object().into(), class.loader(jvm).clone());
        object.view().fields().enumerate().for_each(|(i, _)| {
            res.push((object.clone(), i));
        });
    } else {
        let name = class.view().super_name();
        let super_ = check_inited_class(jvm, int_state, &name.unwrap().into(), class.loader(jvm).clone());
        for (c, i) in get_all_fields(jvm, int_state, super_) {
            res.push((c, i));//todo accidental O(n^2)
        }
    }

    res
}


//for now a method id is a pair of class pointers and i.
//turns out this is for member functions only
//see also get_static_method_id
unsafe extern "C" fn get_method_id(env: *mut JNIEnv,
                                   clazz: jclass,
                                   name: *const c_char,
                                   sig: *const c_char)
                                   -> jmethodID {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
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

    let class_obj = from_object(clazz);
    let runtime_class = class_object_to_runtime_class(&JavaValue::Object(class_obj).cast_class(), jvm, int_state).unwrap();
    let all_methods = get_all_methods(jvm, int_state, runtime_class);

    let (_method_i, (c, m)) = all_methods.iter().enumerate().find(|(_, (c, i))| {
        let method_view = &c.view().method_view_i(*i);
        let cur_desc = method_view.desc_str();
        let cur_method_name = method_view.name();
        cur_method_name == method_name &&
            method_descriptor_str == cur_desc
    }).unwrap();
    let method_id = jvm.method_table.write().unwrap().get_method_id(c.clone(), *m as u16);
    transmute(method_id)
}

// #[derive(Clone, Hash, Eq, PartialEq)]
// pub struct MethodId {
//     pub class: Arc<RuntimeClass>,
//     pub method_i: usize,
// }
//
// impl Debug for MethodId{
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(),std::fmt::Error> {
//         let method_view = self.class.view().method_view_i(self.method_i);
//         write!(f, "{:?} {} {}",self.class.view().name(), method_view.name(),method_view.desc_str())
//     }
// }

pub mod native_util;
pub mod interface;
pub mod dlopen;
pub mod stdarg;