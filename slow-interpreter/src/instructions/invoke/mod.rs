use std::sync::Arc;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
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
    use classfile_view::view::constant_info_view::{ConstantInfoView, InvokeSpecial, InvokeStatic, MethodHandleView, ReferenceData};
    use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
    use rust_jvm_common::classnames::ClassName;
    use rust_jvm_common::ptype::{PType, ReferenceType};

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
    use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::ExceptionError;
    use crate::java::lang::invoke::lambda_form::LambdaForm;
    use crate::java::lang::invoke::method_handle::{Lookup, MethodHandle};
    use crate::java::lang::invoke::method_type::MethodType;
    use crate::java::lang::member_name::MemberName;
    use crate::java::lang::string::JString;
    use crate::java_values::JavaValue;

    pub fn invoke_dynamic(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
        let method_handle_class = check_initing_or_inited_class(
            jvm,
            int_state,
            ClassName::method_handle().into(),
        );
        let _method_type_class = check_initing_or_inited_class(
            jvm,
            int_state,
            ClassName::method_type().into(),
        );
        let _call_site_class = check_initing_or_inited_class(
            jvm,
            int_state,
            ClassName::Str("java/lang/invoke/CallSite".to_string()).into(),
        );
        let class_pointer_view = int_state.current_class_view().clone();
        // dbg!(cp);
        let invoke_dynamic_view = match class_pointer_view.constant_pool_view(cp as usize) {
            ConstantInfoView::InvokeDynamic(id) => id,
            _ => panic!(),
        };
        // dbg!(invoke_dynamic_view.name_and_type().name());
        // dbg!(invoke_dynamic_view.name_and_type().desc_str());
        let other_name = invoke_dynamic_view.name_and_type().name();//todo get better names
        // dbg!(&other_name);
        let other_desc_str = invoke_dynamic_view.name_and_type().desc_str();
        // let other_desc = invoke_dynamic_view.name_and_type().desc_method();
        // dbg!(&other_desc_str);


        let bootstrap_method_view = invoke_dynamic_view.bootstrap_method();
        let method_ref = bootstrap_method_view.bootstrap_method_ref();
        let bootstrap_method_handle = match method_handle_from_method_view(jvm, int_state, &method_ref) {
            Ok(res) => res,
            Err(_) => return
        };
        let arg_iterator = bootstrap_method_view.bootstrap_args();
        let mut args = vec![];

        for x in arg_iterator {
            args.push(match x {
                BootstrapArgView::String(s) => JString::from_rust(jvm, int_state, s.string()).java_value(),
                BootstrapArgView::Class(c) => JClass::from_name(jvm, int_state, c.name()).java_value(),
                BootstrapArgView::Integer(i) => JavaValue::Int(i.int),
                BootstrapArgView::Long(_) => unimplemented!(),
                BootstrapArgView::Float(_) => unimplemented!(),
                BootstrapArgView::Double(_) => unimplemented!(),
                BootstrapArgView::MethodHandle(mh) => match method_handle_from_method_view(jvm, int_state, &mh) {
                    Ok(method_handle_obj) => method_handle_obj,
                    Err(_) => return
                }.java_value(),
                BootstrapArgView::MethodType(mt) => desc_from_rust_str(jvm, int_state, mt.get_descriptor())
            })
        }

        // dbg!(args.iter().map(|j| j.to_type()).collect::<Vec<_>>());


        //A call site specifier gives a symbolic reference to a method handle which is to serve as
        // the bootstrap method for a dynamic call site (§4.7.23).The method handle is resolved to
        // obtain a reference to an instance of java.lang.invoke.MethodHandle (§5.4.3.5)
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
            ReferenceData::InvokeSpecial(is) => match is {
                InvokeSpecial::Interface(_) => todo!(),
                InvokeSpecial::Method(_) => todo!()
            }
        };

        //todo this trusted lookup is wrong. should use whatever the current class is for determining caller class
        let lookup_for_this = Lookup::trusted_lookup(jvm, int_state);
        let method_type = desc_from_rust_str(jvm, int_state, other_desc_str.clone());
        let name_jstring = JString::from_rust(jvm, int_state, other_name.clone()).java_value();

        // dbg!(bootstrap_method_handle.clone().java_value().to_type());
        int_state.push_current_operand_stack(bootstrap_method_handle.java_value());
        int_state.push_current_operand_stack(lookup_for_this.java_value());
        int_state.push_current_operand_stack(name_jstring);
        int_state.push_current_operand_stack(method_type);
        for arg in args {
            int_state.push_current_operand_stack(arg);//todo check order is correct
        }
        // dbg!(&name);
        // dbg!(&desc_str);
        let method_handle_clone = method_handle_class.clone();
        let lookup_res = method_handle_clone.view().lookup_method_name("invoke");
        assert_eq!(lookup_res.len(), 1);
        let invoke = lookup_res.iter().next().unwrap();
        // dbg!(int_state.current_frame().operand_stack_types());
        // dbg!(invoke.desc_str());
        // dbg!(invoke.name());
        //todo theres a MHN native for this upcall
        invoke_virtual_method_i(jvm, int_state, parse_method_descriptor(&desc_str).unwrap(), method_handle_class.clone(), invoke.method_i(), invoke);
        let call_site = int_state.pop_current_operand_stack().cast_call_site();
        // dbg!(call_site.to_string(jvm, int_state).to_rust_string());
        let target = call_site.get_target(jvm, int_state);
        // dbg!(target.to_string(jvm, int_state).to_rust_string());
        // dbg!(target.)
        let method_handle_clone = method_handle_class.clone();
        let lookup_res = method_handle_clone.view().lookup_method_name("invokeExact");//todo need safe java wrapper way of doing this
        let invoke = lookup_res.iter().next().unwrap();
        let (num_args, args) = if int_state.current_frame().operand_stack().is_empty() {
            (0, vec![])
        } else {
            // dbg!(target.to_string(jvm, int_state).to_rust_string());
            let method_type = target.type__();
            let args = method_type.get_ptypes_as_types(jvm);
            // dbg!(target.clone().java_value().unwrap_normal_object().class_pointer.view().name());
            let form: LambdaForm = target.get_form();
            let member_name: MemberName = form.get_vmentry();
            // dbg!(member_name.clazz());
            // dbg!(member_name.get_name_or_null().unwrap().to_rust_string());
            let static_: bool = member_name.is_static(jvm, int_state);
            (args.len() + if static_ { 0 } else { 1 }, args)
        }; //todo also sketch
        let operand_stack_len = int_state.current_frame().operand_stack().len();
        int_state.current_frame_mut().operand_stack_mut().insert(operand_stack_len - num_args, target.java_value());
        // int_state.push_current_operand_stack(target.java_value());
        // let target_method_type = call_site.get_target(jvm, int_state).internal_member_name(jvm, int_state).get_method_type(jvm, int_state);
        // let num_params = target_method_type.get_ptypes_as_types().len();
        // let operand_stack = int_state.current_frame_mut().operand_stack_mut();
        // let len = operand_stack.len();
        // assert!(target_method_type.is sratic)
        // [(len - 1 - num_params)..(len - 1)].reverse();
        // dbg!(int_state.current_frame().operand_stack_types());
        //todo not passing final call args?
        // int_state.print_stack_trace();
        // dbg!(&args);
        invoke_virtual_method_i(jvm, int_state, MethodDescriptor { parameter_types: args, return_type: PType::Ref(ReferenceType::Class(ClassName::object())) }, method_handle_class, invoke.method_i(), invoke);

        assert!(int_state.throw().is_none());

        let res = int_state.pop_current_operand_stack();
        // dbg!(&res);
        int_state.push_current_operand_stack(res);
    }

    //todo this should go in MethodType or something.
    fn desc_from_rust_str(jvm: &JVMState, int_state: &mut InterpreterStateGuard, desc_str: String) -> JavaValue {
        let desc_str = JString::from_rust(jvm, int_state, desc_str);
        let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc_str, None);
        method_type.java_value()
    }

    fn method_handle_from_method_view(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_ref: &MethodHandleView) -> Result<MethodHandle, ExceptionError> {
        let methodref_view = method_ref.clone();
        Ok(match methodref_view.get_reference_data() {
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
                        lookup.find_static(jvm, int_state, target_class, name, method_type)?
                    }
                }
            }
            ReferenceData::InvokeSpecial(is) => {
                match is {
                    InvokeSpecial::Interface(_) => todo!(),
                    InvokeSpecial::Method(mr) =>
                        {
                            //todo dupe
                            let lookup = Lookup::trusted_lookup(jvm, int_state);
                            let name = JString::from_rust(jvm, int_state, mr.name_and_type().name());
                            let desc = JString::from_rust(jvm, int_state, mr.name_and_type().desc_str());
                            let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc, None);
                            let target_class = JClass::from_name(jvm, int_state, mr.class());
                            let not_sure_if_correct_at_all = int_state.current_frame().class_pointer().view().name();
                            let special_caller = JClass::from_name(jvm, int_state, not_sure_if_correct_at_all);
                            lookup.find_special(jvm, int_state, target_class, name, method_type, special_caller)
                        }
                }
            }
        })
    }
}

