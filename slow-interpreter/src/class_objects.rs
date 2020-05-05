use crate::JVMState;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::{Object, JavaValue};
use std::sync::Arc;
use classfile_view::loading::LoaderArc;
use crate::stack_entry::StackEntry;

use rust_jvm_common::ptype::PType;
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::instructions::ldc::create_string_on_stack;
use rust_jvm_common::classnames::ClassName;
use std::ops::Deref;
use crate::runtime_class::RuntimeClass;

//todo do something about this class object crap
pub fn get_or_create_class_object(state: &JVMState,
                                  type_: &PTypeView,
                                  current_frame: &StackEntry,
                                  loader_arc: LoaderArc,
) -> Arc<Object> {
    match type_ {
        PTypeView::Ref(t) => match t {
            ReferenceTypeView::Array(c) => {
                return array_object(state, c.deref(), current_frame);
            }
            _ => {}
        },
        _ => {}
    }

    regular_object(state, type_, current_frame, loader_arc)
}

fn array_object(state: &JVMState, array_sub_type: &PTypeView, current_frame: &StackEntry) -> Arc<Object> {
    let type_for_object: PType = array_sub_type.to_ptype();
    array_of_type_class(state, current_frame, &type_for_object)
}

pub fn array_of_type_class(state: &JVMState, current_frame: &StackEntry, type_for_object: &PType) -> Arc<Object> {
    //todo wrap in array and convert
    let array_type = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::from_ptype(type_for_object).into()));
    let res = state.class_object_pool.read().unwrap().get(&array_type).cloned();
    match res {
        None => {
            let array_ptype_view = array_type.clone();
            let r = create_a_class_object(state, current_frame, array_ptype_view);
            state.class_object_pool.write().unwrap().insert(array_type, r.clone());//todo race condition see below
            r
        }
        Some(r) => r.clone(),
    }
}

fn regular_object(state: &JVMState, class_type: &Arc<RuntimeClass>, current_frame: &StackEntry, loader_arc: LoaderArc) -> Arc<Object> {
    check_inited_class(state, class_type.unwrap_type_to_name().as_ref().unwrap(), loader_arc);
    let res = state.class_object_pool.read().unwrap().get(&class_type.ptypeview()).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, current_frame, class_type.clone());
            //todo likely race condition created by expectation that Integer.class == Integer.class, maybe let it happen anyway?
            state.class_object_pool.write().unwrap().insert(class_type.ptypeview(), r.clone());
            if class_type.is_primitive() {
                //handles edge case of classes whose names do not correspond to the name of the class they represent
                //normally names are obtained with getName0 which gets handled in libjvm.so
                create_string_on_stack(state, class_type.primitive_name().to_string());
                r.unwrap_normal_object().fields.borrow_mut().insert("name".to_string(), current_frame.pop());
            }
            r
        }
        Some(r) => r.clone(),
    }
}

fn create_a_class_object(jvm: &JVMState, current_frame: &StackEntry, ptypev: Arc<RuntimeClass>) -> Arc<Object> {
    let java_lang_class = ClassName::class();
    let current_loader = current_frame.class_pointer.loader(jvm).clone();
    let class_class = check_inited_class(jvm, &java_lang_class, current_loader.clone());
    let boostrap_loader_object = jvm.get_or_create_bootstrap_object_loader();
    //the above would only be required for higher jdks where a class loader object is part of Class.
    //as it stands we can just push to operand stack
    push_new_object(jvm, current_frame, &class_class, ptypev.into());
    let object = current_frame.pop();
    match object.clone() {
        JavaValue::Object(o) => {
            let bootstrap_arc = boostrap_loader_object.clone();
            if boostrap_loader_object.unwrap_object().is_some()
            {
                bootstrap_arc.unwrap_normal_object().fields.borrow_mut().insert("assertionLock".to_string(), boostrap_loader_object.clone());//itself...
                bootstrap_arc.unwrap_normal_object().fields.borrow_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
                o.as_ref().unwrap().unwrap_normal_object().fields.borrow_mut().insert("classLoader".to_string(), JavaValue::Object(None));
            }
            if !jvm.system_domain_loader {
                o.as_ref().unwrap().unwrap_normal_object().fields.borrow_mut().insert("classLoader".to_string(), boostrap_loader_object.clone());
            }
        }
        _ => panic!(),
    }
    let r = object.unwrap_object_nonnull();
    r
}
