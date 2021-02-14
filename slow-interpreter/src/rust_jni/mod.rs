extern crate libc;
extern crate libloading;

use std::collections::HashMap;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_void;
use std::sync::{Arc, RwLock};

use libffi::middle::Arg;
use libffi::middle::Cif;
use libffi::middle::CodePtr;
use libffi::middle::Type;
use libloading::Library;
use libloading::os::unix::RTLD_NOW;
use libloading::Symbol;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use jvmti_jni_bindings::jobject;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::ldc::load_class_constant_by_type;
use crate::java_values::JavaValue;
use crate::jvm_state::LibJavaLoading;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::dlopen::{RTLD_GLOBAL, RTLD_LAZY};
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::native_util::from_object;
use crate::rust_jni::value_conversion::{to_native, to_native_type};

pub mod value_conversion;
pub mod mangling;

impl LibJavaLoading {
    pub fn new_java_loading(path: String) -> LibJavaLoading {
        let lib = Library::new(path.clone(), (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
        let nio_path = path.replace("libjava.so", "libnio.so");
        let awt_path = path.replace("libjava.so", "libawt.so");
        let xawt_path = path.replace("libjava.so", "libawt_xawt.so");
        let zip_path = path.replace("libjava.so", "libzip.so");
        let libfontmanager_path = path.replace("libjava.so", "libfontmanager.so");
        let nio_lib = Library::new(nio_path, (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
        let libawt = Library::new(awt_path, (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
        let libxawt = Library::new(xawt_path, (RTLD_NOW | RTLD_GLOBAL as i32) as i32).unwrap();
        let libzip = Library::new(zip_path, (RTLD_NOW | RTLD_GLOBAL as i32) as i32).unwrap();
        let libfontmanager = Library::new(libfontmanager_path, (RTLD_NOW | RTLD_GLOBAL as i32) as i32).unwrap();
        LibJavaLoading {
            libjava: lib,
            libnio: nio_lib,
            libawt,
            libxawt,
            libzip,
            libfontmanager,
            registered_natives: RwLock::new(HashMap::new()),
        }
    }
}


pub fn call(
    state: &JVMState,
    int_state: &mut InterpreterStateGuard,
    classfile: Arc<RuntimeClass>,
    method_i: usize,
    args: Vec<JavaValue>,
    md: MethodDescriptor,
) -> Result<Option<JavaValue>, libloading::Error> {
    let mangled = mangling::mangle(classfile.clone(), method_i);
    let raw = {
        let symbol: Symbol<unsafe extern fn()> = unsafe {
            match state.libjava.libjava.get(mangled.as_bytes()) {
                Ok(o) => o,
                Err(_) => {
                    match state.libjava.libnio.get(mangled.as_bytes()) {
                        Ok(o) => o,
                        Err(_) => {
                            match state.libjava.libawt.get(mangled.as_bytes()) {
                                Ok(o) => o,
                                Err(_) => {
                                    //todo maybe do something about this nesting lol
                                    match state.libjava.libxawt.get(mangled.as_bytes()) {
                                        Ok(o) => o,
                                        Err(_) => {
                                            //todo maybe do something about this nesting lol
                                            match state.libjava.libzip.get(mangled.as_bytes()) {
                                                Ok(o) => o,
                                                Err(_) => {
                                                    match state.libjava.libfontmanager.get(mangled.as_bytes()) {
                                                        Ok(o) => o,
                                                        Err(e) => {
                                                            return Result::Err(e);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        *symbol.deref()
    };
    if classfile.view().method_view_i(method_i).is_static() {
        Result::Ok(call_impl(state, int_state, classfile, args, md, &raw, false))
    } else {
        Result::Ok(call_impl(state, int_state, classfile, args, md, &raw, true))
    }
}

pub fn call_impl(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    classfile: Arc<RuntimeClass>,
    args: Vec<JavaValue>,
    md: MethodDescriptor,
    raw: &unsafe extern "C" fn(),
    suppress_runtime_class: bool) -> Option<JavaValue> {
    let mut args_type = if suppress_runtime_class {
        vec![Type::pointer()]
    } else {
        vec![Type::pointer(), Type::pointer()]
    };
    let env = get_interface(jvm, int_state);
    let mut c_args = if suppress_runtime_class {
        vec![Arg::new(&env)]
    } else {
        load_class_constant_by_type(jvm, int_state, PTypeView::Ref(ReferenceTypeView::Class(classfile.view().name())));
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
    // int_state.print_stack_trace();
    let cif_res: *mut c_void = unsafe {
        cif.call(fn_ptr, c_args.as_slice())
    };
    // trace!("----NATIVE EXIT ----");
    match PTypeView::from_ptype(&md.return_type) {
        PTypeView::VoidType => {
            None
        }
        PTypeView::ByteType => {
            Some(JavaValue::Byte(cif_res as usize as i8))//todo is this correct?
        }
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
                Some(JavaValue::Object(from_object(cif_res as jobject)))
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

pub mod native_util;
pub mod interface;
pub mod dlopen;
pub mod stdarg;