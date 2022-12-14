use std::cell::RefCell;
use std::ffi::c_void;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::RwLock;

use lazy_static::lazy_static;
use wtf8::Wtf8Buf;

use jvmti_jni_bindings::{jboolean, jint, JNIEnv, jobject, JVM_INTERFACE_VERSION, jvm_version_info};
use jvmti_jni_bindings::jmm_interface::JMMInterfaceNamedReservedPointers;
use slow_interpreter::better_java_stack::java_stack_guard::JMM;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::stdlib::java::lang::string::JString;

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
        if let Some(func) = func.as_ref() {
            func();
        };
    }
    exit(code);
}

#[no_mangle]
unsafe extern "system" fn JVM_Halt(code: jint) {
    exit(code);
    // halt means that no cleanup is desired
}

#[no_mangle]
unsafe extern "system" fn JVM_ActiveProcessorCount() -> jint {
    num_cpus::get() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSupportedJNIVersion(_version: jint) -> jboolean {
    //todo for now we support everything?
    true as jboolean
}


#[no_mangle]
unsafe extern "system" fn JVM_GetManagement(_version: jint) -> *mut ::std::os::raw::c_void {
    eprintln!("Attempt to get jmm which is unsupported");
    JMM.with(|refcell: &RefCell<Option<*mut JMMInterfaceNamedReservedPointers>>| {
        *refcell.borrow().as_ref().unwrap()
    }) as *mut c_void
}

#[no_mangle]
unsafe extern "system" fn JVM_InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> jobject {
    match InitAgentProperties(env, agent_props) {
        Ok(res) => res,
        Err(_) => null_mut(),
    }
}

unsafe fn InitAgentProperties<'gc>(env: *mut JNIEnv, agent_props: jobject) -> Result<jobject, WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let props = match from_object_new(jvm, agent_props) {
        Some(x) => x,
        None => todo!(),
    }.cast_properties();

    let sun_java_command = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("sun.java.command"))?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    match props.set_property(jvm, int_state, sun_java_command, sun_java_command_val) {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let sun_java_command = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("sun.jvm.flags"))?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    match props.set_property(jvm, int_state, sun_java_command, sun_java_command_val) {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    let sun_java_command = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("sun.jvm.args"))?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    match props.set_property(jvm, int_state, sun_java_command, sun_java_command_val) {
        Ok(x) => x,
        Err(_) => todo!(),
    };

    Ok(agent_props)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetVersionInfo(_env: *mut JNIEnv, info: *mut jvm_version_info, _info_size: usize) {
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