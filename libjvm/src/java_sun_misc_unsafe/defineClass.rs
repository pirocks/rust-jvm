use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::loading::LoaderName;
use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jbyteArray, jclass, jint, JNIEnv, jobject, jstring};
use rust_jvm_common::classnames::ClassName;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassClass};
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(env: *mut JNIEnv, _the_unsafe: jobject, name: jstring, bytes: jbyteArray, off: jint, len: jint, loader: jobject, protection_domain: jobject) -> jclass {
    //todo handle protection domain
    assert_eq!(off, 0);
    let mut byte_array = from_object(bytes).unwrap().unwrap_array().unwrap_byte_array().iter().map(|byte| *byte as u8).collect::<Vec<_>>();//todo handle npe
    let jname = JavaValue::Object(from_object(name)).cast_string();
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class_name = ClassName::Str(jname.to_rust_string());//todo need to parse arrays here
    let classfile = Arc::new(parse_class_file(&mut byte_array.as_slice()).expect("todo error handling and verification"));
    let class_view = Arc::new(ClassView::from(classfile.clone()));
    let loader_name = if loader != null_mut() {
        JavaValue::Object(from_object(loader)).cast_class_loader().to_jvm_loader(jvm)
    } else {
        LoaderName::BootstrapLoader
    };
    todo!()
}
