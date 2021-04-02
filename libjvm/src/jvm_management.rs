use std::ops::Deref;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::RwLock;

use lazy_static::lazy_static;

use jvmti_jni_bindings::{_jobject, jboolean, jint, JNIEnv, jobject, JVM_INTERFACE_VERSION, jvm_version_info};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::util::properties::Properties;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_GetInterfaceVersion() -> jint {
    JVM_INTERFACE_VERSION as jint
}


lazy_static! {
    static ref ON_EXIT: RwLock<Vec<Option<unsafe extern "C" fn()>>> = RwLock::new(Vec::new());
}

#[no_mangle]
unsafe extern "system" fn JVM_OnExit(func: Option<unsafe extern "C" fn()>) {
    ON_EXIT.write().unwrap().push(func);
}

#[no_mangle]
unsafe extern "system" fn JVM_Exit(code: jint) {
    //todo run finalizers blocking on gc
    for func in ON_EXIT.read().unwrap().iter() {
        if let Some(func) = func.as_ref() { func(); };
    }
    exit(code);
}

#[no_mangle]
unsafe extern "system" fn JVM_Halt(code: jint) {
    exit(code);// halt means that no cleanup is desired
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
    eprintln!("Attempt to get jmm which is unsupported");
    null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> jobject {
    match InitAgentProperties(env, agent_props) {
        Ok(res) => res,
        Err(_) => null_mut()
    }
}

unsafe fn InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let props = JavaValue::Object(from_object(agent_props)).cast_properties();

    let sun_java_command = JString::from_rust(jvm, int_state, "sun.java.command".to_string())?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, "command line not currently compatible todo".to_string())?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, int_state, "sun.jvm.flags".to_string())?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, "command line not currently compatible todo".to_string())?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, int_state, "sun.jvm.args".to_string())?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, "command line not currently compatible todo".to_string())?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    Ok(agent_props)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetVersionInfo(env: *mut JNIEnv, info: *mut jvm_version_info, info_size: usize) {
    (*info).jvm_version = 8;
    (*info).set_is_attach_supported(0);
    (*info).set_update_version(0);
    (*info).set_special_update_version(0);
    (*info).set_reserved1(0);
}


#[no_mangle]
unsafe extern "system" fn JVM_SupportsCX8() -> jboolean {
    false as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_BeforeHalt() {}


