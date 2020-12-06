use std::mem::forget;
use std::os::raw::c_void;

use parking_lot::ReentrantMutex;

use jvmti_jni_bindings::{jint, JNI_OK};
use slow_interpreter::jvmti::monitor::create_raw_monitor;
use slow_interpreter::threading::monitors::Monitor;

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorCreate() -> *mut ::std::os::raw::c_void {
    let reentrant_mutex = box ReentrantMutex::new(());
    Box::into_raw(reentrant_mutex) as *mut c_void
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorDestroy(mon: *mut ::std::os::raw::c_void) {
    let mon_box: Box<ReentrantMutex<()>> = Box::from_raw(mon as *mut ReentrantMutex<()>);
    drop(mon_box);
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorEnter(mon: *mut ::std::os::raw::c_void) -> jint {
    forget((mon as *mut ReentrantMutex<()>).as_mut().unwrap().lock());
    JNI_OK as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorExit(mon: *mut ::std::os::raw::c_void) {
    (mon as *mut ReentrantMutex<()>).as_mut().unwrap().force_unlock();
}

