use std::sync::Arc;

use classfile_parser::parse_class_file;
use jvmti_jni_bindings::{jbyteArray, jclass, jint, JNIEnv, jobject, jstring};
use rust_jvm_common::classnames::ClassName;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineClass(env: *mut JNIEnv, _the_unsafe: jobject, name: jstring, bytes: jbyteArray, off: jint, len: jint, loader: jobject, protection_domain: jobject) -> jclass {
    //todo handle protection domain
    unimplemented!();
    assert_eq!(off, 0);
    let mut byte_array = from_object(bytes).unwrap().unwrap_array().unwrap_byte_array().iter().map(|byte| *byte as u8).collect::<Vec<_>>();
    let jname = JavaValue::Object(from_object(name)).cast_string();
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class_name = ClassName::Str(jname.to_rust_string());
    let classfile = Arc::new(parse_class_file(&mut byte_array.as_slice()));
    // int_state.print_stack_trace();
    jvm.bootstrap_loader.add_pre_loaded(&class_name, &classfile);
    load_class_constant_by_type(jvm, int_state, &class_name.into());
    to_object(int_state.pop_current_operand_stack().unwrap_object())
}
