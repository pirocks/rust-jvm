use iced_x86::CC_be::be;

use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
use crate::class_loading::assert_loaded_class;
use crate::instructions::fields::{get_static, get_static_impl};
use crate::ir_to_java_layer::instruction_correctness_assertions::BeforeState::{NoValidate, TopOfOperandStackIs};
use crate::ir_to_java_layer::instruction_correctness_assertions::interpreted_impls::{fcmpg, fcmpl};
use crate::java_values::NativeJavaValue;

#[derive(Debug, Clone)]
pub enum BeforeState<'gc_life> {
    NoValidate,
    TopOfOperandStackIs{
        native_jv: NativeJavaValue<'gc_life>,
        rtype: RuntimeType
    },
}

pub struct AssertionState<'gc_life> {
    pub(crate) current_before: Vec<Option<BeforeState<'gc_life>>>,
}


impl<'gc_life> AssertionState<'gc_life> {
    pub fn handle_trace_after(&mut self, jvm: &'gc_life JVMState<'gc_life>, code: &CInstruction, int_state: &mut InterpreterStateGuard<'gc_life, '_>){
        let current_assertion_check  = self.current_before.last_mut().unwrap().take().unwrap();
        match current_assertion_check{
            NoValidate => {}
            BeforeState::TopOfOperandStackIs { native_jv, rtype } => {
                let actual_value = int_state.current_frame().operand_stack(jvm).get_from_end(0, rtype);
                unsafe {
                    assert_eq!(actual_value.as_njv().to_native().object, native_jv.object);
                }
            }
        }
    }

