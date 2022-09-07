use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use itertools::Itertools;
use libc::time;

use jvmti_jni_bindings::{JNIEnv, jobject};
use runtime_class_stuff::{FieldNameAndFieldType, RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use rust_jvm_common::compressed_classfile::code::CompressedInstructionInfo::new;
use slow_interpreter::class_loading::assert_inited_or_initing_class;
use slow_interpreter::java_values::{ArrayObject, NormalObject, Object, ObjectFieldsAndClass};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray, UnAllocatedObjectObject};
use slow_interpreter::rust_jni::interface::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object};
use slow_interpreter::sun::misc::unsafe_::Unsafe;

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let to_clone = from_object_new(jvm, obj);
    match to_clone {
        None => unimplemented!(),
        Some(o) => {
            if o.is_array(jvm) {
                let mut new_array = vec![];
                let to_clone_array = o.unwrap_array();
                for elem in to_clone_array.array_iterator() {
                    new_array.push(elem);
                }
                return new_local_ref_public_new(Some(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray {
                    whole_array_runtime_class: o.runtime_class(jvm),
                    elems: new_array.iter().map(|handle| handle.as_njv()).collect(),
                })).as_allocated_obj()), int_state);
            } else {
                let rc = o.unwrap_normal_object_ref().runtime_class(jvm);
                let owned_copied_fields = copy_fields(jvm, o.unwrap_normal_object_ref(), rc.unwrap_class_class());
                let cloned = jvm.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject {
                    object_rc: rc,
                    fields: owned_copied_fields.iter().map(|(number, handle)| (*number, handle.as_njv())).collect(),
                }));
                return new_local_ref_public_new(Some(cloned.as_allocated_obj()), int_state);
            }
        }
    }
}

pub fn copy_fields<'gc>(jvm: &'gc JVMState<'gc>, obj: &AllocatedNormalObjectHandle<'gc>, rc: &RuntimeClassClass<'gc>) -> HashMap<FieldNumber, NewJavaValueHandle<'gc>> {
    let mut res = HashMap::new();
    if let Some(parent) = rc.parent.as_ref() {
        res.extend(copy_fields(jvm, obj, parent.unwrap_class_class()).into_iter());
    }
    for (number, FieldNameAndFieldType { name, cpdtype }) in rc.field_numbers_reverse.iter() {
        res.insert(*number, obj.raw_get_var(jvm, *number, *cpdtype));
    }
    res
}