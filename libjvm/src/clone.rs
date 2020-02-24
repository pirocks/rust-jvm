use jni_bindings::{jobject, JNIEnv};
use slow_interpreter::rust_jni::native_util::{to_object, from_object};
use std::sync::Arc;
use runtime_common::java_values::{Object, ArrayObject, NormalObject};
use std::ops::Deref;
use std::cell::RefCell;
use rust_jvm_common::view::ptype_view::PTypeView::Ref;

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    let to_clone = from_object(obj);
    to_object(match to_clone {
        None => unimplemented!(),
        Some(o) => {
            match o.deref() {
                Object::Array(a) => {
                    let cloned_arr : Vec<_>= a.elems.borrow().iter().cloned().collect();
                    Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(cloned_arr), elem_type: a.elem_type.clone() })))
                },
                Object::Object(o) => {
                    Arc::new(Object::Object(NormalObject {
                        gc_reachable: o.gc_reachable,
                        fields: RefCell::new(o.fields.borrow().iter().map(|(k,v)|{(k.clone(),v.clone())}).collect()),
                        class_pointer: o.class_pointer.clone(),
                        bootstrap_loader: o.bootstrap_loader,
                        object_class_object_pointer: o.object_class_object_pointer.clone(),
                        array_class_object_pointer: o.array_class_object_pointer.clone()
                    })).into()
                },
            }
        },
    })
}