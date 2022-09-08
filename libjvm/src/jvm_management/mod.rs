use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ffi::c_void;
use std::ops::Deref;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::RwLock;

use lazy_static::lazy_static;
use wtf8::Wtf8Buf;


use jvmti_jni_bindings::jmmInterface_1_;
use jvmti_jni_bindings::{_jobject, jboolean, jint, JNIEnv, jobject, JVM_INTERFACE_VERSION, jvm_version_info};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::util::properties::Properties;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::interface::jmm::initial_jmm;
use slow_interpreter::rust_jni::interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_object, to_object};
use slow_interpreter::utils::pushable_frame_todo;

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
unsafe extern "system" fn JVM_IsSupportedJNIVersion(version: jint) -> jboolean {
    //todo for now we support everything?
    true as jboolean
}

thread_local! {
    static JMM: RefCell<Option<*const jmmInterface_1_>> = RefCell::new(None)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetManagement(version: jint) -> *mut ::std::os::raw::c_void {
    eprintln!("Attempt to get jmm which is unsupported");
    JMM.with(|refcell: &RefCell<Option<*const jmmInterface_1_>>| {
        if refcell.borrow().is_none() {
            *refcell.borrow_mut() = Some(Box::into_raw(box initial_jmm()));
        }
        *refcell.borrow().as_ref().unwrap()
    }) as *mut c_void
}

pub mod management_impl;

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
    let props = JavaValue::Object(todo!() /*from_jclass(jvm,agent_props)*/).cast_properties();

    let sun_java_command = JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_str("sun.java.command"))?;
    let sun_java_command_val = JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_str("sun.jvm.flags"))?;
    let sun_java_command_val = JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_str("sun.jvm.args"))?;
    let sun_java_command_val = JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_str("command line not currently compatible todo"))?;
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