use rust_jvm_common::classfile::Atype;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
use crate::class_loading::{check_initing_or_inited_class, check_resolved_class};
use crate::interpreter::WasException;
use crate::interpreter_util::new_object;
use crate::java_values::{ArrayObject, default_value, JavaValue, Object};
use crate::new_java_values::NewJavaValueHandle;


pub fn a_new_array_from_name<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, len: i32, t: CPDType) -> Result<NewJavaValueHandle<'gc_life>, WasException> {
    check_resolved_class(jvm, int_state, t.clone())?;
    let new_array = JavaValue::new_vec(jvm, int_state, len as usize, NewJavaValue::Null, t)?;
    Ok(NewJavaValueHandle::Object(new_array))
    /*Ok(int_state.push_current_operand_stack(JavaValue::Object(Some(new_array.unwrap().to_gc_managed()))))*/
}