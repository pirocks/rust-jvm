use std::collections::HashMap;

use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::vtype::VType;

pub enum ConstrainedInference {
    ForwardAssignable(VType),
    BackwardsAssignable(VType),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SimplifiedVType {
    OneWord,
    TwoWord,
    Top,
}

impl SimplifiedVType {
    pub fn is_one_word(&self) -> bool {
        match self {
            SimplifiedVType::OneWord => true,
            SimplifiedVType::TwoWord => false,
            SimplifiedVType::Top => false,
        }
    }

    pub fn is_two_word(&self) -> bool {
        match self {
            SimplifiedVType::OneWord => false,
            SimplifiedVType::TwoWord => true,
            SimplifiedVType::Top => false,
        }
    }
}

pub struct MethodFrames {
    frames: Vec<Frame>,
    offset_to_index: HashMap<ByteCodeOffset, ByteCodeIndex>,
}

impl MethodFrames {
    pub fn new(method: &MethodView) -> MethodFrames {
        let code = method.code_attribute().unwrap();
        let offset_to_index = code.instructions
            .iter()
            .sorted_by_key(|(offset,_)|**offset)
            .enumerate()
            .map(|(i, (offset, _))| (*offset, ByteCodeIndex(i as u16)))
            .collect();
        let is_static = method.is_static();
        let max_locals = code.max_locals as usize;
        let num_instructs = code.instructions.len();
        let arg_types = method.desc().arg_types.clone();
        let mut initial_local_vars = vec![Some(SimplifiedVType::Top); max_locals];
        let initial_local_var_i = if is_static {
            0
        } else {
            initial_local_vars[0] = Some(SimplifiedVType::OneWord);
            1
        };
        for (i, cpdtype) in arg_types.iter().enumerate() {
            let v_type = cpdtype_to_simplified_vtype(cpdtype);
            initial_local_vars[initial_local_var_i + i] = Some(v_type);
        }
        let start_frame = Frame {
            local_vars: initial_local_vars,
            operand_stack: Some(vec![]),
        };
        let mut all_frames = (0..num_instructs).map(|_| Frame {
            local_vars: vec![None; max_locals],
            operand_stack: None,
        }).collect_vec();
        all_frames[0] = start_frame;
        MethodFrames {
            frames: all_frames,
            offset_to_index,
        }
    }

    pub fn nth_frame_mut(&mut self, offset: ByteCodeOffset) -> &mut Frame{
        self.nth_frame_and_next_mut_offset(offset).0
    }

    pub fn nth_frame_and_next_mut_offset(&mut self, offset: ByteCodeOffset) -> (&mut Frame, Option<&mut Frame>) {
        let index = *self.offset_to_index.get(&offset).unwrap();
        self.nth_frame_and_next_mut(index)
    }

    pub fn nth_frame_and_next_mut(&mut self, n: ByteCodeIndex) -> (&mut Frame, Option<&mut Frame>) {
        let (below, above) = self.frames.split_at_mut((n.0 + 1) as usize);
        (below.last_mut().unwrap(), above.first_mut())
    }
}

pub struct InferredFrame {
    local_vars: Vec<SimplifiedVType>,
    pub operand_stack: Vec<SimplifiedVType>,
}

#[derive(Clone)]
pub struct Frame {
    local_vars: Vec<Option<SimplifiedVType>>,
    pub operand_stack: Option<Vec<Option<SimplifiedVType>>>,
}

impl Frame {
    pub fn assert_local_is(&mut self, i: u16, vtype: SimplifiedVType) {
        match self.local_vars[i as usize].as_ref() {
            Some(existing) => {
                assert_eq!(*existing, vtype)
            }
            None => {
                self.local_vars[i as usize] = Some(vtype);
            }
        };
    }

    pub fn assert_operand_stack_is(&mut self, mut operand_stack: Vec<Option<SimplifiedVType>>) {
        if let Some(current_operand_stack) = self.operand_stack.as_mut() {
            if current_operand_stack.len() != operand_stack.len() {
                panic!()
            }
            current_operand_stack.iter_mut().zip(operand_stack.iter_mut()).for_each(|(current, expected)| {
                match current {
                    None => {
                        if let Some(expected) = expected {
                            *current = Some(*expected);
                        }
                    }
                    Some(current) => {
                        if let Some(expected) = expected {
                            if current != expected {
                                panic!()
                            }
                        }
                    }
                }
            })
        } else {
            self.operand_stack = Some(operand_stack)
        }
    }

