use jvmti_jni_bindings::{jobject, JNIEnv};
use slow_interpreter::rust_jni::native_util::{to_object, from_object, get_state};
use std::sync::Arc;
use std::ops::Deref;
use std::cell::RefCell;
use slow_interpreter::java_values::{Object, ArrayObject, NormalObject};


#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    let jvm = get_state(env);
    let to_clone = from_object(obj);
    to_object(match to_clone {
        None => unimplemented!(),
        Some(o) => {
            match o.deref() {
                Object::Array(a) => {
                    let cloned_arr : Vec<_>= a.elems.borrow().iter().cloned().collect();
                    Some(Arc::new(Object::Array(ArrayObject {
                        elems: RefCell::new(cloned_arr),
                        elem_type: a.elem_type.clone(),
                        monitor: jvm.thread_state.new_monitor("".to_string())
                    })))
                },
                Object::Object(o) => {
                    Arc::new(Object::Object(NormalObject {
                        monitor: jvm.thread_state.new_monitor("".to_string()),
                        fields: RefCell::new(o.fields.borrow().iter().map(|(k,v)|{(k.clone(),v.clone())}).collect()),
                        class_pointer: o.class_pointer.clone(),
                        class_object_type: o.class_object_type.clone()
                    })).into()
                },
            }
        },
    })
}
