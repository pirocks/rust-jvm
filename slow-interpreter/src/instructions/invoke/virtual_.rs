use std::ops::Deref;
use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{JVM_REF_invokeSpecial, JVM_REF_invokeStatic, JVM_REF_invokeVirtual};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use rust_jvm_common::ptype::PType;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::invoke::native::mhn_temp::{REFERENCE_KIND_MASK, REFERENCE_KIND_SHIFT};
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::resolved_class;
use crate::interpreter::{run_function, WasException};
use crate::java::lang::invoke::lambda_form::LambdaForm;
use crate::java::lang::member_name::MemberName;
use crate::java_values::{JavaValue, Object};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::misc::get_all_methods;
use crate::utils::run_static_or_virtual;

/**
Should only be used for an actual invoke_virtual instruction.
Otherwise we have a better method for invoke_virtual w/ resolution
*/
pub fn invoke_virtual_instruction(state: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let (_resolved_class, method_name, expected_descriptor) = match match resolved_class(state, int_state, cp) {
        Ok(res) => res,
        Err(WasException {}) => return
    } {
        None => return,
        Some(o) => { o }
    };
    //let the main instruction check intresstate inste
    let _ = invoke_virtual(state, int_state, &method_name, &expected_descriptor);
}

pub fn invoke_virtual_method_i(state: &JVMState, int_state: &mut InterpreterStateGuard, expected_descriptor: MethodDescriptor, target_class: Arc<RuntimeClass>, target_method: &MethodView) -> Result<(), WasException> {
    invoke_virtual_method_i_impl(state, int_state, expected_descriptor, target_class, target_method)
}

fn invoke_virtual_method_i_impl(
    jvm: &JVMState,
    interpreter_state: &mut InterpreterStateGuard,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method: &MethodView,
) -> Result<(), WasException> {
    let target_method_i = target_method.method_i();
    if target_method.is_signature_polymorphic() {
        let current_frame = interpreter_state.current_frame_mut();

        let op_stack = current_frame.operand_stack();
        let method_handle = op_stack[op_stack.len() - (expected_descriptor.parameter_types.len() + 1)].cast_method_handle();
        let form: LambdaForm = method_handle.get_form();
        let vmentry: MemberName = form.get_vmentry();
        if target_method.name() == "invoke" || target_method.name() == "invokeBasic" || target_method.name() == "invokeExact" {
            //todo do conversion.
            //todo handle void return
            assert_ne!(expected_descriptor.return_type, PType::VoidType);
            let res = call_vmentry(jvm, interpreter_state, vmentry)?;
            interpreter_state.push_current_operand_stack(res);
        } else {
            unimplemented!()
        }
        return Ok(());
    }
    if target_method.is_native() {
        run_native_method(jvm, interpreter_state, target_class, target_method_i)
    } else if !target_method.is_abstract() {
        let mut args = vec![];
        let max_locals = target_method.code_attribute().unwrap().max_locals;
        setup_virtual_args(interpreter_state, &expected_descriptor, &mut args, max_locals);
        let next_entry = StackEntry::new_java_frame(jvm, target_class, target_method_i as u16, args);
        let frame_for_function = interpreter_state.push_frame(next_entry);
        match run_function(jvm, interpreter_state) {
            Ok(()) => {
                assert!(!interpreter_state.throw().is_some());
                interpreter_state.pop_frame(jvm, frame_for_function, false);
                let function_return = interpreter_state.function_return_mut();
                if *function_return {
                    *function_return = false;
                    return Ok(());
                }
                panic!()
            }
            Err(WasException {}) => {
                assert!(interpreter_state.throw().is_some());
                interpreter_state.pop_frame(jvm, frame_for_function, true);
                return Err(WasException {});
            }
        }
    } else {
        dbg!(target_method.is_abstract());
        panic!()
    }
}

pub fn call_vmentry(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard, vmentry: MemberName) -> Result<JavaValue, WasException> {
    assert_eq!(vmentry.clone().java_value().to_type(), ClassName::member_name().into());
    let flags = vmentry.get_flags() as u32;
    let ref_kind = ((flags >> REFERENCE_KIND_SHIFT) & REFERENCE_KIND_MASK) as u32;
    let invoke_static = ref_kind == JVM_REF_invokeStatic;
    let invoke_virtual = ref_kind == JVM_REF_invokeVirtual;
    let invoke_special = ref_kind == JVM_REF_invokeSpecial;
    assert!(invoke_static || invoke_virtual || invoke_special);
    //todo assert descriptors match and no conversions needed, or handle conversions as needed.
    //possibly use invokeWithArguments for conversions
    if invoke_virtual {
        unimplemented!()
    } else if invoke_static {
        let by_address = ByAddress(vmentry.clone().object());
        let method_id = *jvm.resolved_method_handles.read().unwrap().get(&by_address).unwrap();
        let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let class_view = class.view();
        let res_method = class_view.method_view_i(method_i as usize);
        run_static_or_virtual(jvm, interpreter_state, &class, res_method.name(), res_method.desc_str())?;
        assert!(interpreter_state.throw().is_none());
        let res = interpreter_state.pop_current_operand_stack();
        Ok(res)
    } else {
        unimplemented!()
    }
}

