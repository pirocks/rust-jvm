use crate::{InterpreterState, StackEntry};
use std::rc::Rc;
use verification::verifier::instructions::branches::get_method_descriptor;
use std::sync::Arc;

use crate::interpreter_util::check_inited_class;
use crate::utils::lookup_method_parsed;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::view::ClassView;
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
    use std::rc::Rc;

    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use classfile_view::view::constant_info_view::{ConstantInfoView, ReferenceData, InvokeStatic};
    use crate::{InterpreterState, StackEntry};
    use classfile_view::view::attribute_view::BootstrapArgView;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::string::JString;
    use crate::java::lang::invoke::method_handle::MethodHandle;

    pub fn invoke_dynamic(state: &mut InterpreterState, frame: Rc<StackEntry>, cp: u16) {
        let method_handle_class = check_inited_class(
            state,
            &ClassName::method_handle(),
            frame.clone().into(),
            frame.class_pointer.loader.clone(),
        );
        let method_type_class = check_inited_class(
            state,
            &ClassName::method_type(),
            frame.clone().into(),
            frame.class_pointer.loader.clone(),
        );
        let invoke_dynamic_view = match frame.class_pointer.class_view.constant_pool_view(cp as usize) {
            ConstantInfoView::InvokeDynamic(id) => id,
            _ => panic!(),
        };
        frame.print_stack_trace();
        let method_handle = {
            let methodref_view = invoke_dynamic_view.bootstrap_method().bootstrap_method_ref();
            match methodref_view.get_reference_data(){
                ReferenceData::InvokeStatic(is) => {
                    match is {
                        InvokeStatic::Interface(_) => unimplemented!(),
                        InvokeStatic::Method(mr) => {
                            let lookup = MethodHandle::public_lookup(state, &frame);
                            let a_rando_class_object = lookup.get_class(state, frame.clone());
                            // dbg!(&a_rando_class_object.clone().java_value().unwrap_normal_object().fields);
                            // let loader = a_rando_class_object.get_class_loader(state, &frame);
                            let name = JString::from(state, &frame, mr.name_and_type().name());
                            let desc = JString::from(state, &frame, mr.name_and_type().desc());
                            let method_type = MethodType::from_method_descriptor_string(state, &frame, desc, None);
                            let target_class = JClass::from_name(state, &frame, mr.class());
                            lookup.find_virtual(state, &frame, target_class, name, method_type)
                        },
                    }
                },
            }

        };
        let arg_iterator = invoke_dynamic_view.bootstrap_method().bootstrap_args();
        arg_iterator.map(|x|{
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
        // let _bootstrap_method_class = check_inited_class(state, &bootstrap_method.class(), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
        // dbg!(invoke_dynamic_view.name_and_type().name());
        // dbg!(invoke_dynamic_view.name_and_type().desc());
        // dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().name_and_type());
        // dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().class());

//        invoke_dynamic_view.


        // dbg!(&current_frame.class_pointer.classfile.constant_pool[cp as usize]);
        unimplemented!()
    }
}

fn resolved_class(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) -> Option<(Arc<RuntimeClass>, String, MethodDescriptor)> {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(classfile.clone()));
    let class_name_ = match class_name_type {
        PTypeView::Ref(r) => {
            match r {
                ReferenceTypeView::Class(c) => c,
                ReferenceTypeView::Array(_a) => {
                    if expected_method_name == "clone".to_string() {
                        //todo replace with proper native impl
                        let temp = current_frame.pop().unwrap_object().unwrap();
                        let to_clone_array = temp.unwrap_array();
                        current_frame.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: to_clone_array.elems.clone(), elem_type: to_clone_array.elem_type.clone() })))));
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
    let resolved_class = check_inited_class(state, &class_name_, current_frame.clone().into(), loader_arc.clone());
    (resolved_class, expected_method_name, expected_descriptor).into()
}

pub fn find_target_method(
    state: &mut InterpreterState,
    loader_arc: LoaderArc,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>,
) -> (usize, Arc<RuntimeClass>) {
    //todo bug need to handle super class, issue with that is need frame/state.
    lookup_method_parsed(state, target_class, expected_method_name, parsed_descriptor, &loader_arc).unwrap()
}