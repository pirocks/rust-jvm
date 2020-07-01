use crate::{JVMState, StackEntry};

use verification::verifier::instructions::branches::get_method_descriptor;
use std::sync::Arc;

use crate::interpreter_util::check_inited_class;
use crate::utils::lookup_method_parsed;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::loading::LoaderArc;
use crate::java_values::{JavaValue, Object, ArrayObject};
use crate::runtime_class::RuntimeClass;
use descriptor_parser::MethodDescriptor;


pub mod special;
pub mod native;
pub mod interface;
pub mod virtual_;
pub mod static_;

pub mod dynamic {


    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use classfile_view::view::constant_info_view::{ConstantInfoView, ReferenceData, InvokeStatic};
    use crate::{JVMState, StackEntry};
    use classfile_view::view::attribute_view::BootstrapArgView;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::string::JString;
    use crate::java::lang::invoke::method_handle::MethodHandle;

    pub fn invoke_dynamic(jvm: &'static JVMState, frame: &mut StackEntry, cp: u16) {
        let _method_handle_class = check_inited_class(
            jvm,
            &ClassName::method_handle().into(),
            frame.class_pointer.loader(jvm).clone(),
        );
        let _method_type_class = check_inited_class(
            jvm,
            &ClassName::method_type().into(),
            frame.class_pointer.loader(jvm).clone(),
        );
        let _call_site_class = check_inited_class(
            jvm,
            &ClassName::Str("java/lang/invoke/CallSite".to_string()).into(),
            frame.class_pointer.loader(jvm).clone(),
        );
        let class_pointer_view = frame.class_pointer.view().clone();
        let invoke_dynamic_view = match class_pointer_view.constant_pool_view(cp as usize) {
            ConstantInfoView::InvokeDynamic(id) => id,
            _ => panic!(),
        };

        let bootstrap_method_view = invoke_dynamic_view.bootstrap_method();
        let _method_handle = {
            let methodref_view = bootstrap_method_view.bootstrap_method_ref();
            match methodref_view.get_reference_data(){
                ReferenceData::InvokeStatic(is) => {
                    match is {
                        InvokeStatic::Interface(_) => unimplemented!(),
                        InvokeStatic::Method(mr) => {
                            let lookup = MethodHandle::public_lookup(jvm, frame);
                            // let _a_rando_class_object = lookup.get_class(state, frame.clone());
                            // dbg!(&a_rando_class_object.clone().java_value().unwrap_normal_object().fields);
                            // let loader = a_rando_class_object.get_class_loader(state, &frame);
                            let name = JString::from(jvm, &frame, mr.name_and_type().name());
                            let desc = JString::from(jvm, &frame, mr.name_and_type().desc_str());
                            let method_type = MethodType::from_method_descriptor_string(jvm, frame, desc, None);
                            let target_class = JClass::from_name(jvm, frame, mr.class());
                            lookup.find_virtual(jvm, frame, target_class, name, method_type)
                        },
                    }
                },
            }

        };
        let arg_iterator = bootstrap_method_view.bootstrap_args();
        arg_iterator.for_each(|x|{
            match x {
                BootstrapArgView::String(_) => unimplemented!(),
                BootstrapArgView::Class(_) => unimplemented!(),
                BootstrapArgView::Integer(_) => unimplemented!(),
                BootstrapArgView::Long(_) => unimplemented!(),
                BootstrapArgView::Float(_) => unimplemented!(),
                BootstrapArgView::Double(_) => unimplemented!(),
                BootstrapArgView::MethodHandle(_) => unimplemented!(),
                BootstrapArgView::MethodType(_) => unimplemented!()
            };
        });


        //A call site specifier gives a symbolic reference to a method handle which is to serve as
        // the bootstrap method for a dynamic call site (§4.7.23).The method handle is resolved to
        // obtain a reference to an instance of java.lang.invoke.MethodHandle (§5.4.3.5)
        // invoke_dynamic_view.name_and_type()
        // let bootstrap_method = invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref();
        // invoke_dynamic_view.bootstrap_method_attr().bootstrap_args();
        // let _bootstrap_method_class = check_inited_class(state, &bootstrap_method.class(), current_ current_frame.class_pointer.loader(jvm).clone());
        // dbg!(invoke_dynamic_view.name_and_type().name());
        // dbg!(invoke_dynamic_view.name_and_type().desc());
        // dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().name_and_type());
        // dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().class());

//        invoke_dynamic_view.


        // dbg!(&current_frame.class_pointer.classfile.constant_pool[cp as usize]);
        unimplemented!()
    }
}

fn resolved_class(jvm: &'static JVMState, current_frame: &mut StackEntry, cp: u16) -> Option<(Arc<RuntimeClass>, String, MethodDescriptor)> {
    let view = current_frame.class_pointer.view();
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, view);
    let class_name_ = match class_name_type {
        PTypeView::Ref(r) => {
            match r {
                ReferenceTypeView::Class(c) => c,
                ReferenceTypeView::Array(_a) => {
                    if expected_method_name == "clone".to_string() {
                        //todo replace with proper native impl
                        let temp = current_frame.pop().unwrap_object().unwrap();
                        let to_clone_array = temp.unwrap_array();
                        current_frame.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject {
                            elems: to_clone_array.elems.clone(),
                            elem_type: to_clone_array.elem_type.clone(),
                            monitor: jvm.thread_state.new_monitor("monitor for cloned object".to_string())
                        })))));
                        return None;
                    } else {
                        unimplemented!();
                    }
                }
            }
        }
        _ => panic!()
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let resolved_class = check_inited_class(
        jvm,
        &class_name_.into(),
        loader_arc.clone()
    );
    (resolved_class, expected_method_name, expected_descriptor).into()
}

pub fn find_target_method(
    state: &'static JVMState,
    loader_arc: LoaderArc,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>,
) -> (usize, Arc<RuntimeClass>) {
    lookup_method_parsed(state, target_class, expected_method_name, parsed_descriptor, &loader_arc).unwrap()
}