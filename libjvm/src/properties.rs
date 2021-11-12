use std::ptr::null_mut;

use classfile_view::view::ClassView;
use jvmti_jni_bindings::{JNIEnv, jobject};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use slow_interpreter::instructions::ldc::create_string_on_stack;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};
use slow_interpreter::utils::throw_npe_res;

#[no_mangle]
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p0: jobject) -> jobject {
    //todo get rid of these  hardcoded paths
    // sun.boot.class.path
    match (|| {
        add_prop(env, p0, "sun.boot.library.path".to_string(), "/home/francis/Clion/rust-jvm/target/debug/deps:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/amd64".to_string())?;
        add_prop(env, p0, "sun.boot.class.path".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/jce.jar:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes:/home/francis/Desktop/test/unzipped-jar".to_string())?;
        add_prop(env, p0, "java.class.path".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/jce.jar:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes:/home/francis/Desktop/test/unzipped-jar".to_string())?;
        add_prop(env, p0, "java.library.path".to_string(), "/usr/java/packages/lib/amd64:/usr/lib64:/lib64:/lib:/usr/lib".to_string())?;
        // add_prop(env, p0, "org.slf4j.simpleLogger.defaultLogLevel ".to_string(), "off".to_string())?;
        add_prop(env, p0, "log4j2.disable.jmx".to_string(), "true".to_string());
        Ok(add_prop(env, p0, "java.home".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/".to_string())?)
    })() {
        Err(WasException {}) => null_mut(),
        Ok(res) => res
    }
}

unsafe fn add_prop(env: *mut JNIEnv, p: jobject, key: String, val: String) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    create_string_on_stack(jvm, int_state, key);
    let key = int_state.pop_current_operand_stack(Some(CClassName::object().into()));
    create_string_on_stack(jvm, int_state, val);
    let val = int_state.pop_current_operand_stack(Some(CClassName::object().into()));
    let prop_obj = match from_object(jvm, p) {
        Some(x) => x,
        None => return throw_npe_res(jvm, int_state),
    };
    let runtime_class = &prop_obj.unwrap_normal_object().objinfo.class_pointer;
    let class_view = &runtime_class.view();
    let candidate_meth = class_view.lookup_method_name(MethodName::method_setProperty());
    let meth = candidate_meth.get(0).unwrap();
    let md = meth.desc();
    int_state.push_current_operand_stack(JavaValue::Object(prop_obj.clone().into()));
    int_state.push_current_operand_stack(key);
    int_state.push_current_operand_stack(val);
    invoke_virtual_method_i(jvm, int_state, md, runtime_class.clone(), meth, todo!());
    int_state.pop_current_operand_stack(Some(CClassName::object().into()));
    Ok(p)
}