use std::cell::{RefCell, UnsafeCell};
use std::ops::Deref;

use itertools::Itertools;
use libc::time;

use jvmti_jni_bindings::{JNIEnv, jobject};
use rust_jvm_common::compressed_classfile::code::CompressedInstructionInfo::new;
use slow_interpreter::class_loading::assert_inited_or_initing_class;
use slow_interpreter::java_values::{ArrayObject, NormalObject, Object, ObjectFieldsAndClass};
use slow_interpreter::new_java_values::{UnAllocatedObject, UnAllocatedObjectArray};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state, to_object};
use slow_interpreter::sun::misc::unsafe_::Unsafe;

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let to_clone = from_object_new(jvm, obj);
    new_local_ref_public_new(
        match to_clone {
            None => unimplemented!(),
            Some(o) => {
                if o.is_array(jvm) {
                    let mut new_array = vec![];
                    let to_clone_array = o.unwrap_array(jvm);
                    for elem in to_clone_array.array_iterator() {
                        new_array.push(elem);
                    }
                    return new_local_ref_public_new(Some(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray {
                        whole_array_runtime_class: o.as_allocated_obj().runtime_class(jvm),
                        elems: new_array.iter().map(|handle| handle.as_njv()).collect(),
                    })).as_allocated_obj()),int_state)
                } else {
                    todo!()
                }
                /*                match o.deref() {
                                    Object::Array(a) => {
                                        // let cloned_arr: Vec<_> = /*a.elems.get().as_ref().unwrap().iter().map(|elem| elem.clone()).collect_vec()*/todo!();
                                        todo!()
                /*                        Some(jvm.allocate_object(todo!()/*Object::Array(ArrayObject {
                                            // elems: UnsafeCell::new(cloned_arr),
                                            whole_array_runtime_class: a.whole_array_runtime_class.clone(),
                                            loader: a.loader.clone(),
                                            len: todo!(),
                                            elems_base: todo!(),
                                            phantom_data: Default::default(),
                                            elem_type: a.elem_type.clone(),
                                            // monitor: jvm.thread_state.new_monitor("".to_string()),
                                        })*/))
                */                    }
                                    Object::Object(o) => {
                                        todo!()
                /*                        jvm.allocate_object(Object::Object(NormalObject {
                                            // monitor: jvm.thread_state.new_monitor("".to_string()),
                                            objinfo: ObjectFieldsAndClass {
                                                fields: todo!(),
                                                /*o.objinfo.fields.iter().map(|val| UnsafeCell::new(val.get().as_ref().unwrap().clone())).collect()*/
                                                class_pointer: o.objinfo.class_pointer.clone(),
                                            },
                                            obj_ptr: todo!(),
                                        }))
                */
                                    }
                                }
                */
            }
        },
        int_state,
    )
}