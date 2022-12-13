use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_char;
use std::ptr::null_mut;

use by_address::ByAddress;
use nix::sys::aio::aio_suspend;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject, jstring, JVM_Available};
use rust_jvm_common::classfile::AttributeType::RuntimeInvisibleAnnotations;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::class_names::CompressedClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::loading::LoaderName::BootstrapLoader;
use rust_jvm_common::ptype::PType::Ref;
use rust_jvm_common::runtime_type::RuntimeType;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::better_java_stack::opaque_frame::OpaqueFrame;
use slow_interpreter::class_loading::bootstrap_load;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::{ExceptionReturn, JavaValue};


use slow_interpreter::rust_jni::jni_utils::{get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object, to_object_new};
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::utils::{pushable_frame_todo, throw_npe};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromBootLoader<'gc, 'l>(env: *mut JNIEnv, name: *const c_char) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_str = CStr::from_ptr(name).to_str().unwrap().to_string(); //todo handle utf8 here
    //todo duplication
    let class_name = CompressedClassName(jvm.string_pool.add_name(name_str, true));

    let loader_obj = int_state.frame_iter().next()/*previous_frame()*/.unwrap().local_get_handle(0, RuntimeType::object()).cast_class_loader();
    let current_loader = loader_obj.to_jvm_loader(jvm);
    let guard = jvm.classes.write().unwrap();
    let runtime_class = match guard.loaded_classes_by_type.get(&BootstrapLoader).unwrap().get(&class_name.clone().into()) {
        None => {
            drop(guard);
            let runtime_class = match bootstrap_load(jvm, int_state, class_name.into()) {
                Ok(x) => x,
                Err(WasException { exception_obj }) => {
                    *get_throw(env) = Some(WasException { exception_obj });
                    return ExceptionReturn::invalid_default();
                }
            };
            let ptype = runtime_class.cpdtype();
            let mut guard = jvm.classes.write().unwrap();
            guard.initiating_loaders.entry(ptype.clone()).or_insert((BootstrapLoader, runtime_class.clone())); //todo wrong loader?
            guard.loaded_classes_by_type.entry(BootstrapLoader).or_insert(HashMap::new()).insert(ptype, runtime_class.clone());
            runtime_class
        }
        Some(runtime_class) => runtime_class.clone(),
    };
    let guard = jvm.classes.write().unwrap();
    to_object_new(guard.get_class_obj_from_runtime_class(runtime_class.clone()).as_allocated_obj().into())
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClassLoader(env: *mut JNIEnv, name: *const c_char, init: jboolean, loader: jobject, throwError: jboolean) -> jclass {
    dbg!(CStr::from_ptr(name).to_str().unwrap());
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClass(env: *mut JNIEnv, name: *const c_char, init: jboolean, from: jclass) -> jclass {
    dbg!(CStr::from_ptr(name).to_str().unwrap());
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLoadedClass(env: *mut JNIEnv, loader: jobject, name: jstring) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_str = match from_object_new(jvm, name) {
        None => return throw_npe(jvm, int_state,get_throw(env)),
        Some(name_str) => name_str.cast_string(),
    }
        .to_rust_string(jvm);
    assert_ne!(&name_str, "int");
    // dbg!(&name_str);
    //todo what if not bl
    let class_name = CompressedClassName(jvm.string_pool.add_name(name_str.replace(".", "/"), true));
    let loaded = jvm.classes.write().unwrap().is_loaded(&class_name.clone().into());
    match loaded {
        None => null_mut(),
        Some(view) => {
            // todo what if name is long/int etc.
            let res = get_or_create_class_object(jvm, class_name.into(), int_state).unwrap(); //todo handle exception
            new_local_ref_public_new(res.as_allocated_obj().into(), int_state)
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_FindPrimitiveClass(env: *mut JNIEnv, utf: *const c_char) -> jclass {
    assert_ne!(utf, std::ptr::null());
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let float = CStr::from_bytes_with_nul(b"float\0").unwrap(); //todo make these expects
    let double = CStr::from_bytes_with_nul(b"double\0").unwrap();
    let int = CStr::from_bytes_with_nul(b"int\0").unwrap();
    let boolean = CStr::from_bytes_with_nul(b"boolean\0").unwrap();
    let char_ = CStr::from_bytes_with_nul(b"char\0").unwrap();
    let long = CStr::from_bytes_with_nul(b"long\0").unwrap();
    let byte = CStr::from_bytes_with_nul(b"byte\0").unwrap();
    let short = CStr::from_bytes_with_nul(b"short\0").unwrap();
    let void = CStr::from_bytes_with_nul(b"void\0").unwrap();
    let utf_input = CStr::from_ptr(utf);
    let (class_name, as_str, ptype) = if utf_input == float {
        (ClassName::raw_float(), "float", CPDType::FloatType)
    } else if utf_input == double {
        (ClassName::raw_double(), "double", CPDType::DoubleType)
    } else if utf_input == int {
        (ClassName::raw_int(), "int", CPDType::IntType)
    } else if utf_input == boolean {
        (ClassName::raw_boolean(), "boolean", CPDType::BooleanType)
    } else if utf_input == char_ {
        (ClassName::raw_char(), "char", CPDType::CharType)
    } else if utf_input == long {
        (ClassName::raw_long(), "long", CPDType::LongType)
    } else if utf_input == byte {
        (ClassName::raw_byte(), "byte", CPDType::ByteType)
    } else if utf_input == short {
        (ClassName::raw_short(), "short", CPDType::ShortType)
    } else if utf_input == void {
        (ClassName::raw_void(), "void", CPDType::VoidType)
    } else {
        dbg!(utf_input);
        int_state.debug_print_stack_trace(jvm);
        unimplemented!()
    };

    let res = get_or_create_class_object(jvm, ptype, int_state).unwrap(); //todo what if not using bootstap loader, todo handle exception
    new_local_ref_public_new(res.as_allocated_obj().into(), int_state)
}