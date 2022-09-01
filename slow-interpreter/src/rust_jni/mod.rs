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

use another_jit_vm_ir::WasException;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jchar, jobject, jshort};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::{JavaValueCommon, JVMState, NewJavaValue};
use crate::better_java_stack::native_frame::NativeFrame;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::jvm_state::NativeLibraries;
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::ffi_arg_holder::ArgBoxesToFree;
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::native_util::{from_object_new};
use crate::rust_jni::value_conversion::{to_native, to_native_type};

pub mod mangling;
pub mod value_conversion;
pub mod ffi_arg_holder;

impl<'gc> NativeLibraries<'gc> {
    pub fn new(libjava: OsString) -> NativeLibraries<'gc> {
        NativeLibraries {
            libjava_path: libjava,
            native_libs: Default::default(),
            registered_natives: RwLock::new(HashMap::new()),
        }
    }
}

pub fn call<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut NativeFrame<'gc, 'l>, classfile: Arc<RuntimeClass<'gc>>, method_view: MethodView, args: Vec<NewJavaValue<'gc, 'k>>, md: CMethodDescriptor) -> Result<Option<Option<NewJavaValueHandle<'gc>>>, WasException> {
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

    Ok(if method_view.is_static() {
        Some(call_impl(jvm, int_state, classfile, args, md, &raw, false)?)
    } else {
        Some(call_impl(jvm, int_state, classfile, args, md, &raw, true)?)
    })
}

pub fn call_impl<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut NativeFrame<'gc, 'l>,
    classfile: Arc<RuntimeClass<'gc>>,
    mut args: Vec<NewJavaValue<'gc, 'k>>,
    md: CMethodDescriptor,
    raw: &unsafe extern "C" fn(),
    suppress_runtime_class: bool,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    args.retain(|arg| !matches!(arg,NewJavaValue::Top));
    int_state.debug_assert();
    let mut args_type = if suppress_runtime_class { vec![Type::pointer()] } else { vec![Type::pointer(), Type::pointer()] };
    let env = get_interface(jvm, int_state);
    let mut arg_boxes = ArgBoxesToFree::new();
    let mut c_args = if suppress_runtime_class {
        vec![Arg::new(&env)]
    } else {
        int_state.debug_assert();
        let class_popped_jv = load_class_constant_by_type(jvm, int_state, classfile.view().type_())?;
        int_state.debug_assert();
        let class_constant = unsafe { to_native(env, &mut arg_boxes, class_popped_jv.as_njv(), &Into::<CPDType>::into(CClassName::object())) };
        let res = vec![Arg::new(&env), class_constant];
        res
    };
    //todo inconsistent use of class and/pr arc<RuntimeClass>

    let temp_vec = vec![CClassName::object().into()];
    let args_and_type = if suppress_runtime_class {
        assert_eq!(args.len(), md.arg_types.len() + 1);
        args.iter().zip(temp_vec.iter().chain(md.arg_types.iter())).collect::<Vec<_>>()
    } else {
        args.iter().zip(md.arg_types.iter()).collect::<Vec<_>>()
    };
    for (j, t) in args_and_type.iter() {
        args_type.push(to_native_type(&t));
        unsafe {
            c_args.push(to_native(env, &mut arg_boxes, (*j).clone(), &t));
        }
    }
    let cif = Cif::new(args_type.into_iter(), match &md.return_type {
        CompressedParsedDescriptorType::BooleanType => Type::u8(),
        CompressedParsedDescriptorType::ByteType => Type::i8(),
        CompressedParsedDescriptorType::ShortType => Type::i16(),
        CompressedParsedDescriptorType::CharType => Type::u16(),
        CompressedParsedDescriptorType::IntType => Type::i32(),
        CompressedParsedDescriptorType::LongType => Type::i64(),
        CompressedParsedDescriptorType::FloatType => Type::f32(),
        CompressedParsedDescriptorType::DoubleType => Type::f64(),
        CompressedParsedDescriptorType::VoidType => Type::void(),
        CompressedParsedDescriptorType::Class(_) => Type::pointer(),
        CompressedParsedDescriptorType::Array { .. } => Type::pointer()
    });
    let fn_ptr = CodePtr::from_fun(*raw);
    unsafe { assert!(jvm.get_int_state().registered()); }
    assert!(jvm.thread_state.int_state_guard_valid.with(|valid| valid.borrow().clone()));
    let cif_res: *mut c_void = unsafe { cif.call(fn_ptr, c_args.as_slice()) };
    let res = match &md.return_type {
        CPDType::VoidType => None,
        CPDType::ByteType => Some(NewJavaValueHandle::Byte(cif_res as i8)),
        CPDType::FloatType => Some(NewJavaValueHandle::Float(unsafe { transmute(cif_res as usize as u32) })),
        CPDType::DoubleType => Some(NewJavaValueHandle::Double(unsafe { transmute(cif_res as u64) })),
        CPDType::ShortType => Some(NewJavaValueHandle::Short(cif_res as jshort)),
        CPDType::CharType => Some(NewJavaValueHandle::Char(cif_res as jchar)),
        CPDType::IntType => Some(NewJavaValueHandle::Int(cif_res as i32)),
        CPDType::LongType => Some(NewJavaValueHandle::Long(cif_res as i64)),
        CPDType::BooleanType => Some(NewJavaValueHandle::Boolean(cif_res as u8)),
        CPDType::Class(_) | CPDType::Array { .. } => {
            Some(unsafe {
                match from_object_new(jvm, cif_res as jobject) {
                    None => {
                        NewJavaValueHandle::Null
                    }
                    Some(obj) => {
                        NewJavaValueHandle::Object(obj)
                    }
                }
            })
        }
    };
    todo!();
    /*if int_state.throw().is_some() {
        return Err(WasException {});
    }*/
    Ok(res)
}

pub mod dlopen;
pub mod interface;
pub mod native_util;
pub mod stdarg;