fn resolved_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) -> Option<(Arc<RuntimeClass>, String, MethodDescriptor)> {
    let view = int_state.current_class_view();
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, view);
    let class_name_ = match class_name_type {
        PTypeView::Ref(r) => match r {
            ReferenceTypeView::Class(c) => c,
            ReferenceTypeView::Array(_a) => if expected_method_name == *"clone" {
                //todo replace with proper native impl
                let temp = int_state.pop_current_operand_stack().unwrap_object().unwrap();
                let ArrayObject { elems: _, elem_type, monitor: _monitor } = temp.unwrap_array();
                let array_object = ArrayObject::new_array(
                    jvm,
                    int_state,
                    temp.unwrap_array().mut_array().clone(),
                    elem_type.clone(),
                    jvm.thread_state.new_monitor("monitor for cloned object".to_string())
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
    let resolved_class = check_initing_or_inited_class(
        jvm,
        int_state,
        class_name_.into(),
    );
    (resolved_class, expected_method_name, expected_descriptor).into()
}

pub fn find_target_method(
    state: &JVMState,
    int_state: &mut InterpreterStateGuard,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>,
) -> (usize, Arc<RuntimeClass>) {
    lookup_method_parsed(state, int_state, target_class, expected_method_name, parsed_descriptor).unwrap()
}
