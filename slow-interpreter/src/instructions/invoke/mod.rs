use std::sync::Arc;

use classfile_view::loading::LoaderArc;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::check_inited_class;
use crate::java_values::{ArrayObject, JavaValue, Object};
use crate::runtime_class::RuntimeClass;
use crate::utils::lookup_method_parsed;

pub mod special;
pub mod native;
pub mod interface;
pub mod virtual_;
pub mod static_;

pub mod dynamic {
    use classfile_view::view::attribute_view::BootstrapArgView;
    use classfile_view::view::constant_info_view::{ConstantInfoView, InvokeStatic, ReferenceData};
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::interpreter_util::check_inited_class;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_handle::Lookup;
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::string::JString;

    pub fn invoke_dynamic(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
        let _method_handle_class = check_inited_class(
            jvm,
            int_state,
            &ClassName::method_handle().into(),
            int_state.current_loader(jvm).clone(),
        );
        let _method_type_class = check_inited_class(
            jvm,
            int_state,
            &ClassName::method_type().into(),
            int_state.current_loader(jvm).clone(),
        );
        let _call_site_class = check_inited_class(
            jvm,
            int_state,
            &ClassName::Str("java/lang/invoke/CallSite".to_string()).into(),
            int_state.current_loader(jvm).clone(),
        );
        let class_pointer_view = int_state.current_class_view().clone();
        let invoke_dynamic_view = match class_pointer_view.constant_pool_view(cp as usize) {
            ConstantInfoView::InvokeDynamic(id) => id,
            _ => panic!(),
        };

        let bootstrap_method_view = invoke_dynamic_view.bootstrap_method();
        let method_ref = bootstrap_method_view.bootstrap_method_ref();
        let method_handle = {
            let methodref_view = method_ref.clone();
            match methodref_view.get_reference_data() {
                ReferenceData::InvokeStatic(is) => {
                    match is {
                        InvokeStatic::Interface(_) => unimplemented!(),
                        InvokeStatic::Method(mr) => {
                            // let lookup = MethodHandle::lookup(jvm, int_state);//todo use public
                            let lookup = Lookup::trusted_lookup(jvm, int_state);
                            let name = JString::from_rust(jvm, int_state, mr.name_and_type().name());
                            let desc = JString::from_rust(jvm, int_state, mr.name_and_type().desc_str());
                            let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc, None);
                            let target_class = JClass::from_name(jvm, int_state, mr.class());
                            lookup.find_static(jvm, int_state, target_class, name, method_type)
                        }
                    }
                }
            }
        };
        let arg_iterator = bootstrap_method_view.bootstrap_args();
        let args = arg_iterator.map(|x| {
            match x {
                BootstrapArgView::String(_) => unimplemented!(),
                BootstrapArgView::Class(_) => unimplemented!(),
                BootstrapArgView::Integer(_) => unimplemented!(),
                BootstrapArgView::Long(_) => unimplemented!(),
                BootstrapArgView::Float(_) => unimplemented!(),
                BootstrapArgView::Double(_) => unimplemented!(),
                BootstrapArgView::MethodHandle(mh) => {
                    let reference_data = mh.get_reference_data();
                    match reference_data {
                        ReferenceData::InvokeStatic(is) => {
                            match is {
                                InvokeStatic::Interface(i) => {
                                    dbg!(i.class());
                                    dbg!(i.name_and_type());
                                }
                                InvokeStatic::Method(mt) => {
                                    dbg!(mt.class());
                                    dbg!(mt.name_and_type().name());
                                    dbg!(mt.name_and_type().desc_str());
                                }
                            }
                        }
                    }
                    unimplemented!()
                }
                BootstrapArgView::MethodType(mt) => {
                    let desc_str = JString::from_rust(jvm, int_state, mt.get_descriptor());
                    let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc_str, None);
                    method_type.java_value()
                }
            };
        }).collect::<Vec<_>>();


        //A call site specifier gives a symbolic reference to a method handle which is to serve as
        // the bootstrap method for a dynamic call site (§4.7.23).The method handle is resolved to
        // obtain a reference to an instance of java.lang.invoke.MethodHandle (§5.4.3.5)
        let name_and_type = invoke_dynamic_view.name_and_type();
        let name = name_and_type.name();
        let desc_str = name_and_type.desc_str();
        let ref_data = method_ref.get_reference_data();
        match ref_data {
            ReferenceData::InvokeStatic(is) => {
                match is {
                    InvokeStatic::Interface(_) => unimplemented!(),
                    InvokeStatic::Method(m) => {
                        let name = m.name_and_type().name();
                        let class = m.name_and_type().desc_str();
                        dbg!(name);
                        dbg!(class);
                    }
                }
            }
        }
        dbg!(name);
        dbg!(desc_str);
        // let bootstrap_method = invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref();
        // invoke_dynamic_view.bootstrap_method_attr().bootstrap_args();
        // let _bootstrap_method_class = check_inited_class(state, &bootstrap_method.class(), current_ current_int_state.current_loader(jvm).clone());
        // dbg!(invoke_dynamic_view.name_and_type().name());
        // dbg!(invoke_dynamic_view.name_and_type().desc());
        // dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().name_and_type());
        // dbg!(invoke_dynamic_view.bootstrap_method_attr().bootstrap_method_ref().class());

//        invoke_dynamic_view.


        // dbg!(&current_frame.class_pointer.classfile.constant_pool[cp as usize]);
        unimplemented!()
    }
}

fn resolved_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) -> Option<(Arc<RuntimeClass>, String, MethodDescriptor)> {
    let view = int_state.current_class_view();
    let loader_arc = &int_state.current_loader(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, view);
    let class_name_ = match class_name_type {
        PTypeView::Ref(r) => match r {
            ReferenceTypeView::Class(c) => c,
            ReferenceTypeView::Array(_a) => if expected_method_name == *"clone" {
                //todo replace with proper native impl
                let temp = int_state.pop_current_operand_stack().unwrap_object().unwrap();
                let ArrayObject { elems, elem_type, monitor: _monitor } = temp.unwrap_array();
                let array_object = ArrayObject::new_array(
                    jvm,
                    int_state,
                    elems.borrow().clone(),
                    elem_type.clone(),
                    jvm.thread_state.new_monitor("monitor for cloned object".to_string()),
                );
                int_state.push_current_operand_stack(JavaValue::Object(Some(Arc::new(Object::Array(array_object)))));
                return None;
            } else {
                unimplemented!();
            },
        },
        _ => panic!()
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let resolved_class = check_inited_class(
        jvm,
        int_state,
        &class_name_.into(),
        loader_arc.clone(),
    );
    (resolved_class, expected_method_name, expected_descriptor).into()
}

pub fn find_target_method(
    state: &JVMState,
    loader_arc: LoaderArc,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>,
) -> (usize, Arc<RuntimeClass>) {
    lookup_method_parsed(state, target_class, expected_method_name, parsed_descriptor, &loader_arc).unwrap()
}