use crate::InterpreterState;
use std::rc::Rc;
use verification::verifier::instructions::branches::get_method_descriptor;
use std::sync::Arc;
use rust_jvm_common::loading::LoaderArc;
use crate::interpreter_util::check_inited_class;
use runtime_common::java_values::{JavaValue, Object, ArrayObject};
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::StackEntry;
use utils::lookup_method_parsed;
use descriptor_parser::MethodDescriptor;
use rust_jvm_common::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::view::ClassView;

pub mod special;
pub mod native;
pub mod interface;
pub mod virtual_;
pub mod static_;

pub mod dynamic {
    use runtime_common::{InterpreterState, StackEntry};
    use std::rc::Rc;
    use rust_jvm_common::view::constant_info_view::ConstantInfoView;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;

    pub fn invoke_dynamic(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
        let method_handle = check_inited_class(
            state,
            &ClassName::Str("java/lang/invoke/MethodHandle".to_string()),
            current_frame.clone().into(),
            current_frame.class_pointer.loader.clone(),
        );
        let method_type = check_inited_class(
            state,
            &ClassName::Str("java/lang/invoke/MethodType".to_string()),
            current_frame.clone().into(),
            current_frame.class_pointer.loader.clone(),
        );
        let invoke_dynamic_view = match current_frame.class_pointer.class_view.constant_pool_view(cp as usize) {
            ConstantInfoView::InvokeDynamic(id) => id,
            _ => panic!(),
        };

        //A call site specifier gives a symbolic reference to a method handle which is to serve as
        // the bootstrap method for a dynamic call site (§4.7.23).The method handle is resolved to
        // obtain a reference to an instance of java.lang.invoke.MethodHandle (§5.4.3.5)
        let bootstrap_method = invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref();
        invoke_dynamic_view.bootstrap_method_attr().bootstrap_args();
        let bootstrap_method_class = check_inited_class(state, &bootstrap_method.class(), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
        dbg!(invoke_dynamic_view.name_and_type().name());
        dbg!(invoke_dynamic_view.name_and_type().desc());
        dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().name_and_type());
        dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().class());

//        invoke_dynamic_view.


        dbg!(&current_frame.class_pointer.classfile.constant_pool[cp as usize]);
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