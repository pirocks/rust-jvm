extern crate libc;
extern crate libloading;

use std::collections::HashMap;
use std::ffi::OsString;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_void;
use std::sync::{Arc, RwLock};

use libffi::middle::Arg;
use libffi::middle::Cif;
use libffi::middle::CodePtr;
use libffi::middle::Type;
use libloading::Symbol;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jchar, jobject, jshort};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter::WasException;
use crate::java_values::JavaValue;
use crate::jvm_state::NativeLibraries;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::native_util::from_object;
use crate::rust_jni::value_conversion::{free_native, to_native, to_native_type};

pub mod mangling;
pub mod value_conversion;

impl<'gc_life> NativeLibraries<'gc_life> {
    pub fn new(libjava: OsString) -> NativeLibraries<'gc_life> {
        NativeLibraries {
            libjava_path: libjava,
            native_libs: Default::default(),
            registered_natives: RwLock::new(HashMap::new()),
        }
    }
}

pub fn call<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, classfile: Arc<RuntimeClass<'gc_life>>, method_view: MethodView, args: Vec<JavaValue<'gc_life>>, md: CMethodDescriptor) -> Result<Option<Option<JavaValue<'gc_life>>>, WasException> {
    let mangled = mangling::mangle(&jvm.string_pool, &method_view);
    // dbg!(&mangled);
    let raw: unsafe extern "C" fn() = unsafe {
        let libraries_guard = jvm.native_libaries.native_libs.read().unwrap();
        let possible_symbol = libraries_guard.values().find_map(|native_lib| native_lib.library.get(&mangled.as_bytes()).ok());
        match possible_symbol {
            Some(symbol) => {
                let symbol: Symbol<unsafe extern "C" fn()> = symbol;
                *symbol.deref()
            }
            None => {
                return Ok(None);
            }
        }
    };

    Ok(if method_view.is_static() { Some(call_impl(jvm, int_state, classfile, args, md, &raw, false)?) } else { Some(call_impl(jvm, int_state, classfile, args, md, &raw, true)?) })
}

pub fn call_impl<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, classfile: Arc<RuntimeClass<'gc_life>>, args: Vec<JavaValue<'gc_life>>, md: CMethodDescriptor, raw: &unsafe extern "C" fn(), suppress_runtime_class: bool) -> Result<Option<JavaValue<'gc_life>>, WasException> {
    let mut args_type = if suppress_runtime_class { vec![Type::pointer()] } else { vec![Type::pointer(), Type::pointer()] };
    let env = get_interface(jvm, int_state);
    let mut c_args = if suppress_runtime_class {
        vec![Arg::new(&env)]
    } else {
        let class_popped_jv = load_class_constant_by_type(jvm, int_state, &classfile.view().type_())?;
        let class_constant = unsafe { to_native(env, class_popped_jv, &Into::<CPDType>::into(CClassName::object())) };
        let res = vec![Arg::new(&env), class_constant];
        res
    };
    //todo inconsistent use of class and/pr arc<RuntimeClass>

    let temp_vec = vec![CPDType::Ref(CPRefType::Class(CClassName::object()))];
    let args_and_type = if suppress_runtime_class { args.iter().zip(temp_vec.iter().chain(md.arg_types.iter())).collect::<Vec<_>>() } else { args.iter().zip(md.arg_types.iter()).collect::<Vec<_>>() };
    for (j, t) in args_and_type.iter() {
        args_type.push(to_native_type(&t));
        unsafe {
            c_args.push(to_native(env, (*j).clone(), &t));
        }
    }
    let cif = Cif::new(args_type.into_iter(), Type::usize());
    let fn_ptr = CodePtr::from_fun(*raw);
    let cif_res: *mut c_void = unsafe { cif.call(fn_ptr, c_args.as_slice()) };
    let res = match &md.return_type {
        CPDType::VoidType => None,
        CPDType::ByteType => Some(JavaValue::Byte(cif_res as i8)),
        CPDType::FloatType => Some(JavaValue::Float(unsafe { transmute(cif_res as usize as u32) })),
        CPDType::DoubleType => Some(JavaValue::Double(unsafe { transmute(cif_res as u64) })),
        CPDType::ShortType => Some(JavaValue::Short(cif_res as jshort)),
        CPDType::CharType => Some(JavaValue::Char(cif_res as jchar)),
        CPDType::IntType => Some(JavaValue::Int(cif_res as i32)),
        CPDType::LongType => Some(JavaValue::Long(cif_res as i64)),
        CPDType::BooleanType => Some(JavaValue::Boolean(cif_res as u8)),
        CPDType::Ref(_) => unsafe { Some(JavaValue::Object(from_object(jvm, cif_res as jobject))) },
        _ => {
            dbg!(md.return_type); //todo
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

pub mod dlopen;
pub mod interface;
pub mod native_util;
pub mod stdarg;