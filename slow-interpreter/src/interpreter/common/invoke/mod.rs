use std::sync::Arc;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;

use crate::JVMState;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::utils::lookup_method_parsed;

pub mod interface;
pub mod native;
pub mod special;
pub mod static_;
pub mod virtual_;
pub mod dynamic;

pub fn find_target_method<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaInterpreterFrame<'gc, 'l>, expected_method_name: MethodName, parsed_descriptor: &CMethodDescriptor, target_class: Arc<RuntimeClass<'gc>>) -> (u16, Arc<RuntimeClass<'gc>>) {
    lookup_method_parsed(jvm, target_class, expected_method_name, parsed_descriptor).unwrap()
}
