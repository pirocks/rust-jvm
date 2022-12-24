use itertools::Itertools;
use wtf8::Wtf8Buf;

use classfile_view::view::attribute_view::BootstrapArgView;
use classfile_view::view::ClassView;
use classfile_view::view::constant_info_view::{ConstantInfoView, InvokeSpecial, InvokeStatic, InvokeVirtual, MethodHandleView, MethodrefView, ReferenceInvokeKind};
use rust_jvm_common::StackNativeJavaValue;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::descriptor_parser::parse_method_descriptor;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValueCommon, JVMState, NewJavaValueHandle, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::common::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::java_values::native_to_new_java_value_rtype;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::invoke::call_site::CallSite;
use crate::stdlib::java::lang::invoke::lambda_form::LambdaForm;
use crate::stdlib::java::lang::invoke::method_handle::MethodHandle;
use crate::stdlib::java::lang::invoke::method_handles::lookup::Lookup;
use crate::stdlib::java::lang::invoke::method_type::MethodType;
use crate::stdlib::java::lang::member_name::MemberName;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub mod resolvers;

pub fn invoke_dynamic<'l, 'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cp: u16) -> PostInstructionAction<'gc> {
    let class_pointer_view = int_state.inner().class_pointer(jvm).view().clone();
    let invoke_dynamic_view = match class_pointer_view.constant_pool_view(cp as usize) {
        ConstantInfoView::InvokeDynamic(id) => id,
        _ => panic!(),
    };
    let method_desc_str = invoke_dynamic_view.name_and_type().desc_method(&jvm.string_pool);

    let mut args = vec![];
    let mut current_frame_mut = int_state.current_frame_mut();
    for _ in method_desc_str.arg_types {
        let raw = current_frame_mut.pop(RuntimeType::LongType).unwrap_long() as u64;
        args.push(raw);
    }
    args.reverse();

    match invoke_dynamic_impl(jvm, int_state.inner(), cp, args) {
        Ok(res) => {
            if let Some(res) = res {
                int_state.current_frame_mut().push(res.to_interpreter_jv());
            }
            PostInstructionAction::Next {}
        }
        Err(WasException { exception_obj }) => {
            exception_obj.print_stack_trace(jvm, int_state.inner()).unwrap();
            dbg!(exception_obj.to_string(jvm, int_state.inner()).unwrap().unwrap().to_rust_string(jvm));
            panic!();
            PostInstructionAction::Exception { exception: WasException { exception_obj } }
        }
    }
}

pub fn invoke_dynamic_impl<'l, 'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, cp: u16, raw_args: Vec<u64>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    let call_site = invoke_dynamic_resolve(&jvm, cp, int_state)?;

    let method_handle_class = check_initing_or_inited_class(jvm, int_state, CClassName::method_handle().into())?;
    let method_handle_view = method_handle_class.view();

    let target = call_site.get_target(jvm, int_state)?;
    let lookup_res = method_handle_view.lookup_method_name(MethodName::method_invokeExact()); //todo need safe java wrapper way of doing this
    let invoke = lookup_res.iter().next().unwrap();
    let method_type = target.type__(jvm);
    let args = method_type.get_ptypes_as_types(jvm);
    let form: LambdaForm<'gc> = target.get_form(jvm)?;
    let member_name: MemberName<'gc> = form.get_vmentry(jvm);
    let static_: bool = member_name.is_static(jvm, int_state)?;
    assert!(static_);

    let mut main_invoke_args_owned = vec![target.new_java_value_handle()];
    assert_eq!(args.len(), raw_args.len());
    for (cpd_type, raw_arg) in args.iter().zip(raw_args.into_iter()) {
        let native_jv = StackNativeJavaValue { as_u64: raw_arg };
        let expected_type = cpd_type.to_runtime_type().unwrap();
        let njv = native_to_new_java_value_rtype(native_jv, expected_type, jvm);
        main_invoke_args_owned.push(njv);
    }
    let main_invoke_args = main_invoke_args_owned.iter().map(|arg| arg.as_njv()).collect_vec();
    let desc = CMethodDescriptor { arg_types: args, return_type: CClassName::object().into() };
    let res = invoke_virtual_method_i(jvm, int_state, &desc, method_handle_class, invoke, main_invoke_args)?;

    Ok(res)
}

