use std::collections::HashMap;
use std::sync::Arc;

use classfile_view::loading::{ClassLoadingError, LoaderName};
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, Object};
use crate::runtime_class::RuntimeClass;

//todo do something about this class object crap
pub fn get_or_create_class_object(jvm: &JVMState,
                                  type_: &PTypeView,
                                  int_state: &mut InterpreterStateGuard,
) -> Result<Arc<Object>, ClassLoadingError> {
    get_or_creat_class_object_override_loader(jvm, type_, int_state, int_state.current_loader())
}

pub fn get_or_creat_class_object_override_loader(jvm: &JVMState,
                                                 type_: &PTypeView,
                                                 int_state: &mut InterpreterStateGuard,
                                                 loader: LoaderName) -> Result<Arc<Object>, ClassLoadingError> {
    regular_class_object(jvm, type_.clone(), int_state, loader)
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

fn regular_class_object(jvm: &JVMState, ptype: PTypeView, int_state: &mut InterpreterStateGuard, loader: LoaderName) -> Result<Arc<Object>, ClassLoadingError> {
    // let current_frame = int_state.current_frame_mut();
    let runtime_class = check_inited_class(jvm, int_state, ptype.clone())?;
    let mut classes = jvm.classes.write().unwrap();
    let res = classes.class_object_pool.entry(loader).or_default().get(&ptype).cloned();
    Ok(match res {
        None => {
            drop(classes);
            let r = create_a_class_object(jvm, int_state, runtime_class.clone());
            let mut classes = jvm.classes.write().unwrap();
            //todo likely race condition created by expectation that Integer.class == Integer.class, maybe let it happen anyway?
            classes.class_object_pool.entry(int_state.current_loader()).or_default().insert(ptype, r.clone());
            drop(classes);//todo get rid of these manual drops
            if runtime_class.ptypeview().is_primitive() {
                //handles edge case of classes whose names do not correspond to the name of the class they represent
                //normally names are obtained with getName0 which gets handled in libjvm.so
                let jstring = JString::from_rust(jvm, int_state, runtime_class.ptypeview().primitive_name().to_string());
                // dbg!(&jstring.to_rust_string());
                r.unwrap_normal_object().fields_mut().insert("name".to_string(), jstring.java_value());
                let classes_guard = jvm.classes.read().unwrap();
                let bl = LoaderName::BootstrapLoader;
                let initiating_loader = classes_guard.initiating_loaders.get(&runtime_class).unwrap_or(&bl);
                let loader_val = match initiating_loader {
                    LoaderName::UserDefinedLoader(idx) => {
                        JavaValue::Object(Some(jvm.class_loaders.read().unwrap().get_by_left(idx).unwrap().0.clone()))
                    }
                    LoaderName::BootstrapLoader => JavaValue::Object(None)
                };
                r.unwrap_normal_object().fields_mut().insert("classLoader".to_string(), loader_val);
            }/*else if !runtime_class.ptypeview().is_array() {
                let jstring = JString::from(state, int_state, runtime_class.ptypeview().unwrap_class_type().get_referred_name().to_string());
                r.unwrap_normal_object().fields_mut().insert("name".to_string(), jstring.java_value());
            }*/
            r
        }
        Some(r) => r,
    })
}

fn create_a_class_object(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptypev: Arc<RuntimeClass>) -> Arc<Object> {
    let java_lang_class = ClassName::class();
    let class_class = check_inited_class(jvm, int_state, java_lang_class.into()).unwrap();
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
                bootstrap_arc.unwrap_normal_object().fields_mut().insert("assertionLock".to_string(), boostrap_loader_object.clone());//itself...
                bootstrap_arc.unwrap_normal_object().fields_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
                o.as_ref().unwrap().unwrap_normal_object().fields_mut().insert("classLoader".to_string(), JavaValue::Object(None));
            }
            if !jvm.system_domain_loader {
                o.as_ref().unwrap().unwrap_normal_object().fields_mut().insert("classLoader".to_string(), boostrap_loader_object);
            }
        }
        _ => panic!(),
    }
    object.unwrap_object_nonnull()
}
