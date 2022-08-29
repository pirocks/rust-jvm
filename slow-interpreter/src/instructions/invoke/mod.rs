use std::sync::Arc;

use itertools::Itertools;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_initing_or_inited_class;
use another_jit_vm_ir::WasException;
use crate::java_values::ArrayObject;
use crate::utils::{lookup_method_parsed, throw_npe_res};

pub mod interface;
pub mod native;
pub mod special;
pub mod static_;
pub mod virtual_;
pub mod dynamic;

fn resolved_class<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, cp: u16) -> Result<Option<(Arc<RuntimeClass<'gc>>, MethodName, CMethodDescriptor)>, WasException> {
    let view = int_state.current_class_view(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(&jvm.string_pool, cp as usize, &*view);
    let class_name_ = match class_name_type {
        CPDType::Class(c) => c,
        CPDType::Array { .. } => {
            if expected_method_name == MethodName::method_clone() {
                //todo replace with proper native impl
                let temp = match int_state.pop_current_operand_stack(Some(CClassName::object().into())).unwrap_object() {
                    Some(x) => x,
                    None => {
                        throw_npe_res(jvm, int_state)?;
                        unreachable!()
                    }
                };
                let ArrayObject { elem_type, .. } = temp.unwrap_array();
                let array_object = ArrayObject::new_array(jvm, int_state, temp.unwrap_array().array_iterator(jvm).collect_vec(), elem_type.clone(), jvm.thread_state.new_monitor("monitor for cloned object".to_string()))?;
                int_state.push_current_operand_stack(todo!()/*JavaValue::Object(Some(jvm.allocate_object(todo!()/*Object::Array(array_object)*/)))*/);
                return Ok(None);
            } else {
                unimplemented!();
            }
        }
        _ => panic!(),
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let resolved_class = check_initing_or_inited_class(jvm, /*int_state*/todo!(), class_name_.into())?;
    Ok((resolved_class, expected_method_name, expected_descriptor).into())
}

pub fn find_target_method<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, expected_method_name: MethodName, parsed_descriptor: &CMethodDescriptor, target_class: Arc<RuntimeClass<'gc>>) -> (u16, Arc<RuntimeClass<'gc>>) {
    lookup_method_parsed(jvm, target_class, expected_method_name, parsed_descriptor).unwrap()
}
