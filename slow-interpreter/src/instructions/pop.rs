use classfile_view::view::ptype_view::PTypeView;
use classfile_view::vtype::VType;
use jvmti_jni_bindings::P_tmpdir;
use verification::OperandStack;
use verification::verifier::Frame;

use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::stack_entry::StackEntryMut;

pub fn pop2(jvm: &'_ JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life>) {
    let current_pc = current_frame.to_ref().pc();
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[&current_pc];
    let value1_vtype = data[0].clone();
    let value1 = current_frame.pop(jvm, PTypeView::LongType);
    match value1_vtype {
        VType::LongType | VType::DoubleType => {}
        _ => {
            if let JavaValue::Long(_) | JavaValue::Double(_) = current_frame.pop(jvm, PTypeView::IntType) {
                panic!()
            };
        }
    };
}

pub fn pop(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) { current_frame.pop(jvm, PTypeView::LongType); }
