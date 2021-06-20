use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::P_tmpdir;

use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

//Floating-point comparison is performed in accordance with IEEE754
// this is the same as regular rust floats


pub fn fcmpl(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(jvm, JavaValue::Int(-1));
        return;
    }
    fcmp_common(jvm, current_frame, value2, value1)
}

pub fn fcmpg(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(jvm, JavaValue::Int(1));
        return;
    }
    fcmp_common(jvm, current_frame, value2, value1)
}

fn fcmp_common(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, value2: f32, value1: f32) {
    if value1.to_bits() == value2.to_bits() {
        current_frame.push(jvm, JavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.push(jvm, JavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.push(jvm, JavaValue::Int(-1))
    } else { panic!() }
}

