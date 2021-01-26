use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::ptr::null_mut;

use nix::sys::aio::aio_suspend;

use classfile_view::loading::LoaderName;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject, jstring, JVM_Available};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType::Ref;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::jvm_state::ClassStatus;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromBootLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_str = CStr::from_ptr(name).to_str().unwrap().to_string();
    //todo duplication
    let class_name = ClassName::Str(name_str);
    dbg!(&class_name);

    let loader_obj = int_state.previous_frame().local_vars()[0].cast_class_loader();
    let current_loader = loader_obj.to_jvm_loader(jvm);
    todo!()
    // let runtime_class_res = find_class_from_bootloader(jvm, int_state, current_loader, class_name.clone()).unwrap();
    // assert_eq!(runtime_class_res.loader(), current_loader);

    // let res = to_object(get_or_create_class_object_override_loader(jvm, &class_name.into(), int_state, current_loader).unwrap().into());

    // assert_eq!(JavaValue::Object(from_object(res)).cast_class().get_class_loader(jvm, int_state).map(|loader| loader.to_jvm_loader(jvm)).unwrap_or(LoaderName::BootstrapLoader), current_loader); //todo techincally this shouldn't be callled at all on laoded classes?
    // res
    // let loaded = jvm.bootstrap_loader.load_class(jvm.bootstrap_loader.clone(), &class_name, jvm.bootstrap_loader.clone(), jvm.get_live_object_pool_getter());
    // match loaded {
    //     Result::Err(_) => null_mut(),
    //     Result::Ok(view) => {
    //         new_local_ref_public(get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(class_name)), int_state, jvm.bootstrap_loader.clone()).unwrap().into(), int_state)
    //     }
    // }
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
    let name_str = JavaValue::Object(from_object(name)).cast_string().to_rust_string();
    assert_ne!(&name_str, "int");
    // dbg!(&name_str);
    //todo what if not bl
    let class_name = ClassName::Str(name_str.replace(".", "/"));

    // dbg!(loader);
    // let loader_name = JavaValue::Object(from_object(loader)).cast_class_loader().to_jvm_loader(jvm);
    // let maybe_status = jvm.classes.read().unwrap().get_status(loader_name, class_name.into());
    //
    // match maybe_status {
    //     None => {
    //         return null_mut();
    //     }
    //     Some(_) => todo!()
    // }


    dbg!(&class_name);
    let loaded = jvm.classes.write().unwrap().is_loaded(&class_name.clone().into());
    match loaded {
        None => null_mut(),
        Some(view) => {
            // todo what if name is long/int etc.
            let res = get_or_create_class_object(jvm, PTypeView::Ref(ReferenceTypeView::Class(class_name)), int_state).unwrap();
            new_local_ref_public(res.into(), int_state)
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

    let res = get_or_create_class_object(jvm, ptype, int_state).unwrap();//todo what if not using bootstap loader
    new_local_ref_public(res.into(), int_state)
}