pub fn setup_virtual_args(int_state: &mut InterpreterStateGuard, expected_descriptor: &MethodDescriptor, args: &mut Vec<JavaValue>, max_locals: u16) {
    let current_frame = int_state.current_frame_mut();
    // dbg!(current_frame.operand_stack_types());
    for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    let mut i = 1;
    for _ in &expected_descriptor.parameter_types {
        let value = current_frame.pop();
        // dbg!(ptype);
        match value.clone() {
            JavaValue::Long(_) | JavaValue::Double(_) => {
                args[i] = JavaValue::Top;
                args[i + 1] = value;
                i += 2
            }
            _ => {
                args[i] = value;
                i += 1
            }
        };
    }
    if !expected_descriptor.parameter_types.is_empty() {
        args[1..i].reverse();
    }
    args[0] = current_frame.pop();
}


/*
args should be on the stack
*/
pub fn invoke_virtual(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_name: &str, md: &MethodDescriptor) -> Result<(), WasException> {
    //The resolved method must not be an instance initialization method,or the class or interface initialization method (§2.9)
    if method_name == "<init>" ||
        method_name == "<clinit>" {
        panic!()//should have been caught by verifier, though perhaps it is possible to reach this w/ invokedynamic todo
    }
    //todo implement locking on synchronized methods

//If the resolved method is not signature polymorphic ( §2.9), then the invokevirtual instruction proceeds as follows.
//we assume that it isn't signature polymorphic for now todo

//Let C be the class of objectref.
    let this_pointer = {
        let operand_stack = &int_state.current_frame().operand_stack();
        // int_state.print_stack_trace();
        // dbg!(&operand_stack);
        &operand_stack[operand_stack.len() - md.parameter_types.len() - 1].clone()
    };
    let c = match match this_pointer.unwrap_object() {
        Some(x) => x,
        None => {
            int_state.debug_print_stack_trace();
            let method_i = int_state.current_frame().method_i();
            let class_view = int_state.current_frame().class_pointer().view();
            let method_view = class_view.method_view_i(method_i as usize);
            dbg!(&method_view.code_attribute().unwrap().code);
            dbg!(&int_state.current_frame().operand_stack_types());
            dbg!(&int_state.current_frame().local_vars_types());
            dbg!(&int_state.current_frame().pc());
            dbg!(method_view.name());
            dbg!(method_view.desc_str());
            dbg!(method_view.classview().name());
            dbg!(method_name);
            int_state.debug_print_stack_trace();
            panic!()
        }
    }.deref() {
        Object::Array(_a) => {
//todo so spec seems vague about this, but basically assume this is an Object
            let object_class = assert_inited_or_initing_class(
                jvm,
                int_state,
                ClassName::object().into(),
            );
            object_class
        }
        Object::Object(o) => {
            o.class_pointer.clone()
        }
    };

    let (final_target_class, new_i) = virtual_method_lookup(jvm, int_state, &method_name, md, c)?;
    let final_class_view = &final_target_class.view();
    let target_method = &final_class_view.method_view_i(new_i);
    invoke_virtual_method_i(jvm, int_state, md.clone(), final_target_class.clone(), target_method)
}

pub fn virtual_method_lookup(
    state: &JVMState,
    int_state: &mut InterpreterStateGuard,
    method_name: &str,
    md: &MethodDescriptor,
    c: Arc<RuntimeClass>,
) -> Result<(Arc<RuntimeClass>, usize), WasException> {
    let all_methods = get_all_methods(state, int_state, c.clone(), false)?;
    let (final_target_class, new_i) = all_methods.iter().find(|(c, i)| {
        let final_target_class_view = c.view();
        let method_view = final_target_class_view.method_view_i(*i);
        let cur_name = method_view.name();
        let cur_desc = method_view.desc();
        let expected_name = &method_name;
        &cur_name == expected_name &&
            // !method_view.is_static() &&
            // !method_view.is_abstract() &&
            if method_view.is_signature_polymorphic() {
                // let _matches = method_view.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
                //     method_view.desc().return_type == PTypeView::object().to_ptype() &&
                //     md.parameter_types.last()
                //         .and_then(|x| PTypeView::from_ptype(x).try_unwrap_ref_type().cloned())
                //         .map(|x| x.unwrap_name() == ClassName::member_name())
                //         .unwrap_or(false) && unimplemented!();//todo this is currently under construction.

                true
            } else {
                md.parameter_types == cur_desc.parameter_types //we don't check return types b/c these could be subclassed
            }
    }).unwrap_or_else(|| {
        dbg!(method_name);
        dbg!(md);
        dbg!(c.view().name());
        int_state.debug_print_stack_trace();
        dbg!(int_state.current_frame().operand_stack_types());
        dbg!(int_state.current_frame().local_vars_types());
        dbg!(int_state.previous_frame().operand_stack_types());
        dbg!(int_state.previous_frame().local_vars_types());
        let call_stack = &int_state.int_state.as_ref().unwrap().call_stack;
        let prev_prev_frame = &call_stack[call_stack.len() - 3];
        dbg!(prev_prev_frame.operand_stack_types());
        dbg!(prev_prev_frame.local_vars_types());
        panic!()
    });
    Ok((final_target_class.clone(), *new_i))
}