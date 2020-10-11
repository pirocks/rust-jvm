use std::mem::transmute;
use std::os::raw::c_char;

use jvmti_jni_bindings::{jclass, jmethodID, JNIEnv};

use crate::rust_jni::interface::misc::get_all_methods;
use crate::rust_jni::native_util::{from_jclass, get_interpreter_state, get_state};

//for now a method id is a pair of class pointers and i.
//turns out this is for member functions only
//see also get_static_method_id
pub unsafe extern "C" fn get_method_id(env: *mut JNIEnv,
                                       clazz: jclass,
                                       name: *const c_char,
                                       sig: *const c_char)
                                       -> jmethodID {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_len = libc::strlen(name);
    let mut method_name = String::with_capacity(name_len);
    for i in 0..name_len {
        method_name.push(name.add(i).read() as u8 as char);
    }

    let desc_len = libc::strlen(sig);
    //todo dup
    let mut method_descriptor_str = String::with_capacity(desc_len);
    for i in 0..desc_len {
        method_descriptor_str.push(sig.add(i).read() as u8 as char);
    }

    let runtime_class = from_jclass(clazz).as_runtime_class();
    let all_methods = get_all_methods(jvm, int_state, runtime_class);

    let (_method_i, (c, m)) = all_methods.iter().enumerate().find(|(_, (c, i))| {
        let method_view = &c.view().method_view_i(*i);
        let cur_desc = method_view.desc_str();
        let cur_method_name = method_view.name();
        cur_method_name == method_name &&
            method_descriptor_str == cur_desc
    }).unwrap();
    let method_id = jvm.method_table.write().unwrap().get_method_id(c.clone(), *m as u16);
    transmute(method_id)
}
