use rust_jvm_common::compressed_classfile::CPDType;

use crate::{JVMState, NewJavaValue, PushableFrame, WasException};
use crate::class_loading::check_resolved_class;
use crate::java_values::JavaValue;
use crate::new_java_values::NewJavaValueHandle;

pub fn a_new_array_from_name<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, len: i32, t: CPDType) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    check_resolved_class(jvm, int_state, t.clone())?;
    let new_array = JavaValue::new_vec(jvm, int_state, len as usize, NewJavaValue::Null, t)?;
    Ok(NewJavaValueHandle::Object(new_array))
    /*Ok(int_state.push_current_operand_stack(JavaValue::Object(Some(new_array.unwrap().to_gc_managed()))))*/
}