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
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jchar, jobject, jshort};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::MethodDescriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter::WasException;
use crate::java_values::JavaValue;
use crate::jvm_state::LibJavaLoading;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::dlopen::{RTLD_GLOBAL, RTLD_LAZY};
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::native_util::from_object;
use crate::rust_jni::value_conversion::{free_native, to_native, to_native_type};

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
        let nio_lib = Library::new(nio_path, (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap(); //todo make these expects
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
    method_view: MethodView,
    args: Vec<JavaValue>,
    md: MethodDescriptor,
) -> Result<Result<Option<JavaValue>, libloading::Error>, WasException> {
    let mangled = mangling::mangle(&method_view);
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
                                                            return Ok(Err(e));
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
    Ok(if method_view.is_static() {
        Ok(call_impl(state, int_state, classfile, args, md, &raw, false)?)
    } else {
        Ok(call_impl(state, int_state, classfile, args, md, &raw, true)?)
    })
}

pub fn call_impl(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    classfile: Arc<RuntimeClass>,
    args: Vec<JavaValue>,
    md: MethodDescriptor,
    raw: &unsafe extern "C" fn(),
    suppress_runtime_class: bool) -> Result<Option<JavaValue>, WasException> {
    let mut args_type = if suppress_runtime_class {
        vec![Type::pointer()]
    } else {
        vec![Type::pointer(), Type::pointer()]
    };
    let env = get_interface(jvm, int_state);
    let mut c_args = if suppress_runtime_class {
        vec![Arg::new(&env)]
    } else {
        load_class_constant_by_type(jvm, int_state, classfile.view().type_())?;
        let res = vec![Arg::new(&env), unsafe { to_native(int_state.pop_current_operand_stack(), &PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())).to_ptype()) }];
        res
    };
//todo inconsistent use of class and/pr arc<RuntimeClass>

    let temp_vec = vec![PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())).to_ptype()];
    let args_and_type = if suppress_runtime_class {
        args
            .iter()
            .zip(temp_vec
                .iter()
                .chain(md.parameter_types.iter())).collect::<Vec<_>>()
    } else {
        args.iter().zip(md.parameter_types.iter()).collect::<Vec<_>>()
    };
    for (j, t) in args_and_type.iter() {
        args_type.push(to_native_type(&t));
        unsafe { c_args.push(to_native((*j).clone(), &t)); }
    }
    let cif = Cif::new(args_type.into_iter(), Type::usize());
    let fn_ptr = CodePtr::from_fun(*raw);
    let cif_res: *mut c_void = unsafe {
        cif.call(fn_ptr, c_args.as_slice())
    };
    let res = match PTypeView::from_ptype(&md.return_type) {
        PTypeView::VoidType => {
            None
        }
        PTypeView::ByteType => {
            Some(JavaValue::Byte(cif_res as i8))
        }
        PTypeView::FloatType => {
            Some(JavaValue::Float(unsafe { transmute(cif_res as usize as u32) }))
        }
        PTypeView::DoubleType => {
            Some(JavaValue::Double(unsafe { transmute(cif_res as u64) }))
        }
        PTypeView::ShortType => {
            Some(JavaValue::Short(cif_res as jshort))
        }
        PTypeView::CharType => {
            Some(JavaValue::Char(cif_res as jchar))
        }
        PTypeView::IntType => {
            Some(JavaValue::Int(cif_res as i32))
        }
        PTypeView::LongType => {
            Some(JavaValue::Long(cif_res as i64))
        }
        PTypeView::BooleanType => {
            Some(JavaValue::Boolean(cif_res as u8))
        }
        PTypeView::Ref(_) => {
            unsafe {
                Some(JavaValue::Object(from_object(cif_res as jobject)))
            }
        }
        _ => {
            dbg!(md.return_type);//todo
            panic!()
        }
    };
    unsafe {
        for (i, (j, t)) in args_and_type.iter().enumerate() {
            let offset = if suppress_runtime_class { 1 } else { 2 };
            let to_free = &mut c_args[i + offset];
            free_native((*j).clone(), t, to_free)
        }
    }
    if int_state.throw().is_some() {
        return Err(WasException {});
    }
    Ok(res)
}

pub mod native_util;
pub mod interface;
pub mod dlopen;
pub mod stdarg;