    pub fn assert_operand_stack_entry_is(&mut self, from_end: u16, vtype: SimplifiedVType) {
        let operand_stack = self.operand_stack.as_mut().unwrap();
        match operand_stack.iter().nth(from_end as usize).unwrap() {
            None => {
                *operand_stack.iter_mut().nth(from_end as usize).unwrap() = Some(vtype);
            }
            Some(existing) => {
                assert_eq!(*existing, vtype)
            }
        }
    }

    pub fn operand_stack(&self) -> Vec<Option<SimplifiedVType>> {
        self.operand_stack.as_ref().unwrap().clone()
    }

    pub fn to_inferred_frame(&self) -> InferredFrame {
        InferredFrame {
            local_vars: self.local_vars.iter().map(|local_var| local_var.unwrap()).collect(),
            operand_stack: self.operand_stack.as_ref().unwrap().iter().map(|svtype| svtype.unwrap()).collect(),
        }
    }
}

fn cpdtype_to_simplified_vtype(cpdtype: &CPDType) -> SimplifiedVType {
    match cpdtype {
        CPDType::BooleanType => SimplifiedVType::OneWord,
        CPDType::ByteType => SimplifiedVType::OneWord,
        CPDType::ShortType => SimplifiedVType::OneWord,
        CPDType::CharType => SimplifiedVType::OneWord,
        CPDType::IntType => SimplifiedVType::OneWord,
        CPDType::LongType => SimplifiedVType::TwoWord,
        CPDType::FloatType => SimplifiedVType::OneWord,
        CPDType::DoubleType => SimplifiedVType::TwoWord,
        CPDType::VoidType => {
            panic!()
        }
        CPDType::Ref(_) => SimplifiedVType::OneWord,
    }
}

pub fn type_infer(method_view: &MethodView) -> MethodFrames {
    let mut method_frames = MethodFrames::new(method_view);
    let code = method_view.code_attribute().unwrap();
    let return_type = method_view.desc().return_type;
    let instructions = code.instructions.iter().sorted_by_key(|(offset, _)| **offset)
        .map(|(offset, instr)| (*offset, instr)).collect_vec();//todo do this a lot dup
    for (i, (_offset, instruct)) in instructions.iter().enumerate() {
        dbg!(instruct);
        infer_single_instruct(&mut method_frames, return_type, instruct, i);
    }
    method_frames
}

fn infer_single_instruct(method_frames: &mut MethodFrames, return_type: CPDType, instruct: &CInstruction, i: usize) {
    let current_offset = instruct.offset;
    let current_index = ByteCodeIndex(i as u16);
    assert_eq!(*method_frames.offset_to_index.get(&current_offset).unwrap(), current_index);
    match &instruct.info {
        CompressedInstructionInfo::aaload => {
            todo!()
        }
        CompressedInstructionInfo::aastore => {
            todo!()
        }
        CompressedInstructionInfo::aconst_null => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::aload(n) => {
            one_word_variable_load(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::aload_0 => {
            one_word_variable_load(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::aload_1 => {
            one_word_variable_load(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::aload_2 => {
            one_word_variable_load(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::aload_3 => {
            one_word_variable_load(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::anewarray(_) => {
            todo!()
        }
        CompressedInstructionInfo::areturn => {
            todo!()
        }
        CompressedInstructionInfo::arraylength => {
            todo!()
        }
        CompressedInstructionInfo::astore(_) => {
            todo!()
        }
        CompressedInstructionInfo::astore_0 => {
            todo!()
        }
        CompressedInstructionInfo::astore_1 => {
            todo!()
        }
        CompressedInstructionInfo::astore_2 => {
            todo!()
        }
        CompressedInstructionInfo::astore_3 => {
            todo!()
        }
        CompressedInstructionInfo::athrow => {
            todo!()
        }
        CompressedInstructionInfo::baload => {
            todo!()
        }
        CompressedInstructionInfo::bastore => {
            todo!()
        }
        CompressedInstructionInfo::bipush(_) => {
            todo!()
        }
        CompressedInstructionInfo::caload => {
            todo!()
        }
        CompressedInstructionInfo::castore => {
            todo!()
        }
        CompressedInstructionInfo::checkcast(_) => {
            todo!()
        }
        CompressedInstructionInfo::d2f => {
            todo!()
        }
        CompressedInstructionInfo::d2i => {
            todo!()
        }
        CompressedInstructionInfo::d2l => {
            todo!()
        }
        CompressedInstructionInfo::dadd => {
            todo!()
        }
        CompressedInstructionInfo::daload => {
            todo!()
        }
        CompressedInstructionInfo::dastore => {
            todo!()
        }
        CompressedInstructionInfo::dcmpg => {
            todo!()
        }
        CompressedInstructionInfo::dcmpl => {
            todo!()
        }
        CompressedInstructionInfo::dconst_0 => {
            todo!()
        }
        CompressedInstructionInfo::dconst_1 => {
            todo!()
        }
        CompressedInstructionInfo::ddiv => {
            todo!()
        }
        CompressedInstructionInfo::dload(_) => {
            todo!()
        }
        CompressedInstructionInfo::dload_0 => {
            todo!()
        }
        CompressedInstructionInfo::dload_1 => {
            todo!()
        }
        CompressedInstructionInfo::dload_2 => {
            todo!()
        }
        CompressedInstructionInfo::dload_3 => {
            todo!()
        }
        CompressedInstructionInfo::dmul => {
            todo!()
        }
        CompressedInstructionInfo::dneg => {
            todo!()
        }
        CompressedInstructionInfo::drem => {
            todo!()
        }
        CompressedInstructionInfo::dreturn => {
            todo!()
        }
        CompressedInstructionInfo::dstore(_) => {
            todo!()
        }
        CompressedInstructionInfo::dstore_0 => {
            todo!()
        }
        CompressedInstructionInfo::dstore_1 => {
            todo!()
        }
        CompressedInstructionInfo::dstore_2 => {
            todo!()
        }
        CompressedInstructionInfo::dstore_3 => {
            todo!()
        }
        CompressedInstructionInfo::dsub => {
            todo!()
        }
        CompressedInstructionInfo::dup => {
            let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
            current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
            let mut operand_stack = current_frame.operand_stack();
            operand_stack.push(Some(SimplifiedVType::OneWord));
            operand_stack.push(Some(SimplifiedVType::OneWord));
            next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
        }
        CompressedInstructionInfo::dup_x1 => {
            todo!()
        }
        CompressedInstructionInfo::dup_x2 => {
            todo!()
        }
        CompressedInstructionInfo::dup2 => {
            todo!()
        }
        CompressedInstructionInfo::dup2_x1 => {
            todo!()
        }
        CompressedInstructionInfo::dup2_x2 => {
            todo!()
        }
        CompressedInstructionInfo::f2d => {
            todo!()
        }
        CompressedInstructionInfo::f2i => {
            todo!()
        }
        CompressedInstructionInfo::f2l => {
            todo!()
        }
        CompressedInstructionInfo::fadd => {
            todo!()
        }
        CompressedInstructionInfo::faload => {
            todo!()
        }
        CompressedInstructionInfo::fastore => {
            todo!()
        }
        CompressedInstructionInfo::fcmpg => {
            todo!()
        }
        CompressedInstructionInfo::fcmpl => {
            todo!()
        }
        CompressedInstructionInfo::fconst_0 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::fconst_1 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::fconst_2 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::fdiv => {
            todo!()
        }
        CompressedInstructionInfo::fload(n) => {
            one_word_variable_load(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::fload_0 => {
            one_word_variable_load(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::fload_1 => {
            one_word_variable_load(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::fload_2 => {
            one_word_variable_load(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::fload_3 => {
            one_word_variable_load(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::fmul => {
            todo!()
        }
        CompressedInstructionInfo::fneg => {
            todo!()
        }
        CompressedInstructionInfo::frem => {
            todo!()
        }
        CompressedInstructionInfo::freturn => {
            todo!()
        }
        CompressedInstructionInfo::fstore(_) => {
            todo!()
        }
        CompressedInstructionInfo::fstore_0 => {
            todo!()
        }
        CompressedInstructionInfo::fstore_1 => {
            todo!()
        }
        CompressedInstructionInfo::fstore_2 => {
            todo!()
        }
        CompressedInstructionInfo::fstore_3 => {
            todo!()
        }
        CompressedInstructionInfo::fsub => {
            todo!()
        }
        CompressedInstructionInfo::getfield { .. } => {
            todo!()
        }
        CompressedInstructionInfo::getstatic { .. } => {
            todo!()
        }
        CompressedInstructionInfo::goto_(_) => {
            todo!()
        }
        CompressedInstructionInfo::goto_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::i2b => {
            todo!()
        }
        CompressedInstructionInfo::i2c => {
            todo!()
        }
        CompressedInstructionInfo::i2d => {
            todo!()
        }
        CompressedInstructionInfo::i2f => {
            todo!()
        }
        CompressedInstructionInfo::i2l => {
            todo!()
        }
        CompressedInstructionInfo::i2s => {
            todo!()
        }
        CompressedInstructionInfo::iadd => {
            todo!()
        }
        CompressedInstructionInfo::iaload => {
            todo!()
        }
        CompressedInstructionInfo::iand => {
            todo!()
        }
        CompressedInstructionInfo::iastore => {
            todo!()
        }
        CompressedInstructionInfo::iconst_m1 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::iconst_0 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::iconst_1 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::iconst_2 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::iconst_3 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::iconst_4 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::iconst_5 => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::idiv => {
            todo!()
        }
        CompressedInstructionInfo::if_acmpeq(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_acmpne(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpeq(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpne(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmplt(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpge(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpgt(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmple(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifeq(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifne(_) => {
            todo!()
        }
        CompressedInstructionInfo::iflt(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifge(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifgt(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifle(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifnonnull(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifnull(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::iinc(_) => {
            todo!()
        }
        CompressedInstructionInfo::iload(n) => {
            one_word_variable_load(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::iload_0 => {
            one_word_variable_load(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::iload_1 => {
            one_word_variable_load(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::iload_2 => {
            one_word_variable_load(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::iload_3 => {
            one_word_variable_load(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::imul => {
            todo!()
        }
        CompressedInstructionInfo::ineg => {
            todo!()
        }
        CompressedInstructionInfo::instanceof(_) => {
            todo!()
        }
        CompressedInstructionInfo::invokedynamic(_) => {
            todo!()
        }
        CompressedInstructionInfo::invokeinterface { .. } => {
            todo!()
        }
        CompressedInstructionInfo::invokespecial { method_name: _, descriptor, classname_ref_type: _ } => {
            let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(ByteCodeIndex(i as u16));
            let mut operand_stack = current_frame.operand_stack();
            for arg_type in descriptor.arg_types.iter().rev() {
                match cpdtype_to_simplified_vtype(arg_type) {
                    SimplifiedVType::OneWord => {
                        let should_be_one_word = operand_stack.pop().unwrap().unwrap();
                        if let SimplifiedVType::OneWord = should_be_one_word {} else {
                            panic!()
                        }
                    }
                    SimplifiedVType::TwoWord => {
                        todo!()
                    }
                    SimplifiedVType::Top => {
                        panic!()
                    }
                }
            }
            let should_be_one_word = operand_stack.pop().unwrap().unwrap();
            if let SimplifiedVType::OneWord = should_be_one_word {} else {
                panic!()
            }
            next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
        }
        CompressedInstructionInfo::invokestatic { .. } => {
            todo!()
        }
        CompressedInstructionInfo::invokevirtual { .. } => {
            todo!()
        }
        CompressedInstructionInfo::ior => {
            todo!()
        }
        CompressedInstructionInfo::irem => {
            todo!()
        }
        CompressedInstructionInfo::ireturn => {
            todo!()
        }
        CompressedInstructionInfo::ishl => {
            todo!()
        }
        CompressedInstructionInfo::ishr => {
            todo!()
        }
        CompressedInstructionInfo::istore(_) => {
            todo!()
        }
        CompressedInstructionInfo::istore_0 => {
            todo!()
        }
        CompressedInstructionInfo::istore_1 => {
            todo!()
        }
        CompressedInstructionInfo::istore_2 => {
            todo!()
        }
        CompressedInstructionInfo::istore_3 => {
            todo!()
        }
        CompressedInstructionInfo::isub => {
            todo!()
        }
        CompressedInstructionInfo::iushr => {
            todo!()
        }
        CompressedInstructionInfo::ixor => {
            todo!()
        }
        CompressedInstructionInfo::jsr(_) => {
            todo!()
        }
        CompressedInstructionInfo::jsr_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::l2d => {
            todo!()
        }
        CompressedInstructionInfo::l2f => {
            todo!()
        }
        CompressedInstructionInfo::l2i => {
            todo!()
        }
        CompressedInstructionInfo::ladd => {
            todo!()
        }
        CompressedInstructionInfo::laload => {
            todo!()
        }
        CompressedInstructionInfo::land => {
            todo!()
        }
        CompressedInstructionInfo::lastore => {
            todo!()
        }
        CompressedInstructionInfo::lcmp => {
            todo!()
        }
        CompressedInstructionInfo::lconst_0 => {
            todo!()
        }
        CompressedInstructionInfo::lconst_1 => {
            todo!()
        }
        CompressedInstructionInfo::ldc(_) => {
            todo!()
        }
        CompressedInstructionInfo::ldc_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::ldc2_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::ldiv => {
            todo!()
        }
        CompressedInstructionInfo::lload(_) => {
            todo!()
        }
        CompressedInstructionInfo::lload_0 => {
            todo!()
        }
        CompressedInstructionInfo::lload_1 => {
            todo!()
        }
        CompressedInstructionInfo::lload_2 => {
            todo!()
        }
        CompressedInstructionInfo::lload_3 => {
            todo!()
        }
        CompressedInstructionInfo::lmul => {
            todo!()
        }
        CompressedInstructionInfo::lneg => {
            todo!()
        }
        CompressedInstructionInfo::lookupswitch(_) => {
            todo!()
        }
        CompressedInstructionInfo::lor => {
            todo!()
        }
        CompressedInstructionInfo::lrem => {
            todo!()
        }
        CompressedInstructionInfo::lreturn => {
            todo!()
        }
        CompressedInstructionInfo::lshl => {
            todo!()
        }
        CompressedInstructionInfo::lshr => {
            todo!()
        }
        CompressedInstructionInfo::lstore(_) => {
            todo!()
        }
        CompressedInstructionInfo::lstore_0 => {
            todo!()
        }
        CompressedInstructionInfo::lstore_1 => {
            todo!()
        }
        CompressedInstructionInfo::lstore_2 => {
            todo!()
        }
        CompressedInstructionInfo::lstore_3 => {
            todo!()
        }
        CompressedInstructionInfo::lsub => {
            todo!()
        }
        CompressedInstructionInfo::lushr => {
            todo!()
        }
        CompressedInstructionInfo::lxor => {
            todo!()
        }
        CompressedInstructionInfo::monitorenter => {
            todo!()
        }
        CompressedInstructionInfo::monitorexit => {
            todo!()
        }
        CompressedInstructionInfo::multianewarray { .. } => {
            todo!()
        }
        CompressedInstructionInfo::new(_) => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::newarray(_) => {
            todo!()
        }
        CompressedInstructionInfo::nop => {
            todo!()
        }
        CompressedInstructionInfo::pop => {
            todo!()
        }
        CompressedInstructionInfo::pop2 => {
            todo!()
        }
        CompressedInstructionInfo::putfield { .. } => {
            todo!()
        }
        CompressedInstructionInfo::putstatic { .. } => {
            todo!()
        }
        CompressedInstructionInfo::ret(_) => {
            todo!()
        }
        CompressedInstructionInfo::return_ => {
            if let CPDType::VoidType = return_type {} else {
                panic!()
            }
        }
        CompressedInstructionInfo::saload => {
            todo!()
        }
        CompressedInstructionInfo::sastore => {
            todo!()
        }
        CompressedInstructionInfo::sipush(_) => {
            todo!()
        }
        CompressedInstructionInfo::swap => {
            todo!()
        }
        CompressedInstructionInfo::tableswitch(_) => {
            todo!()
        }
        CompressedInstructionInfo::wide(_) => {
            todo!()
        }
        CompressedInstructionInfo::EndOfCode => {
            todo!()
        }
    }
}

fn if_one_word(method_frames: &mut MethodFrames, current_offset: ByteCodeOffset, offset: i16) {
    let res_offset = (current_offset.0 as i32 + offset as i32) as u16;
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut_offset(current_offset);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack.clone());
    let target_frame = method_frames.nth_frame_mut(ByteCodeOffset(res_offset));
    target_frame.assert_operand_stack_is(operand_stack);
}

fn one_word_const(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    let mut operand_stack = frame.operand_stack();
    operand_stack.push(None);
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
    next_frame.as_mut().unwrap().assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
}

fn one_word_variable_load(method_frames: &mut MethodFrames, n: u8, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_local_is(n as u16, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    operand_stack.push(Some(SimplifiedVType::OneWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
    next_frame.as_mut().unwrap().assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
}