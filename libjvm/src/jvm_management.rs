use jvmti_jni_bindings::{jboolean, jint, jobject, JNIEnv, jvm_version_info};
use slow_interpreter::rust_jni::native_util::{get_state, to_object, from_object, get_frame, get_thread};
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::java::util::properties::Properties;
use slow_interpreter::java::lang::string::JString;
use std::ops::Deref;
use std::process::exit;
use slow_interpreter::get_state_thread_frame;

#[no_mangle]
unsafe extern "system" fn JVM_GetInterfaceVersion() -> jint {
    unimplemented!()
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
    get_state_thread_frame!(env,state,thread,frames,frame);
    let props = JavaValue::Object(from_object(agent_props)).cast_properties();

    let sun_java_command = JString::from(state, frame, "sun.java.command".to_string());
    let sun_java_command_val = JString::from(state, frame, "command line not currently compatible todo".to_string());
    props.set_property(state, frame, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from(state, frame, "sun.jvm.flags".to_string());
    let sun_java_command_val = JString::from(state, frame, "command line not currently compatible todo".to_string());
    props.set_property(state, frame, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from(state, frame, "sun.jvm.args".to_string());
    let sun_java_command_val = JString::from(state, frame, "command line not currently compatible todo".to_string());
    props.set_property(state, frame, sun_java_command, sun_java_command_val);

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