    pub fn handle_trace_before(&mut self, jvm: &'gc_life JVMState<'gc_life>, code: &CInstruction, int_state: &mut InterpreterStateGuard<'gc_life, '_>) {
        assert!(self.current_before.last().unwrap().is_none() || matches!(self.current_before.last().unwrap().as_ref().unwrap(), NoValidate));
        let before_state = match &code.info {
            CompressedInstructionInfo::aaload => {
                NoValidate
            }
            CompressedInstructionInfo::aastore => {
                NoValidate
            }
            CompressedInstructionInfo::aconst_null => {
                NoValidate
            }
            CompressedInstructionInfo::aload(_) => {
                NoValidate
            }
            CompressedInstructionInfo::aload_0 => {
                NoValidate
            }
            CompressedInstructionInfo::aload_1 => {
                NoValidate
            }
            CompressedInstructionInfo::aload_2 => {
                NoValidate
            }
            CompressedInstructionInfo::aload_3 => {
                NoValidate
            }
            CompressedInstructionInfo::anewarray(_) => {
                NoValidate
            }
            CompressedInstructionInfo::areturn => {
                self.current_before.pop().unwrap();
                return;
            }
            CompressedInstructionInfo::arraylength => {
                NoValidate
            }
            CompressedInstructionInfo::astore(_) => {
                NoValidate
            }
            CompressedInstructionInfo::astore_0 => {
                NoValidate
            }
            CompressedInstructionInfo::astore_1 => {
                NoValidate
            }
            CompressedInstructionInfo::astore_2 => {
                NoValidate
            }
            CompressedInstructionInfo::astore_3 => {
                NoValidate
            }
            CompressedInstructionInfo::athrow => {
                NoValidate
            }
            CompressedInstructionInfo::baload => {
                NoValidate
            }
            CompressedInstructionInfo::bastore => {
                NoValidate
            }
            CompressedInstructionInfo::bipush(_) => {
                NoValidate
            }
            CompressedInstructionInfo::caload => {
                NoValidate
            }
            CompressedInstructionInfo::castore => {
                NoValidate
            }
            CompressedInstructionInfo::checkcast(_) => {
                NoValidate
            }
            CompressedInstructionInfo::d2f => {
                NoValidate
            }
            CompressedInstructionInfo::d2i => {
                NoValidate
            }
            CompressedInstructionInfo::d2l => {
                NoValidate
            }
            CompressedInstructionInfo::dadd => {
                NoValidate
            }
            CompressedInstructionInfo::daload => {
                NoValidate
            }
            CompressedInstructionInfo::dastore => {
                NoValidate
            }
            CompressedInstructionInfo::dcmpg => {
                NoValidate
            }
            CompressedInstructionInfo::dcmpl => {
                NoValidate
            }
            CompressedInstructionInfo::dconst_0 => {
                NoValidate
            }
            CompressedInstructionInfo::dconst_1 => {
                NoValidate
            }
            CompressedInstructionInfo::ddiv => {
                NoValidate
            }
            CompressedInstructionInfo::dload(_) => {
                NoValidate
            }
            CompressedInstructionInfo::dload_0 => {
                NoValidate
            }
            CompressedInstructionInfo::dload_1 => {
                NoValidate
            }
            CompressedInstructionInfo::dload_2 => {
                NoValidate
            }
            CompressedInstructionInfo::dload_3 => {
                NoValidate
            }
            CompressedInstructionInfo::dmul => {
                NoValidate
            }
            CompressedInstructionInfo::dneg => {
                NoValidate
            }
            CompressedInstructionInfo::drem => {
                NoValidate
            }
            CompressedInstructionInfo::dreturn => {
                self.current_before.pop().unwrap();
                return;
            }
            CompressedInstructionInfo::dstore(_) => {
                NoValidate
            }
            CompressedInstructionInfo::dstore_0 => {
                NoValidate
            }
            CompressedInstructionInfo::dstore_1 => {
                NoValidate
            }
            CompressedInstructionInfo::dstore_2 => {
                NoValidate
            }
            CompressedInstructionInfo::dstore_3 => {
                NoValidate
            }
            CompressedInstructionInfo::dsub => {
                NoValidate
            }
            CompressedInstructionInfo::dup => {
                NoValidate
            }
            CompressedInstructionInfo::dup_x1 => {
                NoValidate
            }
            CompressedInstructionInfo::dup_x2 => {
                NoValidate
            }
            CompressedInstructionInfo::dup2 => {
                NoValidate
            }
            CompressedInstructionInfo::dup2_x1 => {
                NoValidate
            }
            CompressedInstructionInfo::dup2_x2 => {
                NoValidate
            }
            CompressedInstructionInfo::f2d => {
                NoValidate
            }
            CompressedInstructionInfo::f2i => {
                NoValidate
            }
            CompressedInstructionInfo::f2l => {
                NoValidate
            }
            CompressedInstructionInfo::fadd => {
                NoValidate
            }
            CompressedInstructionInfo::faload => {
                NoValidate
            }
            CompressedInstructionInfo::fastore => {
                NoValidate
            }
            CompressedInstructionInfo::fcmpg => {
                let float2 = int_state.current_frame().operand_stack(jvm).get_from_end(0,RuntimeType::FloatType).as_njv().unwrap_float_strict();
                let float1 = int_state.current_frame().operand_stack(jvm).get_from_end(1,RuntimeType::FloatType).as_njv().unwrap_float_strict();
                let expected_res = fcmpg(float2,float1);
                TopOfOperandStackIs { native_jv: NewJavaValue::Int(expected_res).to_native(), rtype: RuntimeType::IntType }
            }
            CompressedInstructionInfo::fcmpl => {
                let float2 = int_state.current_frame().operand_stack(jvm).get_from_end(0,RuntimeType::FloatType).as_njv().unwrap_float_strict();
                let float1 = int_state.current_frame().operand_stack(jvm).get_from_end(1,RuntimeType::FloatType).as_njv().unwrap_float_strict();
                let expected_res = fcmpl(float2,float1);
                TopOfOperandStackIs { native_jv: NewJavaValue::Int(expected_res).to_native(), rtype: RuntimeType::IntType }
            }
            CompressedInstructionInfo::fconst_0 => {
                NoValidate
            }
            CompressedInstructionInfo::fconst_1 => {
                NoValidate
            }
            CompressedInstructionInfo::fconst_2 => {
                NoValidate
            }
            CompressedInstructionInfo::fdiv => {
                NoValidate
            }
            CompressedInstructionInfo::fload(_) => {
                NoValidate
            }
            CompressedInstructionInfo::fload_0 => {
                NoValidate
            }
            CompressedInstructionInfo::fload_1 => {
                NoValidate
            }
            CompressedInstructionInfo::fload_2 => {
                NoValidate
            }
            CompressedInstructionInfo::fload_3 => {
                NoValidate
            }
            CompressedInstructionInfo::fmul => {
                NoValidate
            }
            CompressedInstructionInfo::fneg => {
                NoValidate
            }
            CompressedInstructionInfo::frem => {
                NoValidate
            }
            CompressedInstructionInfo::freturn => {
                self.current_before.pop().unwrap();
                return;
            }
            CompressedInstructionInfo::fstore(_) => {
                NoValidate
            }
            CompressedInstructionInfo::fstore_0 => {
                NoValidate
            }
            CompressedInstructionInfo::fstore_1 => {
                NoValidate
            }
            CompressedInstructionInfo::fstore_2 => {
                NoValidate
            }
            CompressedInstructionInfo::fstore_3 => {
                NoValidate
            }
            CompressedInstructionInfo::fsub => {
                NoValidate
            }
            CompressedInstructionInfo::getfield { .. } => {
                NoValidate
            }
            CompressedInstructionInfo::getstatic { desc, target_class, name } => {
                /*let res = get_static_impl(jvm, int_state, *target_class, *name).unwrap().unwrap();
                BeforeState::TopOfOperandStackIs { native_jv: res.as_njv().to_native(), rtype: res.as_njv().rtype(jvm) }*/
                NoValidate
            }
            CompressedInstructionInfo::goto_(_) => {
                NoValidate
            }
            CompressedInstructionInfo::goto_w(_) => {
                NoValidate
            }
            CompressedInstructionInfo::i2b => {
                NoValidate
            }
            CompressedInstructionInfo::i2c => {
                NoValidate
            }
            CompressedInstructionInfo::i2d => {
                NoValidate
            }
            CompressedInstructionInfo::i2f => {
                NoValidate
            }
            CompressedInstructionInfo::i2l => {
                NoValidate
            }
            CompressedInstructionInfo::i2s => {
                NoValidate
            }
            CompressedInstructionInfo::iadd => {
                NoValidate
            }
            CompressedInstructionInfo::iaload => {
                NoValidate
            }
            CompressedInstructionInfo::iand => {
                NoValidate
            }
            CompressedInstructionInfo::iastore => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_m1 => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_0 => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_1 => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_2 => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_3 => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_4 => {
                NoValidate
            }
            CompressedInstructionInfo::iconst_5 => {
                NoValidate
            }
            CompressedInstructionInfo::idiv => {
                NoValidate
            }
            CompressedInstructionInfo::if_acmpeq(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_acmpne(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_icmpeq(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_icmpne(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_icmplt(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_icmpge(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_icmpgt(_) => {
                NoValidate
            }
            CompressedInstructionInfo::if_icmple(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifeq(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifne(_) => {
                NoValidate
            }
            CompressedInstructionInfo::iflt(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifge(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifgt(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifle(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifnonnull(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ifnull(_) => {
                NoValidate
            }
            CompressedInstructionInfo::iinc(_) => {
                NoValidate
            }
            CompressedInstructionInfo::iload(_) => {
                NoValidate
            }
            CompressedInstructionInfo::iload_0 => {
                NoValidate
            }
            CompressedInstructionInfo::iload_1 => {
                NoValidate
            }
            CompressedInstructionInfo::iload_2 => {
                NoValidate
            }
            CompressedInstructionInfo::iload_3 => {
                NoValidate
            }
            CompressedInstructionInfo::imul => {
                NoValidate
            }
            CompressedInstructionInfo::ineg => {
                NoValidate
            }
            CompressedInstructionInfo::instanceof(_) => {
                NoValidate
            }
            CompressedInstructionInfo::invokedynamic(_) => {
                todo!()
            }
            CompressedInstructionInfo::invokeinterface { .. } => {
                *self.current_before.last_mut().unwrap() = Some(BeforeState::NoValidate);
                self.current_before.push(None);
                return;
            }
            CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                // let rc = assert_loaded_class(jvm, CPDType::Ref(classname_ref_type.clone()));
                // let view = rc.view();
                // let method_view = view.lookup_method(*method_name,descriptor).unwrap();
                // let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_view.method_i());
                *self.current_before.last_mut().unwrap() = Some(BeforeState::NoValidate);
                self.current_before.push(None);
                return;
            }
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                // let rc = assert_loaded_class(jvm, CPDType::Ref(classname_ref_type.clone()));
                // let view = rc.view();
                // let method_view = view.lookup_method(*method_name,descriptor).unwrap();
                // let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_view.method_i());
                *self.current_before.last_mut().unwrap() = Some(BeforeState::NoValidate);
                self.current_before.push(None);
                return;
            }
            CompressedInstructionInfo::invokevirtual { .. } => {
                *self.current_before.last_mut().unwrap() = Some(BeforeState::NoValidate);
                self.current_before.push(None);
                return;
            }
            CompressedInstructionInfo::ior => {
                NoValidate
            }
            CompressedInstructionInfo::irem => {
                NoValidate
            }
            CompressedInstructionInfo::ireturn => {
                self.current_before.pop().unwrap();
                return;
            }
            CompressedInstructionInfo::ishl => {
                NoValidate
            }
            CompressedInstructionInfo::ishr => {
                NoValidate
            }
            CompressedInstructionInfo::istore(_) => {
                NoValidate
            }
            CompressedInstructionInfo::istore_0 => {
                NoValidate
            }
            CompressedInstructionInfo::istore_1 => {
                NoValidate
            }
            CompressedInstructionInfo::istore_2 => {
                NoValidate
            }
            CompressedInstructionInfo::istore_3 => {
                NoValidate
            }
            CompressedInstructionInfo::isub => {
                NoValidate
            }
            CompressedInstructionInfo::iushr => {
                NoValidate
            }
            CompressedInstructionInfo::ixor => {
                NoValidate
            }
            CompressedInstructionInfo::jsr(_) => {
                NoValidate
            }
            CompressedInstructionInfo::jsr_w(_) => {
                NoValidate
            }
            CompressedInstructionInfo::l2d => {
                NoValidate
            }
            CompressedInstructionInfo::l2f => {
                NoValidate
            }
            CompressedInstructionInfo::l2i => {
                NoValidate
            }
            CompressedInstructionInfo::ladd => {
                NoValidate
            }
            CompressedInstructionInfo::laload => {
                NoValidate
            }
            CompressedInstructionInfo::land => {
                NoValidate
            }
            CompressedInstructionInfo::lastore => {
                NoValidate
            }
            CompressedInstructionInfo::lcmp => {
                NoValidate
            }
            CompressedInstructionInfo::lconst_0 => {
                NoValidate
            }
            CompressedInstructionInfo::lconst_1 => {
                NoValidate
            }
            CompressedInstructionInfo::ldc(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ldc_w(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ldc2_w(_) => {
                NoValidate
            }
            CompressedInstructionInfo::ldiv => {
                NoValidate
            }
            CompressedInstructionInfo::lload(_) => {
                NoValidate
            }
            CompressedInstructionInfo::lload_0 => {
                NoValidate
            }
            CompressedInstructionInfo::lload_1 => {
                NoValidate
            }
            CompressedInstructionInfo::lload_2 => {
                NoValidate
            }
            CompressedInstructionInfo::lload_3 => {
                NoValidate
            }
            CompressedInstructionInfo::lmul => {
                NoValidate
            }
            CompressedInstructionInfo::lneg => {
                NoValidate
            }
            CompressedInstructionInfo::lookupswitch(_) => {
                NoValidate
            }
            CompressedInstructionInfo::lor => {
                NoValidate
            }
            CompressedInstructionInfo::lrem => {
                NoValidate
            }
            CompressedInstructionInfo::lreturn => {
                self.current_before.pop().unwrap();
                return;
            }
            CompressedInstructionInfo::lshl => {
                NoValidate
            }
            CompressedInstructionInfo::lshr => {
                NoValidate
            }
            CompressedInstructionInfo::lstore(_) => {
                NoValidate
            }
            CompressedInstructionInfo::lstore_0 => {
                NoValidate
            }
            CompressedInstructionInfo::lstore_1 => {
                NoValidate
            }
            CompressedInstructionInfo::lstore_2 => {
                NoValidate
            }
            CompressedInstructionInfo::lstore_3 => {
                NoValidate
            }
            CompressedInstructionInfo::lsub => {
                NoValidate
            }
            CompressedInstructionInfo::lushr => {
                NoValidate
            }
            CompressedInstructionInfo::lxor => {
                NoValidate
            }
            CompressedInstructionInfo::monitorenter => {
                NoValidate
            }
            CompressedInstructionInfo::monitorexit => {
                NoValidate
            }
            CompressedInstructionInfo::multianewarray { .. } => {
                NoValidate
            }
            CompressedInstructionInfo::new(_) => {
                NoValidate
            }
            CompressedInstructionInfo::newarray(_) => {
                NoValidate
            }
            CompressedInstructionInfo::nop => {
                NoValidate
            }
            CompressedInstructionInfo::pop => {
                NoValidate
            }
            CompressedInstructionInfo::pop2 => {
                NoValidate
            }
            CompressedInstructionInfo::putfield { .. } => {
                NoValidate
            }
            CompressedInstructionInfo::putstatic { .. } => {
                NoValidate
            }
            CompressedInstructionInfo::ret(_) => {
                todo!()
            }
            CompressedInstructionInfo::return_ => {
                self.current_before.pop().unwrap();
                return;
            }
            CompressedInstructionInfo::saload => {
                NoValidate
            }
            CompressedInstructionInfo::sastore => {
                NoValidate
            }
            CompressedInstructionInfo::sipush(_) => {
                NoValidate
            }
            CompressedInstructionInfo::swap => {
                NoValidate
            }
            CompressedInstructionInfo::tableswitch(_) => {
                NoValidate
            }
            CompressedInstructionInfo::wide(_) => {
                NoValidate
            }
            CompressedInstructionInfo::EndOfCode => {
                todo!()
            }
        };
        *self.current_before.last_mut().unwrap() = Some(before_state);
    }

}

pub mod interpreted_impls;