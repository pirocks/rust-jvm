use std::cell::{RefCell, UnsafeCell};
use std::ops::Deref;

use itertools::Itertools;

use jvmti_jni_bindings::{JNIEnv, jobject};
use slow_interpreter::java_values::{ArrayObject, NormalObject, Object, ObjectFieldsAndClass};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::sun::misc::unsafe_::Unsafe;

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let to_clone = from_object(jvm, obj);
    new_local_ref_public(
        match to_clone {
            None => unimplemented!(),
            Some(o) => {
                match o.deref() {
                    Object::Array(a) => {
                        // let cloned_arr: Vec<_> = /*a.elems.get().as_ref().unwrap().iter().map(|elem| elem.clone()).collect_vec()*/todo!();
                        Some(jvm.allocate_object(Object::Array(ArrayObject {
                            // elems: UnsafeCell::new(cloned_arr),
                            whole_array_runtime_class: a.whole_array_runtime_class.clone(),
                            loader: a.loader.clone(),
                            len: todo!(),
                            elems_base: todo!(),
                            phantom_data: Default::default(),
                            elem_type: a.elem_type.clone(),
                            // monitor: jvm.thread_state.new_monitor("".to_string()),
                        })))
                    }
                    Object::Object(o) => {
                        jvm.allocate_object(Object::Object(NormalObject {
                            // monitor: jvm.thread_state.new_monitor("".to_string()),
                            objinfo: ObjectFieldsAndClass {
                                fields: todo!(),
                                /*o.objinfo.fields.iter().map(|val| UnsafeCell::new(val.get().as_ref().unwrap().clone())).collect()*/
                                class_pointer: o.objinfo.class_pointer.clone(),
                            },
                            obj_ptr: todo!(),
                        }))
                            .into()
                    }
                }
            }
        },
        int_state,
    )
}