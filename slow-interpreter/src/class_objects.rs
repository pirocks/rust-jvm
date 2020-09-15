use std::sync::Arc;

use classfile_view::loading::LoaderArc;
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, Object};
use crate::runtime_class::RuntimeClass;

//todo do something about this class object crap
pub fn get_or_create_class_object(state: &JVMState,
                                  type_: &PTypeView,
                                  int_state: &mut InterpreterStateGuard,
                                  loader_arc: LoaderArc,
) -> Arc<Object> {
    // match type_ {
    //     PTypeView::Ref(t) => match t {
    //         ReferenceTypeView::Array(c) => {
    //             return array_object(state, c.deref(), current_frame);
    //         }
    //         _ => {}
    //     },
    //     _ => {}
    // }

    regular_object(state, type_.clone(), int_state, loader_arc)
}

// fn array_object(state: &JVMState, array_sub_type: &PTypeView, current_frame: &StackEntry) -> Arc<Object> {
//     let type_for_object= array_sub_type.to_ptype();
//     array_of_type_class(state, current_frame, type_for_object)
// }
//
// pub fn array_of_type_class(state: &JVMState, current_frame: &StackEntry, type_for_object: RuntimeClass) -> Arc<Object> {
//     //todo wrap in array and convert
//     let array = RuntimeClass::Array(RuntimeClassArray { sub_class: Arc::new(type_for_object) });
//     let res = state.class_object_pool.read().unwrap().get(&array).cloned();
//     match res {
//         None => {
//             let arc = Arc::new(array);
//             let r = create_a_class_object(state, current_frame, arc.clone());
//             state.class_object_pool.write().unwrap().insert(arc, r.clone());//todo race condition see below
//             r
//         }
//         Some(r) => r.clone(),
//     }
// }

fn regular_object<'l>(state: &JVMState, ptype: PTypeView, int_state: &mut InterpreterStateGuard, loader_arc: LoaderArc) -> Arc<Object> {
    // let current_frame = int_state.current_frame_mut();
    let runtime_class = check_inited_class(state, int_state, &ptype, loader_arc);
    let res = state.classes.class_object_pool.read().unwrap().get(&ptype).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, int_state, runtime_class.clone());
            //todo likely race condition created by expectation that Integer.class == Integer.class, maybe let it happen anyway?
            state.classes.class_object_pool.write().unwrap().insert(ptype.clone(), r.clone());
            if runtime_class.ptypeview().is_primitive() {
                //handles edge case of classes whose names do not correspond to the name of the class they represent
                //normally names are obtained with getName0 which gets handled in libjvm.so
                let jstring = JString::from_rust(state, int_state, runtime_class.ptypeview().primitive_name().to_string());
                r.unwrap_normal_object().fields.borrow_mut().insert("name".to_string(), jstring.java_value());
            }/*else if !runtime_class.ptypeview().is_array() {
                let jstring = JString::from(state, int_state, runtime_class.ptypeview().unwrap_class_type().get_referred_name().to_string());
                r.unwrap_normal_object().fields.borrow_mut().insert("name".to_string(), jstring.java_value());
            }*/
            r
        }
        Some(r) => r.clone(),
    }
}

fn create_a_class_object(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptypev: Arc<RuntimeClass>) -> Arc<Object> {
    let java_lang_class = ClassName::class();
    let current_loader = int_state.current_loader(jvm).clone();
    let class_class = check_inited_class(jvm, int_state, &java_lang_class.into(), current_loader.clone());
    let boostrap_loader_object = jvm.get_or_create_bootstrap_object_loader(int_state);
    //the above would only be required for higher jdks where a class loader object is part of Class.
    //as it stands we can just push to operand stack
    push_new_object(jvm, int_state, &class_class, ptypev.into());
    let object = int_state.pop_current_operand_stack();
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
