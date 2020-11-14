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
    use classfile_view::view::constant_info_view::{ConstantInfoView, InvokeStatic, MethodHandleView, ReferenceData};
    use descriptor_parser::parse_method_descriptor;
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
    use crate::interpreter_util::check_inited_class;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_handle::{Lookup, MethodHandle};
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::string::JString;
    use crate::java_values::JavaValue;

    pub fn invoke_dynamic(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
        let method_handle_class = check_inited_class(
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

        let other_name = invoke_dynamic_view.name_and_type().name();//todo get better names
        dbg!(&other_name);
        let other_desc_str = invoke_dynamic_view.name_and_type().desc_str();
        dbg!(&other_desc_str);


        let bootstrap_method_view = invoke_dynamic_view.bootstrap_method();
        let method_ref = bootstrap_method_view.bootstrap_method_ref();
        let bootstrap_method_handle = method_handle_from_method_view(jvm, int_state, &method_ref);
        let arg_iterator = bootstrap_method_view.bootstrap_args();
        let args = arg_iterator.map(|x| {
            match x {
                BootstrapArgView::String(s) => JString::from_rust(jvm, int_state, s.string()).java_value(),
                BootstrapArgView::Class(c) => JClass::from_name(jvm, int_state, c.name()).java_value(),
                BootstrapArgView::Integer(i) => JavaValue::Int(i.int),
                BootstrapArgView::Long(_) => unimplemented!(),
                BootstrapArgView::Float(_) => unimplemented!(),
                BootstrapArgView::Double(_) => unimplemented!(),
                BootstrapArgView::MethodHandle(mh) => method_handle_from_method_view(jvm, int_state, &mh).java_value(),
                BootstrapArgView::MethodType(mt) => desc_from_rust_str(jvm, int_state, mt.get_descriptor())
            }
        }).collect::<Vec<JavaValue>>();
        dbg!(args.iter().map(|j| j.to_type()).collect::<Vec<_>>());


        //A call site specifier gives a symbolic reference to a method handle which is to serve as
        // the bootstrap method for a dynamic call site (§4.7.23).The method handle is resolved to
        // obtain a reference to an instance of java.lang.invoke.MethodHandle (§5.4.3.5)
        let name_and_type = invoke_dynamic_view.name_and_type();
        let name = name_and_type.name();
        let ref_data = method_ref.get_reference_data();
        let desc_str = match ref_data {
            ReferenceData::InvokeStatic(is) => {
                match is {
                    InvokeStatic::Interface(_) => unimplemented!(),
                    InvokeStatic::Method(m) => {
                        m.name_and_type().desc_str()
                    }
                }
            }
        };

        //todo this trusted lookup is wrong. should use whatever the current class is for determining caller class
        let lookup_for_this = Lookup::trusted_lookup(jvm, int_state);
        let method_type = desc_from_rust_str(jvm, int_state, other_desc_str.clone());
        let name_jstring = JString::from_rust(jvm, int_state, other_name.clone()).java_value();

        dbg!(bootstrap_method_handle.clone().java_value().to_type());
        int_state.push_current_operand_stack(bootstrap_method_handle.java_value());
        int_state.push_current_operand_stack(lookup_for_this.java_value());
        int_state.push_current_operand_stack(name_jstring);
        int_state.push_current_operand_stack(method_type);
        for arg in args {
            int_state.push_current_operand_stack(arg);//todo check order is correct
        }
        dbg!(&name);
        dbg!(&desc_str);
        let method_handle_clone = method_handle_class.clone();
        let lookup_res = method_handle_clone.view().lookup_method_name("invoke");
        assert_eq!(lookup_res.len(), 1);
        let invoke = lookup_res.iter().next().unwrap();
        dbg!(int_state.current_frame().operand_stack_types());
        dbg!(invoke.desc_str());
        dbg!(invoke.name());
        invoke_virtual_method_i(jvm, int_state, parse_method_descriptor(&desc_str).unwrap(), method_handle_class, invoke.method_i(), invoke);
        let call_site = int_state.pop_current_operand_stack();
        dbg!(call_site);
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

    //todo this should go in MethodType or something.
    fn desc_from_rust_str(jvm: &JVMState, int_state: &mut InterpreterStateGuard, desc_str: String) -> JavaValue {
        let desc_str = JString::from_rust(jvm, int_state, desc_str);
        let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc_str, None);
        method_type.java_value()
    }

    fn method_handle_from_method_view(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_ref: &MethodHandleView) -> MethodHandle {
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