use std::convert::Infallible;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::mem::transmute;
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::str::FromStr;

use jvmti_jni_bindings::{JavaVM, JNI_VERSION_1_8, JNIInvokeInterface_};
use sketch_jvm_version_of_utf8::{JVMString, PossiblyJVMString, ValidationError};
use slow_interpreter::jvm_state::{JVM, JVMState};

use interfaces::invoke_interface::get_state_invoke_interface;

static mut INVOKE_INTERFACE: *const JNIInvokeInterface_ = null();

#[no_mangle]
unsafe extern "system" fn setup_jvm_pointer_hack(invoke_interface_: *const JNIInvokeInterface_) {
    INVOKE_INTERFACE = invoke_interface_;
}

#[no_mangle]
unsafe extern "system" fn JVM_LoadLibrary(name: *const ::std::os::raw::c_char) -> *mut c_void {
    let jvm = get_state_invoke_interface(&mut INVOKE_INTERFACE);
    let path = match PossiblyJVMString::new(CStr::from_ptr(name).to_bytes().to_vec()).validate() {
        Ok(path) => match OsString::from_str(path.to_string_validated().as_str()) {
            Ok(path) => path,
            Err(_) => return null_mut(),
        },
        Err(_) => return null_mut(),
    };
    let name = match Path::new(&path).file_stem() {
        None => return null_mut(),
        Some(file_name) => match file_name.to_str() {
            None => return null_mut(),
            Some(file_name_str) => file_name_str.replace("lib", ""),
        },
    };
    let res = jvm.native_libaries.get_onload_ptr_and_add(&PathBuf::from(path), name);
    res as *mut c_void
}

#[no_mangle]
unsafe extern "system" fn JVM_UnloadLibrary(handle: *mut c_void) {
    // let jvm = JVM.as_ref().unwrap();
    // unimplemented!()
    //todo this seems to be actually be called so it should really work
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLibraryEntry(handle: *mut c_void, name: *const ::std::os::raw::c_char) -> *mut c_void {
    if name == null() {
        todo!()
    }
    let name = match PossiblyJVMString::new(CStr::from_ptr(name).to_bytes().to_vec()).validate() {
        Ok(name) => name.to_string_validated(),
        Err(ValidationError) => return null_mut(),
    };
    if !handle.is_null() && &name == "JNI_OnLoad" {
        return handle;
    }
    let jvm = get_state_invoke_interface(&mut INVOKE_INTERFACE);
    match jvm.native_libaries.lookup_onload(name) {
        Ok(res) => res as *mut c_void,
        Err(_) => null_mut(),
    }
}