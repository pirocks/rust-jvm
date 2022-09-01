use std::ffi::CStr;
use std::mem::transmute;
use std::os::raw::c_char;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jclass, jmethodID, JNIEnv};
use rust_jvm_common::compressed_classfile::names::MethodName;

use another_jit_vm_ir::WasException;
use crate::rust_jni::interface::{get_interpreter_state, get_state};
use crate::rust_jni::interface::misc::get_all_methods;
use crate::rust_jni::native_util::{from_jclass};

//for now a method id is a pair of class pointers and i.
//turns out this is for member functions only
//see also get_static_method_id
pub unsafe extern "C" fn get_method_id(env: *mut JNIEnv, clazz: jclass, name: *const c_char, sig: *const c_char) -> jmethodID {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let method_name = match CStr::from_ptr(name).to_str() {
        Ok(method_name) => MethodName(jvm.string_pool.add_name(method_name, false)),
        Err(_) => return null_mut(),
    };
    let method_descriptor_str = match CStr::from_ptr(sig).to_str() {
        Ok(method_descriptor_str) => jvm.string_pool.add_name(method_descriptor_str, false),
        Err(_) => return null_mut(),
    };

    let runtime_class = from_jclass(jvm, clazz).as_runtime_class(jvm);
    let all_methods = match get_all_methods(jvm, todo!()/*int_state*/, runtime_class, false) {
        Ok(all_methods) => all_methods,
        Err(WasException {}) => {
            return null_mut();
        }
    };

    let (_method_i, (c, m)) = all_methods
        .iter()
        .enumerate()
        .find(|(_, (c, i))| {
            let c_view = c.view();
            let method_view = &c_view.method_view_i(*i);
            let cur_desc = method_view.desc_str();
            let cur_method_name = method_view.name();
            cur_method_name == method_name && method_descriptor_str == cur_desc
        })
        .unwrap();
    let method_id = jvm.method_table.write().unwrap().get_method_id(c.clone(), *m as u16);
    transmute(method_id)
}
