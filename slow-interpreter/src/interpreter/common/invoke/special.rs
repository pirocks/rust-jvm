use std::sync::Arc;

use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::{JavaValueCommon, JVMState, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::{PostInstructionAction, run_function};
use crate::interpreter::common::invoke::find_target_method;
use crate::interpreter::common::invoke::native::run_native_method;
use crate::interpreter::common::invoke::virtual_::setup_virtual_args2;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::new_java_values::NewJavaValueHandle;
use crate::stack_entry::StackEntryPush;

pub fn invoke_special<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>,
    method_class_name: CClassName,
    method_name: MethodName,
    parsed_descriptor: &CMethodDescriptor,
) -> PostInstructionAction<'gc> {
    // dbg!(method_class_name.0.to_str(&jvm.string_pool));
    let target_class = match check_initing_or_inited_class(jvm, int_state.inner(), method_class_name.into()) {
        Ok(x) => x,
        Err(WasException { exception_obj }) => return PostInstructionAction::Exception { exception: WasException { exception_obj } },
    };
    let (target_m_i, final_target_class) = find_target_method(jvm, int_state.inner(), method_name, &parsed_descriptor, target_class);
    // dbg!(final_target_class.view().name().jvm_representation(&jvm.string_pool));
    // dbg!(final_target_class.view().method_view_i(target_m_i).name().0.to_str(&jvm.string_pool));
    let view = final_target_class.view();
    let target_method = view.method_view_i(target_m_i);
    let mut args = vec![];
    for _ in 0..target_method.local_var_slots() {//todo dupe
        args.push(NewJavaValueHandle::Top)
    }
    let mut i = 1;
    for ptype in parsed_descriptor.arg_types.iter().rev() {
        let popped = int_state.current_frame_mut().pop(ptype.to_runtime_type().unwrap()).to_new_java_handle(jvm);
        args[i] = popped;
        i += 1;
    }
    args[1..i].reverse();
    args[0] = int_state.current_frame_mut().pop(RuntimeType::Ref(RuntimeRefType::Class(CClassName::object()))).to_new_java_handle(jvm);
    match invoke_special_impl(jvm, int_state.inner(), &parsed_descriptor, target_m_i, final_target_class.clone(), args.iter().map(|njvh| njvh.as_njv()).collect_vec()) {
        Ok(Some(res)) => {
            int_state.current_frame_mut().push(res.to_interpreter_jv());
            return PostInstructionAction::Next {};
        }
        Ok(None) => {
            return PostInstructionAction::Next {};
        }
        Err(WasException { exception_obj }) => {
            PostInstructionAction::Exception { exception: WasException { exception_obj } }
        }
    }
}

pub fn invoke_special_impl<'k, 'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    parsed_descriptor: &CMethodDescriptor,
    target_m_i: u16,
    final_target_class: Arc<RuntimeClass<'gc>>,
    input_args: Vec<NewJavaValue<'gc, 'k>>,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    let final_target_view = final_target_class.view();
    let target_m = &final_target_view.method_view_i(target_m_i);
    if final_target_view.method_view_i(target_m_i).is_signature_polymorphic() {
        todo!();/*int_state.debug_print_stack_trace(jvm);*/
        dbg!(target_m.name());
        unimplemented!()
    } else if target_m.is_native() {
        run_native_method(jvm, int_state, final_target_class, target_m_i, input_args)
    } else {
        let mut args = vec![];
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args2(&parsed_descriptor, &mut args, max_locals, input_args);
        assert!(args[0].unwrap_object().is_some());
        let method_id = jvm.method_table.write().unwrap().get_method_id(final_target_class.clone(), target_m_i);
        // jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: int_state.current_loader(jvm) }, method_id);
        let next_entry = StackEntryPush::new_java_frame(jvm, final_target_class.clone(), target_m_i as u16, args);
        int_state.push_frame_java(next_entry, |java_frame| {
            match run_function(jvm, java_frame) {
                Ok(res) => {
                    Ok(res)
                }
                Err(WasException { exception_obj }) => {
                    Err(WasException { exception_obj })
                }
            }
        })
    }
}