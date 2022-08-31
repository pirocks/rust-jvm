use itertools::Itertools;
use wtf8::Wtf8Buf;

use another_jit_vm_ir::WasException;
use classfile_view::view::attribute_view::BootstrapArgView;
use classfile_view::view::ClassView;
use classfile_view::view::constant_info_view::{ConstantInfoView, InvokeSpecial, InvokeStatic, MethodHandleView, ReferenceInvokeKind};
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::descriptor_parser::parse_method_descriptor;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JavaValueCommon, JVMState, NewJavaValueHandle};
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::invoke::lambda_form::LambdaForm;
use crate::java::lang::invoke::method_handle::MethodHandle;
use crate::java::lang::invoke::method_handles::lookup::Lookup;
use crate::java::lang::invoke::method_type::MethodType;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::string::JString;
use crate::java::NewAsObjectOrJavaValue;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::utils::pushable_frame_todo;

pub fn invoke_dynamic<'l, 'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cp: u16, current_pc: ByteCodeOffset) -> PostInstructionAction<'gc> {
    match invoke_dynamic_impl(jvm, int_state, cp, current_pc) {
        Ok(res) => {
            PostInstructionAction::Next {}
        }
        Err(WasException {}) => {
            panic!();
            PostInstructionAction::Exception { exception: WasException {} }
        }
    }
}

fn invoke_dynamic_impl<'l, 'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cp: u16, current_pc: ByteCodeOffset) -> Result<(), WasException> {
    let mut temp: OpaqueFrame<'gc, 'l> = todo!();
    let method_handle_class = check_initing_or_inited_class(jvm, &mut temp/*int_state.inner()*/, CClassName::method_handle().into())?;
    let _method_type_class = check_initing_or_inited_class(jvm, &mut temp/*int_state.inner()*/, CClassName::method_type().into())?;
    let _call_site_class = check_initing_or_inited_class(jvm, &mut temp/*int_state.inner()*/, CClassName::call_site().into())?;
    let class_pointer_view = int_state.inner().current_class_view(jvm).clone();
    let invoke_dynamic_view = match class_pointer_view.constant_pool_view(cp as usize) {
        ConstantInfoView::InvokeDynamic(id) => id,
        _ => panic!(),
    };
    let other_name = invoke_dynamic_view.name_and_type().name(&jvm.string_pool); //todo get better names
    let other_desc_str = invoke_dynamic_view.name_and_type().desc_str(&jvm.string_pool);

    let bootstrap_method_view = invoke_dynamic_view.bootstrap_method();
    let method_ref = bootstrap_method_view.bootstrap_method_ref();
    let bootstrap_method_handle = method_handle_from_method_view(jvm, todo!()/*int_state.inner()*/, &method_ref)?;
    let arg_iterator = bootstrap_method_view.bootstrap_args();
    let mut args = vec![];

    for x in arg_iterator {
        args.push(match x {
            BootstrapArgView::String(s) => JString::from_rust(jvm, todo!()/*int_state.inner()*/, s.string())?.new_java_value_handle(),
            BootstrapArgView::Class(c) => JClass::from_type(jvm, pushable_frame_todo()/*int_state.inner()*/, c.type_())?.new_java_value_handle(),
            BootstrapArgView::Integer(i) => NewJavaValueHandle::Int(i.int),
            BootstrapArgView::Long(_) => unimplemented!(),
            BootstrapArgView::Float(_) => unimplemented!(),
            BootstrapArgView::Double(_) => unimplemented!(),
            BootstrapArgView::MethodHandle(mh) => method_handle_from_method_view(jvm, todo!()/*int_state.inner()*/, &mh)?.new_java_value_handle(),
            BootstrapArgView::MethodType(mt) => desc_from_rust_str(jvm, todo!()/*int_state.inner()*/, mt.get_descriptor())?,
        })
    }

    //A call site specifier gives a symbolic reference to a method handle which is to serve as
    // the bootstrap method for a dynamic call site (§4.7.23).The method handle is resolved to
    // obtain a reference to an instance of java.lang.invoke.MethodHandle (§5.4.3.5)
    let ref_data = method_ref.get_reference_data();
    let desc_str = match ref_data {
        ReferenceInvokeKind::InvokeStatic(is) => match is {
            InvokeStatic::Interface(_) => unimplemented!(),
            InvokeStatic::Method(m) => m.name_and_type().desc_str(&jvm.string_pool),
        },
        ReferenceInvokeKind::InvokeSpecial(is) => match is {
            InvokeSpecial::Interface(_) => todo!(),
            InvokeSpecial::Method(_) => todo!(),
        },
    };

    //todo this trusted lookup is wrong. should use whatever the current class is for determining caller class
    let lookup_for_this = Lookup::trusted_lookup(jvm, todo!()/*int_state.inner()*/);
    let method_type = desc_from_rust_str(jvm, todo!()/*int_state.inner()*/, other_desc_str.to_str(&jvm.string_pool).clone())?;
    let name_jstring = JString::from_rust(jvm, todo!()/*int_state.inner()*/, Wtf8Buf::from_string(other_name.to_str(&jvm.string_pool)))?.new_java_value_handle();

    let mut next_invoke_virtual_args = vec![];

    next_invoke_virtual_args.push(bootstrap_method_handle.new_java_value());
    next_invoke_virtual_args.push(lookup_for_this.new_java_value());
    next_invoke_virtual_args.push(name_jstring.as_njv());
    next_invoke_virtual_args.push(method_type.as_njv());
    for arg in args.iter() {
        next_invoke_virtual_args.push(arg.as_njv()); //todo check order is correct
    }
    let method_handle_clone = method_handle_class.clone();
    let method_handle_view = method_handle_clone.view();
    let lookup_res = method_handle_view.lookup_method_name(MethodName::method_invoke());
    assert_eq!(lookup_res.len(), 1);
    let invoke = lookup_res.iter().next().unwrap();
    //todo theres a MHN native for this upcall
    let from_legacy_desc = CMethodDescriptor::from_legacy(parse_method_descriptor(&desc_str.to_str(&jvm.string_pool)).unwrap(), &jvm.string_pool);
    todo!();/*int_state.inner().set_current_pc(Some(current_pc));*/
    let call_site = invoke_virtual_method_i(jvm, todo!()/*int_state.inner()*/, &from_legacy_desc, method_handle_class.clone(), invoke, next_invoke_virtual_args)?.unwrap();
    let call_site = call_site.cast_call_site();
    let target = call_site.get_target(jvm, todo!()/*int_state.inner()*/)?;
    let lookup_res = method_handle_view.lookup_method_name(MethodName::method_invokeExact()); //todo need safe java wrapper way of doing this
    let invoke = lookup_res.iter().next().unwrap();
    let (num_args, args, is_static) = if int_state.current_frame_mut().operand_stack_depth() == 0 {
        (0u16, vec![], true)
    } else {
        let method_type = target.type__(jvm);
        let args = method_type.get_ptypes_as_types(jvm);
        let form: LambdaForm<'gc> = target.get_form(jvm)?;
        let member_name: MemberName<'gc> = form.get_vmentry(jvm);
        let static_: bool = member_name.is_static(jvm, todo!()/*int_state.inner()*/)?;
        (args.len() as u16 + if static_ { 0u16 } else { 1u16 }, args, static_)
    }; //todo also sketch
    // let operand_stack_len = int_state.current_frame_mut().operand_stack(jvm).len();
    // dbg!(operand_stack_len - num_args);
    // dbg!(operand_stack_len);
    // dbg!(num_args);
    // int_state.current_frame_mut().operand_stack_mut().insert((operand_stack_len - num_args) as usize, target.java_value());
    //todo not passing final call args?
    // int_state.print_stack_trace();
    let mut main_invoke_args_owned = vec![target.new_java_value_handle()];
    if !is_static {
        let arg = int_state.current_frame_mut().pop(RuntimeType::object());
        main_invoke_args_owned.push(arg.to_new_java_handle(jvm));
    }
    for cpd_type in args.iter() {
        //todo is the order correct here
        let arg = int_state.current_frame_mut().pop(cpd_type.to_runtime_type().unwrap());
        main_invoke_args_owned.push(arg.to_new_java_handle(jvm));
    }
    main_invoke_args_owned[(1 + if is_static { 0 } else { 1 })..].reverse();
    // dbg!(main_invoke_args_owned.iter().map(|handle| handle.as_njv().rtype(jvm)).collect_vec());
    let main_invoke_args = main_invoke_args_owned.iter().map(|arg| arg.as_njv()).collect_vec();
    todo!();/*int_state.inner().set_current_pc(Some(current_pc));*/
    let desc = CMethodDescriptor { arg_types: args, return_type: CClassName::object().into() };
    let res = invoke_virtual_method_i(jvm, todo!()/*int_state.inner()*/, &desc, method_handle_class, invoke, main_invoke_args)?;

    todo!();/*assert!(int_state.inner().throw().is_none());*/

    int_state.current_frame_mut().push(res.unwrap().to_interpreter_jv());
    Ok(())
}

