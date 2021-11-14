use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::view::ClassBackedView;
use jvmti_jni_bindings::{jbyteArray, jclass, jint, jio_fprintf, JNIEnv, jobject, jstring, JVM_DefineClass};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassClass};
use slow_interpreter::rust_jni::interface::define_class_safe;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::throw_npe;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(env: *mut JNIEnv, _the_unsafe: jobject, name: jstring, bytes: jbyteArray, off: jint, len: jint, loader: jobject, protection_domain: jobject) -> jclass {
    //todo handle protection domain
    assert_eq!(off, 0);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut byte_array = from_object(jvm, bytes).unwrap().unwrap_array().unwrap_byte_array(jvm).iter().map(|byte| *byte as u8).collect::<Vec<_>>(); //todo handle npe
    let jname = match JavaValue::Object(from_object(jvm, name)).cast_string() {
        None => return throw_npe(jvm, int_state),
        Some(jname) => jname,
    };
    let class_name = ClassName::Str(jname.to_rust_string(jvm)); //todo need to parse arrays here
    let classfile = Arc::new(parse_class_file(&mut byte_array.as_slice()).expect("todo error handling and verification"));
    let class_view = ClassBackedView::from(classfile.clone(), &jvm.string_pool);
    let loader_name = if loader != null_mut() { JavaValue::Object(from_object(jvm, loader)).cast_class_loader().to_jvm_loader(jvm) } else { LoaderName::BootstrapLoader };
    new_local_ref_public(
        match define_class_safe(jvm, int_state, classfile, loader_name, class_view) {
            Ok(object) => object,
            Err(_) => todo!(),
        }
            .unwrap_object(),
        int_state,
    )
}