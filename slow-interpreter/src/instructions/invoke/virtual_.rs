use std::ops::Deref;
use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{JVM_REF_invokeSpecial, JVM_REF_invokeStatic, JVM_REF_invokeVirtual};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::descriptors::ActuallyCompressedMD;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::runtime_type::RuntimeType;

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
pub fn invoke_virtual_instruction(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, method_name: MethodName, expected_descriptor: ActuallyCompressedMD) {
    //let the main instruction check intresstate inste
    let _ = invoke_virtual(jvm, int_state, method_name, expected_descriptor);
}

pub fn invoke_virtual_method_i<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, expected_descriptor: ActuallyCompressedMD, target_class: Arc<RuntimeClass<'gc_life>>, target_method: &MethodView) -> Result<(), WasException> {
    invoke_virtual_method_i_impl(jvm, int_state, expected_descriptor, target_class, target_method)
}

fn invoke_virtual_method_i_impl<'gc_life>(
    jvm: &'gc_life JVMState<'gc_life>,
    interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
    expected_descriptor: ActuallyCompressedMD,
    target_class: Arc<RuntimeClass<'gc_life>>,
    target_method: &MethodView,
) -> Result<(), WasException> {
    let target_method_i = target_method.method_i();
    if target_method.is_signature_polymorphic() {
        let current_frame = interpreter_state.current_frame();

        let op_stack = current_frame.operand_stack(jvm);
        let temp_value = op_stack.get((op_stack.len() - (expected_descriptor.arg_types.len() as u16 + 1)) as u16, CClassName::method_handle().into());
        let method_handle = temp_value.cast_method_handle();
        let form: LambdaForm = method_handle.get_form(jvm);
        let vmentry: MemberName = form.get_vmentry(jvm);
        if target_method.name() == MethodName::method_invoke() || target_method.name() == MethodName::method_invokeBasic() || target_method.name() == MethodName::method_invokeExact() {
            //todo do conversion.
            //todo handle void return
            assert_ne!(expected_descriptor.return_type, CPDType::VoidType);
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
        setup_virtual_args(interpreter_state, expected_descriptor, &mut args, max_locals);
        let next_entry = StackEntry::new_java_frame(jvm, target_class, target_method_i as u16, args);
        let frame_for_function = interpreter_state.push_frame(next_entry, jvm);
        match run_function(jvm, interpreter_state) {
            Ok(()) => {
                assert!(!interpreter_state.throw().is_some());
                interpreter_state.pop_frame(jvm, frame_for_function, false);
                if interpreter_state.function_return() {
                    interpreter_state.set_function_return(false);
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

pub fn call_vmentry(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, vmentry: MemberName<'gc_life>) -> Result<JavaValue<'gc_life>, WasException> {
    assert_eq!(vmentry.clone().java_value().to_type(), CClassName::member_name().into());
    let flags = vmentry.get_flags(jvm) as u32;
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
        let res_method = class_view.method_view_i(method_i);
        run_static_or_virtual(jvm, interpreter_state, &class, res_method.name(), res_method.desc())?;
        assert!(interpreter_state.throw().is_none());
        let res = interpreter_state.pop_current_operand_stack(Some(CClassName::object().into()));
        Ok(res)
    } else {
        unimplemented!()
    }
}

pub fn setup_virtual_args<'gc_life>(int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, expected_descriptor: &CMethodDescriptor, args: &mut Vec<JavaValue<'gc_life>>, max_locals: u16) {
    let mut current_frame = int_state.current_frame_mut();
    for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    let mut i = 1;
    for ptype in expected_descriptor.arg_types.iter().rev() {
        let value = current_frame.pop(Some(ptype.to_runtime_type().unwrap()));
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
    if !expected_descriptor.arg_types.is_empty() {
        args[1..i].reverse();
    }
    args[0] = current_frame.pop(Some(CClassName::object().into()));
}


/*
args should be on the stack
*/
pub fn invoke_virtual(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, method_name: MethodName, md: ActuallyCompressedMD) -> Result<(), WasException> {
    //The resolved method must not be an instance initialization method,or the class or interface initialization method (§2.9)
    if method_name == MethodName::constructor_init() ||
        method_name == MethodName::constructor_clinit() {
        panic!()//should have been caught by verifier, though perhaps it is possible to reach this w/ invokedynamic todo
    }
    //todo implement locking on synchronized methods

//If the resolved method is not signature polymorphic ( §2.9), then the invokevirtual instruction proceeds as follows.
//we assume that it isn't signature polymorphic for now todo

//Let C be the class of objectref.
    let md = jvm.method_descriptor_pool.lookup(md);
    let this_pointer = {
        let current_frame = int_state.current_frame();
        let operand_stack = &current_frame.operand_stack(jvm);
        &operand_stack.get((operand_stack.len() as usize - md.arg_types.len() - 1) as u16, RuntimeType::object())
    };
    let c = match match this_pointer.unwrap_object() {
        Some(x) => x,
        None => {
            let method_i = int_state.current_frame().method_i(jvm);
            let class_view = int_state.current_frame().class_pointer(jvm).view();
            let method_view = class_view.method_view_i(method_i);
            // dbg!(&method_view.code_attribute().unwrap().code);
            // dbg!(&int_state.current_frame().operand_stack_types());
            // dbg!(&int_state.current_frame().local_vars_types());
            // dbg!(&int_state.current_frame().pc());
            // dbg!(method_view.name());
            // dbg!(method_view.desc_str());
            // dbg!(method_view.classview().name());
            // dbg!(method_name);
            int_state.debug_print_stack_trace(jvm);
            panic!()
        }
    }.deref() {
        Object::Array(_a) => {
//todo so spec seems vague about this, but basically assume this is an Object
            let object_class = assert_inited_or_initing_class(
                jvm,
                CClassName::object().into(),
            );
            object_class
        }
        Object::Object(o) => {
            o.objinfo.class_pointer.clone()
        }
    };

    let (final_target_class, new_i) = virtual_method_lookup(jvm, int_state, method_name, md, c)?;
    let final_class_view = &final_target_class.view();
    let target_method = &final_class_view.method_view_i(new_i);
    invoke_virtual_method_i(jvm, int_state, md, final_target_class.clone(), target_method)
}

pub fn virtual_method_lookup(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    method_name: MethodName,
    md: &CMethodDescriptor,
    c: Arc<RuntimeClass<'gc_life>>,
) -> Result<(Arc<RuntimeClass<'gc_life>>, u16), WasException> {
    let all_methods = get_all_methods(jvm, int_state, c.clone(), false)?;
    let (final_target_class, new_i) = all_methods.iter().find(|(c, i)| {
        let final_target_class_view = c.view();
        let method_view = final_target_class_view.method_view_i(*i);
        let cur_name = method_view.name();
        let cur_desc = method_view.desc();
        &cur_name == &method_name &&
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
                md.arg_types == cur_desc.arg_types //we don't check return types b/c these could be subclassed
            }
    }).unwrap_or_else(|| {
        dbg!(method_name);
        dbg!(md);
        dbg!(c.view().name());
        int_state.debug_print_stack_trace(jvm);
        // dbg!(int_state.current_frame().operand_stack_types());
        // dbg!(int_state.current_frame().local_vars_types());
        // dbg!(int_state.previous_frame().operand_stack_types());
        // dbg!(int_state.previous_frame().local_vars_types());
        // let call_stack = &int_state.int_state.as_ref().unwrap().call_stack;
        // let prev_prev_frame = &call_stack[call_stack.len() - 3];
        // dbg!(prev_prev_frame.operand_stack_types());
        // dbg!(prev_prev_frame.local_vars_types());
        panic!()
    });
    Ok((final_target_class.clone(), *new_i))
}