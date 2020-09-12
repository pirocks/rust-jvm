use std::ffi::{CStr, CString};
use std::ops::Deref;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject, jstring};
use libjvm_utils::jstring_to_string;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType::Ref;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromBootLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_str = CStr::from_ptr(name).to_str().unwrap().to_string();
    //todo duplication
    let class_name = ClassName::Str(name_str);
    //todo not sure if this implementation is correct
    let loaded = jvm.bootstrap_loader.load_class(jvm.bootstrap_loader.clone(), &class_name, jvm.bootstrap_loader.clone(), jvm.get_live_object_pool_getter());
    match loaded {
        Result::Err(_) => return new_local_ref_public(None, int_state),
        Result::Ok(view) => {
            new_local_ref_public(get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(class_name)), int_state, jvm.bootstrap_loader.clone()).into(), int_state)
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClassLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, init: jboolean, loader: jobject, throwError: jboolean) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, init: jboolean, from: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLoadedClass(env: *mut JNIEnv, loader: jobject, name: jstring) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_str = jstring_to_string(name);
    assert_ne!(&name_str, "int");
    // dbg!(&name_str);
    //todo what if not bl
    let class_name = ClassName::Str(name_str);
    let loaded = jvm.bootstrap_loader.find_loaded_class(&class_name);
    match loaded {
        None => return new_local_ref_public(None, int_state),
        Some(view) => {
            //todo what if name is long/int etc.
            get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(class_name)), int_state, jvm.bootstrap_loader.clone());
            new_local_ref_public(int_state.pop_current_operand_stack().unwrap_object(), int_state)
        }
    }
}


#[no_mangle]
unsafe extern "system" fn JVM_FindPrimitiveClass(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char) -> jclass {
    assert_ne!(utf, std::ptr::null());
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let float = CString::new("float").unwrap();
    let float_cstr = float.into_raw();
    let double = CString::new("double").unwrap();
    let double_cstr = double.into_raw();
    let int = CString::new("int").unwrap();
    let int_cstr = int.into_raw();
    let boolean = CString::new("boolean").unwrap();
    let boolean_cstr = boolean.into_raw();
    let char_ = CString::new("char").unwrap();
    let char_cstr = char_.into_raw();
    let long = CString::new("long").unwrap();
    let long_cstr = long.into_raw();
    let byte = CString::new("byte").unwrap();
    let byte_cstr = byte.into_raw();
    let short = CString::new("short").unwrap();
    let short_cstr = short.into_raw();
    let void = CString::new("void").unwrap();
    let void_cstr = void.into_raw();
    let (class_name, as_str, ptype) = if libc::strncmp(float_cstr, utf, libc::strlen(float_cstr) + 1) == 0 {
        (ClassName::float(), "float", PTypeView::FloatType)
    } else if libc::strncmp(double_cstr, utf, libc::strlen(double_cstr) + 1) == 0 {
        (ClassName::double(), "double", PTypeView::DoubleType)
    } else if libc::strncmp(int_cstr, utf, libc::strlen(int_cstr) + 1) == 0 {
        (ClassName::int(), "int", PTypeView::IntType)
    } else if libc::strncmp(boolean_cstr, utf, libc::strlen(boolean_cstr) + 1) == 0 {
        (ClassName::boolean(), "boolean", PTypeView::BooleanType)
    } else if libc::strncmp(char_cstr, utf, libc::strlen(char_cstr) + 1) == 0 {
        (ClassName::character(), "character", PTypeView::CharType)
    } else if libc::strncmp(long_cstr, utf, libc::strlen(long_cstr) + 1) == 0 {
        (ClassName::long(), "long", PTypeView::LongType)
    } else if libc::strncmp(byte_cstr, utf, libc::strlen(byte_cstr) + 1) == 0 {
        (ClassName::byte(), "byte", PTypeView::ByteType)
    } else if libc::strncmp(short_cstr, utf, libc::strlen(short_cstr) + 1) == 0 {
        (ClassName::short(), "short", PTypeView::ShortType)
    } else if libc::strncmp(void_cstr, utf, libc::strlen(void_cstr) + 1) == 0 {
        (ClassName::void(), "void", PTypeView::VoidType)
    } else {
        dbg!((*utf) as u8 as char);
        unimplemented!()
    };

    let res = get_or_create_class_object(jvm, &ptype, int_state, jvm.bootstrap_loader.clone());//todo what if not using bootstap loader
    return new_local_ref_public(res.into(), int_state);
}
