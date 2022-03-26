use std::fs::File;
use std::io::{Cursor, Write};
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jbyteArray, jclass, jint, jio_fprintf, JNIEnv, jobject, jstring, JVM_DefineClass};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::runtime_class::{initialize_class, prepare_class};
use slow_interpreter::rust_jni::interface::define_class_safe;
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::throw_npe;
use itertools::Itertools;


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(env: *mut JNIEnv, _the_unsafe: jobject, name: jstring, bytes: jbyteArray, off: jint, len: jint, loader: jobject, protection_domain: jobject) -> jclass {
    //todo handle protection domain
    assert_eq!(off, 0);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut byte_array = from_object_new(jvm, bytes).unwrap().unwrap_array(jvm).array_iterator().map(|byte| byte.as_njv().unwrap_byte_strict() as u8).collect::<Vec<_>>(); //todo handle npe
    let expected = [-54i8,-2,-70,-66,0,0,0,49,0,117,1,0,41,115,117,110,47,114,101,102,108,101,99,116,47,71,101,110,101,114,97,116,101,100,67,111,110,115,116,114,117,99,116,111,114,65,99,99,101,115,115,111,114,49,7,0,1,1,0,35,115,117,110,47,114,101,102,108,101,99,116,47,67,111,110,115,116,114,117,99,116,111,114,65,99,99,101,115,115,111,114,73,109,112,108,7,0,3,1,0,21,99,111,109,47,115,117,110,47,112,114,111,120,121,47,36,80,114,111,120,121,50,7,0,5,1,0,6,60,105,110,105,116,62,1,0,40,40,76,106,97,118,97,47,108,97,110,103,47,114,101,102,108,101,99,116,47,73,110,118,111,99,97,116,105,111,110,72,97,110,100,108,101,114,59,41,86,12,0,7,0,8,10,0,6,0,9,1,0,11,110,101,119,73,110,115,116,97,110,99,101,1,0,39,40,91,76,106,97,118,97,47,108,97,110,103,47,79,98,106,101,99,116,59,41,76,106,97,118,97,47,108,97,110,103,47,79,98,106,101,99,116,59,1,0,35,106,97,118,97,47,108,97,110,103,47,114,101,102,108,101,99,116,47,73,110,118,111,99,97,116,105,111,110,72,97,110,100,108,101,114,7,0,13,1,0,19,106,97,118,97,47,108,97,110,103,47,84,104,114,111,119,97,98,108,101,7,0,15,1,0,28,106,97,118,97,47,108,97,110,103,47,67,108,97,115,115,67,97,115,116,69,120,99,101,112,116,105,111,110,7,0,17,1,0,30,106,97,118,97,47,108,97,110,103,47,78,117,108,108,80,111,105,110,116,101,114,69,120,99,101,112,116,105,111,110,7,0,19,1,0,34,106,97,118,97,47,108,97,110,103,47,73,108,108,101,103,97,108,65,114,103,117,109,101,110,116,69,120,99,101,112,116,105,111,110,7,0,21,1,0,43,106,97,118,97,47,108,97,110,103,47,114,101,102,108,101,99,116,47,73,110,118,111,99,97,116,105,111,110,84,97,114,103,101,116,69,120,99,101,112,116,105,111,110,7,0,23,1,0,6,60,105,110,105,116,62,1,0,3,40,41,86,12,0,25,0,26,10,0,20,0,27,10,0,22,0,27,1,0,21,40,76,106,97,118,97,47,108,97,110,103,47,83,116,114,105,110,103,59,41,86,12,0,25,0,30,10,0,22,0,31,1,0,24,40,76,106,97,118,97,47,108,97,110,103,47,84,104,114,111,119,97,98,108,101,59,41,86,12,0,25,0,33,10,0,24,0,34,10,0,4,0,27,1,0,16,106,97,118,97,47,108,97,110,103,47,79,98,106,101,99,116,7,0,37,1,0,8,116,111,83,116,114,105,110,103,1,0,20,40,41,76,106,97,118,97,47,108,97,110,103,47,83,116,114,105,110,103,59,12,0,39,0,40,10,0,38,0,41,1,0,4,67,111,100,101,1,0,10,69,120,99,101,112,116,105,111,110,115,1,0,17,106,97,118,97,47,108,97,110,103,47,66,111,111,108,101,97,110,7,0,45,1,0,4,40,90,41,86,12,0,25,0,47,10,0,46,0,48,1,0,12,98,111,111,108,101,97,110,86,97,108,117,101,1,0,3,40,41,90,12,0,50,0,51,10,0,46,0,52,1,0,14,106,97,118,97,47,108,97,110,103,47,66,121,116,101,7,0,54,1,0,4,40,66,41,86,12,0,25,0,56,10,0,55,0,57,1,0,9,98,121,116,101,86,97,108,117,101,1,0,3,40,41,66,12,0,59,0,60,10,0,55,0,61,1,0,19,106,97,118,97,47,108,97,110,103,47,67,104,97,114,97,99,116,101,114,7,0,63,1,0,4,40,67,41,86,12,0,25,0,65,10,0,64,0,66,1,0,9,99,104,97,114,86,97,108,117,101,1,0,3,40,41,67,12,0,68,0,69,10,0,64,0,70,1,0,16,106,97,118,97,47,108,97,110,103,47,68,111,117,98,108,101,7,0,72,1,0,4,40,68,41,86,12,0,25,0,74,10,0,73,0,75,1,0,11,100,111,117,98,108,101,86,97,108,117,101,1,0,3,40,41,68,12,0,77,0,78,10,0,73,0,79,1,0,15,106,97,118,97,47,108,97,110,103,47,70,108,111,97,116,7,0,81,1,0,4,40,70,41,86,12,0,25,0,83,10,0,82,0,84,1,0,10,102,108,111,97,116,86,97,108,117,101,1,0,3,40,41,70,12,0,86,0,87,10,0,82,0,88,1,0,17,106,97,118,97,47,108,97,110,103,47,73,110,116,101,103,101,114,7,0,90,1,0,4,40,73,41,86,12,0,25,0,92,10,0,91,0,93,1,0,8,105,110,116,86,97,108,117,101,1,0,3,40,41,73,12,0,95,0,96,10,0,91,0,97,1,0,14,106,97,118,97,47,108,97,110,103,47,76,111,110,103,7,0,99,1,0,4,40,74,41,86,12,0,25,0,101,10,0,100,0,102,1,0,9,108,111,110,103,86,97,108,117,101,1,0,3,40,41,74,12,0,104,0,105,10,0,100,0,106,1,0,15,106,97,118,97,47,108,97,110,103,47,83,104,111,114,116,7,0,108,1,0,4,40,83,41,86,12,0,25,0,110,10,0,109,0,111,1,0,10,115,104,111,114,116,86,97,108,117,101,1,0,3,40,41,83,12,0,113,0,114,10,0,109,0,115,0,1,0,2,0,4,0,0,0,0,0,2,0,1,0,25,0,26,0,1,0,43,0,0,0,17,0,1,0,1,0,0,0,5,42,-73,0,36,-79,0,0,0,0,0,1,0,11,0,12,0,2,0,43,0,0,0,89,0,6,0,2,0,0,0,53,-69,0,6,89,43,-66,17,0,1,-97,0,11,-69,0,22,89,-73,0,29,-65,43,17,0,0,50,-64,0,14,-73,0,10,-80,-73,0,42,-69,0,22,90,95,-73,0,32,-65,-69,0,24,90,95,-73,0,35,-65,0,3,0,0,0,28,0,32,0,18,0,0,0,28,0,32,0,20,0,28,0,31,0,44,0,16,0,0,0,44,0,0,0,4,0,1,0,24,0,0];
    let expected = expected.iter().cloned().map(|signed|signed as u8).collect_vec();
    // assert_eq!(byte_array.as_slice(),&expected);
    let jname = match NewJavaValueHandle::Object(from_object_new(jvm, name).unwrap()).cast_string() {
        None => return throw_npe(jvm, int_state),
        Some(jname) => jname,
    };
    let class_name = ClassName::Str(jname.to_rust_string(jvm)); //todo need to parse arrays here
    let classfile = Arc::new(parse_class_file(&mut Cursor::new(byte_array.as_slice())).expect("todo error handling and verification"));
    let class_view = ClassBackedView::from(classfile.clone(), &jvm.string_pool);
    if jvm.config.store_generated_classes {
        File::create(PTypeView::from_compressed(&class_view.type_(), &jvm.string_pool).class_name_representation()).unwrap().write_all(byte_array.clone().as_slice()).unwrap();
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