//todo this should go in MethodType or something.
fn desc_from_rust_str<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, desc_str: String) -> Result<NewJavaValueHandle<'gc>, WasException> {
    let desc_str = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(desc_str))?;
    let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc_str, None)?;
    Ok(method_type.new_java_value_handle())
}

fn method_handle_from_method_view<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, method_ref: &MethodHandleView) -> Result<MethodHandle<'gc>, WasException> {
    let methodref_view = method_ref.clone();
    Ok(match methodref_view.get_reference_data() {
        ReferenceInvokeKind::InvokeStatic(is) => {
            match is {
                InvokeStatic::Interface(_) => unimplemented!(),
                InvokeStatic::Method(mr) => {
                    // let lookup = MethodHandle::lookup(jvm, int_state);//todo use public
                    let lookup = Lookup::trusted_lookup(jvm, int_state);
                    let name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)))?;
                    let desc = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)))?;
                    let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc, None)?;
                    let target_class = JClass::from_type(jvm, pushable_frame_todo()/*int_state*/, mr.class(&jvm.string_pool).to_cpdtype())?;
                    lookup.find_static(jvm, int_state, target_class, name, method_type)?
                }
            }
        }
        ReferenceInvokeKind::InvokeSpecial(is) => {
            match is {
                InvokeSpecial::Interface(_) => todo!(),
                InvokeSpecial::Method(mr) => {
                    //todo dupe
                    let lookup = Lookup::trusted_lookup(jvm, int_state);
                    let name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)))?;
                    let desc = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)))?;
                    let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc, None)?;
                    let target_class = JClass::from_type(jvm, pushable_frame_todo()/*int_state*/, mr.class(&jvm.string_pool).to_cpdtype())?;
                    let not_sure_if_correct_at_all = int_state.current_frame().class_pointer(jvm).cpdtype();
                    let special_caller = JClass::from_type(jvm, pushable_frame_todo()/*int_state*/, not_sure_if_correct_at_all)?;
                    lookup.find_special(jvm, int_state, target_class, name, method_type, special_caller)?
                }
            }
        }
    })
}