fn invoke_dynamic_resolve<'gc>(jvm: &'gc JVMState<'gc>, cp: u16, int_state: &mut impl PushableFrame<'gc>) -> Result<CallSite<'gc>, WasException<'gc>> {
    let method_handle_class = check_initing_or_inited_class(jvm, int_state, CClassName::method_handle().into())?;
    let _method_type_class = check_initing_or_inited_class(jvm, int_state, CClassName::method_type().into())?;
    let _call_site_class = check_initing_or_inited_class(jvm, int_state, CClassName::call_site().into())?;
    let class_pointer_view = int_state.class_pointer().unwrap().view().clone();
    let invoke_dynamic_view = match class_pointer_view.constant_pool_view(cp as usize) {
        ConstantInfoView::InvokeDynamic(id) => id,
        _ => panic!(),
    };
    let other_name = invoke_dynamic_view.name_and_type().name(&jvm.string_pool); //todo get better names
    let other_desc_str = invoke_dynamic_view.name_and_type().desc_str(&jvm.string_pool);

    let bootstrap_method_view = invoke_dynamic_view.bootstrap_method();
    let method_ref = bootstrap_method_view.bootstrap_method_ref();
    let bootstrap_method_handle = method_handle_from_method_view(jvm, int_state, &method_ref)?;
    let arg_iterator = bootstrap_method_view.bootstrap_args();
    let mut args = vec![];

    for x in arg_iterator {
        args.push(match x {
            BootstrapArgView::String(s) => JString::from_rust(jvm, int_state, s.string())?.new_java_value_handle(),
            BootstrapArgView::Class(c) => JClass::from_type(jvm, int_state, c.type_())?.new_java_value_handle(),
            BootstrapArgView::Integer(i) => NewJavaValueHandle::Int(i.int),
            BootstrapArgView::Long(long) => NewJavaValueHandle::Long(long.long),
            BootstrapArgView::Float(float) => NewJavaValueHandle::Float(float.float),
            BootstrapArgView::Double(double) => NewJavaValueHandle::Double(double.double),
            BootstrapArgView::MethodHandle(mh) => method_handle_from_method_view(jvm, int_state, &mh)?.new_java_value_handle(),
            BootstrapArgView::MethodType(mt) => desc_from_rust_str(jvm, int_state, mt.get_descriptor())?,
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
        ReferenceInvokeKind::NewInvokeSpecial(nis) => match nis {
            MethodrefView { .. } => {
                todo!()
            }
        },
        ReferenceInvokeKind::InvokeVirtual(iv) => {
            match iv {
                InvokeVirtual::Interface(_) => todo!(),
                InvokeVirtual::Method(_) => todo!(),
            }
        }
    };

    //todo this trusted lookup is wrong. should use whatever the current class is for determining caller class
    let lookup_for_this = Lookup::trusted_lookup(jvm, int_state);
    let method_type = desc_from_rust_str(jvm, int_state, other_desc_str.to_str(&jvm.string_pool).clone())?;
    let name_jstring = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(other_name.to_str(&jvm.string_pool)))?.new_java_value_handle();

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
    let call_site = invoke_virtual_method_i(jvm, int_state, &from_legacy_desc, method_handle_class.clone(), invoke, next_invoke_virtual_args)?.unwrap();
    let call_site = call_site.cast_call_site();
    Ok(call_site)
}

//todo this should go in MethodType or something.
fn desc_from_rust_str<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, desc_str: String) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let desc_str = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(desc_str))?;
    let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc_str, None)?;
    Ok(method_type.new_java_value_handle())
}

fn method_handle_from_method_view<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method_ref: &MethodHandleView) -> Result<MethodHandle<'gc>, WasException<'gc>> {
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
                    let target_class = JClass::from_type(jvm, int_state, mr.class(&jvm.string_pool).to_cpdtype())?;
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
                    let target_class = JClass::from_type(jvm, int_state, mr.class(&jvm.string_pool).to_cpdtype())?;
                    let not_sure_if_correct_at_all = int_state.class_pointer().unwrap().cpdtype();
                    let special_caller = JClass::from_type(jvm, int_state, not_sure_if_correct_at_all)?;
                    lookup.find_special(jvm, int_state, target_class, name, method_type, special_caller)?
                }
            }
        }
        ReferenceInvokeKind::NewInvokeSpecial(mr) => {
            //todo dupe
            let lookup = Lookup::trusted_lookup(jvm, int_state);
            let desc = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)))?;
            let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc, None)?;
            let target_class = JClass::from_type(jvm, int_state, mr.class(&jvm.string_pool).to_cpdtype())?;
            lookup.find_constructor(jvm, int_state, target_class, method_type)?
        }
        ReferenceInvokeKind::InvokeVirtual(iv) => {
            match iv {
                InvokeVirtual::Interface(_) => {
                    todo!()
                }
                InvokeVirtual::Method(mr) => {
                    let lookup = Lookup::trusted_lookup(jvm, int_state);
                    let desc = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)))?;
                    let method_type = MethodType::from_method_descriptor_string(jvm, int_state, desc, None)?;
                    let name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(mr.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)))?;
                    let target_class = JClass::from_type(jvm, int_state, mr.class(&jvm.string_pool).to_cpdtype())?;
                    lookup.find_virtual(jvm, int_state, target_class, name, method_type)?
                }
            }
        }
    })
}