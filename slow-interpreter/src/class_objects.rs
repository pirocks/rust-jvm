use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::loading::ClassLoadingError;
use classfile_view::view::ptype_view::PTypeView;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_resolved_class;
use crate::java_values::Object;

//todo do something about this class object crap
pub fn get_or_create_class_object(jvm: &JVMState,
                                  type_: PTypeView,
                                  int_state: &mut InterpreterStateGuard,
) -> Result<Arc<Object>, ClassLoadingError> {
    let arc = check_resolved_class(jvm, int_state, type_)?;
    // dbg!(arc.view().name());
    // int_state.print_stack_trace();
    Ok(jvm.classes.write().unwrap().class_object_pool.get_by_right(&ByAddress(arc.clone())).unwrap().clone().0)
}

// fn regular_class_object(jvm: &JVMState, ptype: PTypeView, int_state: &mut InterpreterStateGuard, loader: LoaderName, override_: bool) -> Result<Arc<Object>, ClassLoadingError> {
//     // let current_frame = int_state.current_frame_mut();
//
//     let runtime_class = todo!();
//     // assert_eq!(runtime_class.loader(),loader);
//     let mut classes = jvm.classes.write().unwrap();
//     let res = classes.class_object_pool.entry(runtime_class.loader()).or_default().get(&ptype).cloned();
//     Ok(match res {
//         None => {
//             drop(classes);
//             let r = create_a_class_object(jvm, int_state, runtime_class.clone());
//             let mut classes = jvm.classes.write().unwrap();
//             //todo likely race condition created by expectation that Integer.class == Integer.class, maybe let it happen anyway?
//             classes.class_object_pool.entry(int_state.current_loader()).or_default().insert(ptype.clone(), r.clone());
//             drop(classes);//todo get rid of these manual drops
//             if runtime_class.ptypeview().is_primitive() {
//                 //handles edge case of classes whose names do not correspond to the name of the class they represent
//                 //normally names are obtained with getName0 which gets handled in libjvm.so
//                 let jstring = JString::from_rust(jvm, int_state, runtime_class.ptypeview().primitive_name().to_string());
//                 r.unwrap_normal_object().fields_mut().insert("name".to_string(), jstring.java_value());
//             }
//             let loader_val = match runtime_class.loader() {
//                 LoaderName::UserDefinedLoader(idx) => {
//                     JavaValue::Object(Some(jvm.class_loaders.read().unwrap().get_by_left(&idx).unwrap().0.clone()))
//                 }
//                 LoaderName::BootstrapLoader => JavaValue::Object(None)
//             };
//             match ptype {
//                 PTypeView::Ref(ref_) => {
//                     match ref_ {
//                         ReferenceTypeView::Class(name) => {
//                             r.unwrap_normal_object().fields_mut().insert("name".to_string(), JString::from_rust(jvm, int_state, name.get_referred_name().replace("/", ".")).java_value());
//                         }
//                         ReferenceTypeView::Array(_) => {}
//                     }
//                 }
//                 _ => {}
//             }
//             r.unwrap_normal_object().fields_mut().insert("classLoader".to_string(), loader_val);
//             r
//         }
//         Some(r) => r,
//     })
// }
//
// fn create_a_class_object(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptypev: Arc<RuntimeClass>) -> Arc<Object> {
//     let java_lang_class = ClassName::class();
//     let class_class = assert_inited_or_initing_class(jvm, int_state, java_lang_class.into());
//     let boostrap_loader_object = jvm.get_or_create_bootstrap_object_loader(int_state);
//     //the above would only be required for higher jdks where a class loader object is part of Class.
//     //as it stands we can just push to operand stack
//     push_new_object(jvm, int_state, &class_class);
//     let object = int_state.pop_current_operand_stack();
//     match object.clone() {
//         JavaValue::Object(o) => {
//             let bootstrap_arc = boostrap_loader_object.clone();
//             if boostrap_loader_object.unwrap_object().is_some()
//             {
//                 bootstrap_arc.unwrap_normal_object().fields_mut().insert("assertionLock".to_string(), boostrap_loader_object.clone());//itself...
//                 bootstrap_arc.unwrap_normal_object().fields_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
//                 o.as_ref().unwrap().unwrap_normal_object().fields_mut().insert("classLoader".to_string(), JavaValue::Object(None));
//             }
//             if !jvm.system_domain_loader {
//                 o.as_ref().unwrap().unwrap_normal_object().fields_mut().insert("classLoader".to_string(), boostrap_loader_object);
//             }
//         }
//         _ => panic!(),
//     }
//     object.unwrap_object_nonnull()
// }
