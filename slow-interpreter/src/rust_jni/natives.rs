use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::Deref;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};
use by_address::ByAddress;
use libc::{RTLD_GLOBAL, RTLD_LAZY};
use libloading::{Library, Symbol};
use jvmti_jni_bindings::invoke_interface::JNIInvokeInterfaceNamedReservedPointers;
use jvmti_jni_bindings::{jint, JNI_VERSION_1_1};
use runtime_class_stuff::RuntimeClass;
use crate::jvm_state::JVMState;
use crate::rust_jni::invoke_interface::get_invoke_interface_new;

#[derive(Debug)]
pub struct NativeLib {
pub library: Library,
}

#[derive(Debug)]
pub struct NativeLibraries<'gc> {
    pub libjava_path: PathBuf,
    pub native_libs: RwLock<HashMap<String, NativeLib>>,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, RwLock<HashMap<u16, unsafe extern "C" fn()>>>>,
}

fn default_on_load(_: *mut *const JNIInvokeInterfaceNamedReservedPointers, _: *mut c_void) -> i32 {
    JNI_VERSION_1_1 as i32
}

impl<'gc> NativeLibraries<'gc> {
    pub unsafe fn load<'l>(&self, jvm: &'gc JVMState<'gc>, path: &PathBuf, name: String) {
        let onload_fn_ptr = self.get_onload_ptr_and_add(path, name);
        let interface: *const JNIInvokeInterfaceNamedReservedPointers = get_invoke_interface_new(jvm);
        onload_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterfaceNamedReservedPointers, null_mut());
        //todo check return res
    }

    pub unsafe fn get_onload_ptr_and_add(&self, path: &PathBuf, name: String) -> fn(*mut *const JNIInvokeInterfaceNamedReservedPointers, *mut c_void) -> i32 {
        let lib = Library::new(path, (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
        let on_load = match lib.get::<fn(vm: *mut *const JNIInvokeInterfaceNamedReservedPointers, reserved: *mut c_void) -> jint>("JNI_OnLoad".as_bytes()) {
            Ok(x) => Some(x),
            Err(err) => {
                if err.to_string().contains(" undefined symbol: JNI_OnLoad") {
                    None
                } else {
                    todo!()
                }
            }
        };
        let onload_fn_ptr = on_load.map(|on_load| *on_load.deref()).unwrap_or(default_on_load);
        self.native_libs.write().unwrap().insert(name, NativeLib { library: lib });
        onload_fn_ptr
    }

    pub unsafe fn lookup_onload(&self, name: String) -> Result<unsafe extern "system" fn(), LookupError> {
        let guard = self.native_libs.read().unwrap();
        let native_lib = guard.get(&name);
        let result = native_lib.ok_or(LookupError::NoLib)?.library.get("JNI_OnLoad".as_bytes());
        let symbol: Symbol<unsafe extern "system" fn()> = result?;
        Ok(*symbol.deref())
    }
}

pub enum LookupError {
    LibLoading(libloading::Error),
    NoLib,
}

impl From<libloading::Error> for LookupError {
    fn from(err: libloading::Error) -> Self {
        LookupError::LibLoading(err)
    }
}

impl<'gc> NativeLibraries<'gc> {
    pub fn new(libjava: PathBuf) -> NativeLibraries<'gc> {
        NativeLibraries {
            libjava_path: libjava,
            native_libs: Default::default(),
            registered_natives: RwLock::new(HashMap::new()),
        }
    }
}
