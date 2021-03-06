use std::cell::{RefCell, UnsafeCell};
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::vtype::VType::Uninitialized;
use jvmti_jni_bindings::{JNIEnv, jobject};
use slow_interpreter::java_values::{ArrayObject, NormalObject, Object};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::sun::misc::unsafe_::Unsafe;

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let to_clone = from_object(obj);
    new_local_ref_public(match to_clone {
        None => unimplemented!(),
        Some(o) => {
            match o.deref() {
                Object::Array(a) => {
                    let cloned_arr: Vec<_> = a.mut_array().iter().cloned().collect();
                    Some(Arc::new(Object::Array(ArrayObject {
                        elems: UnsafeCell::new(cloned_arr),
                        elem_type: a.elem_type.clone(),
                        monitor: jvm.thread_state.new_monitor("".to_string()),
                    })))
                }
                Object::Object(o) => {
                    Arc::new(Object::Object(NormalObject {
                        monitor: jvm.thread_state.new_monitor("".to_string()),
                        fields: UnsafeCell::new(o.fields_mut().iter().map(|(k, v)| { (k.clone(), v.clone()) }).collect()),
                        class_pointer: o.class_pointer.clone(),
                    })).into()
                }
            }
        }
    }, int_state)
}
