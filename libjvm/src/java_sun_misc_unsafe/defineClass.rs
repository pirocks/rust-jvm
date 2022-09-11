use std::fs::File;
use std::io::{Cursor, Write};
use std::ptr::null_mut;
use std::sync::Arc;

use itertools::Itertools;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jbyteArray, jclass, jint, jio_fprintf, JNIEnv, jobject, jstring, JVM_DefineClass};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::interpreter::common::ldc::load_class_constant_by_type;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::runtime_class::{initialize_class, prepare_class};
use slow_interpreter::rust_jni::jni_interface::define_class_safe;
use slow_interpreter::rust_jni::jni_interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::jni_interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object};
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::utils::throw_npe;

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(env: *mut JNIEnv, _the_unsafe: jobject, name: jstring, bytes: jbyteArray, off: jint, len: jint, loader: jobject, protection_domain: jobject) -> jclass {
    //todo handle protection domain
    assert_eq!(off, 0);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut byte_array = from_object_new(jvm, bytes).unwrap().unwrap_array().array_iterator().map(|byte| byte.unwrap_byte_strict() as u8).collect::<Vec<_>>(); //todo handle npe
    let jname = match NewJavaValueHandle::Object(from_object_new(jvm, name).unwrap()).cast_string() {
        None => return throw_npe(jvm, int_state),
        Some(jname) => jname,
    };
    let class_name = ClassName::Str(jname.to_rust_string(jvm)); //todo need to parse arrays here
    let classfile = Arc::new(parse_class_file(&mut Cursor::new(byte_array.as_slice())).expect("todo error handling and verification"));
    let class_view = ClassBackedView::from(classfile.clone(), &jvm.string_pool);
    if jvm.config.store_generated_classes {
        File::create(PTypeView::from_compressed(class_view.type_(), &jvm.string_pool).class_name_representation()).unwrap().write_all(byte_array.clone().as_slice()).unwrap();
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