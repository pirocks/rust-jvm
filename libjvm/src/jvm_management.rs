use std::ops::Deref;
use std::process::exit;

use jvmti_jni_bindings::{jboolean, jint, JNIEnv, jobject, JVM_INTERFACE_VERSION, jvm_version_info};
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::util::properties::Properties;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_GetInterfaceVersion() -> jint {
    JVM_INTERFACE_VERSION as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_OnExit(func: ::std::option::Option<unsafe extern "C" fn()>) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Exit(code: jint) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Halt(code: jint) {
    exit(code);//todo cleanup and gracefully shutdown
}

#[no_mangle]
unsafe extern "system" fn JVM_ActiveProcessorCount() -> jint {
    num_cpus::get() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSupportedJNIVersion(version: jint) -> jboolean {
    //todo for now we support everything?
    true as jboolean
}


#[no_mangle]
unsafe extern "system" fn JVM_GetManagement(version: jint) -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let props = JavaValue::Object(from_object(agent_props)).cast_properties();

    let sun_java_command = JString::from_rust(jvm, int_state, "sun.java.command".to_string());
    let sun_java_command_val = JString::from_rust(jvm, int_state, "command line not currently compatible todo".to_string());
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, int_state, "sun.jvm.flags".to_string());
    let sun_java_command_val = JString::from_rust(jvm, int_state, "command line not currently compatible todo".to_string());
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, int_state, "sun.jvm.args".to_string());
    let sun_java_command_val = JString::from_rust(jvm, int_state, "command line not currently compatible todo".to_string());
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    agent_props
}

#[no_mangle]
unsafe extern "system" fn JVM_GetVersionInfo(env: *mut JNIEnv, info: *mut jvm_version_info, info_size: usize) {
    (*info).jvm_version = 8;//todo what should I put here?
}


#[no_mangle]
unsafe extern "system" fn JVM_SupportsCX8() -> jboolean {
    false as jboolean//todo this is actually something that might be easy to support.
}

#[no_mangle]
unsafe extern "system" fn JVM_BeforeHalt() {
    unimplemented!()
}


