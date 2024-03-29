use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use itertools::Itertools;
use libc::time;

use jvmti_jni_bindings::{JNIEnv, jobject};
use runtime_class_stuff::{FieldNameAndFieldType, RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use runtime_class_stuff::hidden_fields::HiddenJVMFieldAndFieldType;

use slow_interpreter::class_loading::assert_inited_or_initing_class;
use slow_interpreter::java_values::{ArrayObject, NormalObject, Object, ObjectFieldsAndClass};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::new_java_values::unallocated_objects::{ObjectFields, UnAllocatedObject, UnAllocatedObjectArray, UnAllocatedObjectObject};


use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object};
use slow_interpreter::stdlib::sun::misc::unsafe_::Unsafe;

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
                let hidden_owned_copied_fields = copy_hidden_fields(jvm, o.unwrap_normal_object_ref(), rc.unwrap_class_class());
                let fields = owned_copied_fields.iter().map(|(number, handle)| (*number, handle.as_njv())).collect();
                let hidden_fields = hidden_owned_copied_fields.iter().map(|(number, handle)| (*number, handle.as_njv())).collect();
                let cloned = jvm.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject {
                    object_rc: rc,
                    object_fields: ObjectFields {
                        fields,
                        hidden_fields,
                    },
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
    for (number, FieldNameAndFieldType { name, cpdtype }) in rc.object_layout.field_numbers_reverse.iter() {
        res.insert(*number, obj.raw_get_var(jvm, *number, *cpdtype));
    }
    res
}


pub fn copy_hidden_fields<'gc>(jvm: &'gc JVMState<'gc>, obj: &AllocatedNormalObjectHandle<'gc>, rc: &RuntimeClassClass<'gc>) -> HashMap<FieldNumber, NewJavaValueHandle<'gc>> {
    let mut res = HashMap::new();
    for (number, HiddenJVMFieldAndFieldType { name, cpdtype }) in rc.object_layout.hidden_field_numbers_reverse.iter(){
        res.insert(*number, obj.raw_get_var(jvm, *number, *cpdtype));
    }
    if !res.is_empty(){
        assert!(rc.class_view.is_final());//should be final b/c hidden fields don't work with inheritance
    }
    res
}