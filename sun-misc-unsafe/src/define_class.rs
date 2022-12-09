use std::fs::File;
use std::io::{Cursor, Write};
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jbyteArray, jclass, jint, JNIEnv, jobject, jstring};
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::define_class_safe::define_class_safe;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::rust_jni::jni_utils::new_local_ref_public_new;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::from_object_new;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(env: *mut JNIEnv, _the_unsafe: jobject, _name: jstring, bytes: jbyteArray, off: jint, len: jint, loader: jobject, _protection_domain: jobject) -> jclass {
    //todo handle protection domain
    assert_eq!(off, 0);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let byte_array = from_object_new(jvm, bytes).unwrap().unwrap_array().array_iterator().map(|byte| byte.unwrap_byte_strict() as u8).collect::<Vec<_>>(); //todo handle npe
    //todo handle len
    assert_eq!(byte_array.len(), len as usize);
    // let jname = match NewJavaValueHandle::Object(from_object_new(jvm, name).unwrap()).cast_string() {
    //     None => return throw_npe(jvm, int_state,get_throw(env)),
    //     Some(jname) => jname,
    // };
    // let class_name = ClassName::Str(jname.to_rust_string(jvm)); //todo need to parse arrays here
    let classfile = Arc::new(parse_class_file(&mut Cursor::new(byte_array.as_slice())).expect("todo error handling and verification"));
    let class_view = ClassBackedView::from(classfile.clone(), &jvm.string_pool);
    if jvm.config.store_generated_classes {
        let class_name_string = PTypeView::from_compressed(class_view.type_(), &jvm.string_pool).class_name_representation();
        let mut file = File::create(format!("{}.class", class_name_string)).unwrap();
        file.write_all(byte_array.clone().as_slice()).unwrap();
    }
    let loader_name = if loader != null_mut() {
        NewJavaValueHandle::Object(from_object_new(jvm, loader).unwrap()).cast_class_loader().to_jvm_loader(jvm)
    } else {
        LoaderName::BootstrapLoader
    };
    new_local_ref_public_new(
        match define_class_safe(jvm, int_state, classfile, loader_name, class_view) {
            Ok(object) => object,
            Err(_) => todo!(),
        }
            .unwrap_object().unwrap().as_allocated_obj().into(),
        int_state,
    )
}