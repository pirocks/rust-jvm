#![feature(box_patterns)]

use std::collections::HashMap;

use itertools::{Either, Itertools};

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset};
use rust_jvm_common::classfile::{LookupSwitch, TableSwitch};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedExceptionTableElem, CompressedInstructionInfo, CompressedLdc2W, CompressedLdcW};
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

    pub fn try_not_top(&self) -> Option<SimplifiedVType> {
        match self {
            SimplifiedVType::OneWord => Some(SimplifiedVType::OneWord),
            SimplifiedVType::TwoWord => Some(SimplifiedVType::TwoWord),
            SimplifiedVType::Top => None
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
            .sorted_by_key(|(offset, _)| **offset)
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
        let mut res = MethodFrames {
            frames: all_frames,
            offset_to_index,
        };
        for exception_table_entry in code.exception_table.iter() {
            res.apply_exception_table(exception_table_entry);
        }
        res
    }

    pub fn apply_exception_table(&mut self, exception_table_elem: &CompressedExceptionTableElem) {
        let CompressedExceptionTableElem {
            start_pc: _,
            end_pc: _,
            handler_pc,
            catch_type: _
        } = exception_table_elem;
        let handler = *handler_pc;
        let handle_frame = self.nth_frame_mut(handler);
        handle_frame.assert_operand_stack_is(vec![Some(SimplifiedVType::OneWord)]);
    }

    pub fn nth_frame_mut(&mut self, offset: ByteCodeOffset) -> &mut Frame {
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

    pub fn inferred_frames(&self) -> HashMap<ByteCodeOffset, InferredFrame> {
        let mut res = HashMap::new();
        let index_to_offset = self.offset_to_index.iter().map(|(offset, index)| (*index, *offset)).collect::<HashMap<_, _>>();
        for (i, frame) in self.frames.iter().enumerate() {
            res.insert(*index_to_offset.get(&ByteCodeIndex(i as u16)).unwrap(), frame.to_inferred_frame());
        }
        res
    }
}

#[derive(Clone)]
pub struct InferredFrame {
    pub local_vars: Vec<SimplifiedVType>,
    pub operand_stack: Vec<SimplifiedVType>,
}

impl InferredFrame {
    pub fn no_tops(&self) -> InferredFrame {
        InferredFrame {
            local_vars: self.local_vars.iter().flat_map(|vtype| {
                vtype.try_not_top()
            }).collect(),
            operand_stack: self.operand_stack.iter().flat_map(|vtype| {
                vtype.try_not_top()
            }).collect(),
        }
    }
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
        match operand_stack.iter().rev().nth(from_end as usize).unwrap() {
            None => {
                *operand_stack.iter_mut().rev().nth(from_end as usize).unwrap() = Some(vtype);
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
            local_vars: self.local_vars.iter().map(|local_var| local_var.unwrap_or(SimplifiedVType::Top)).collect(),
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
        CPDType::Class(_) => SimplifiedVType::OneWord,
        CPDType::Array { .. } => SimplifiedVType::OneWord,
    }
}

pub fn type_infer(method_view: &MethodView) -> MethodFrames {
    let mut method_frames = MethodFrames::new(method_view);
    let code = method_view.code_attribute().unwrap();
    let return_type = method_view.desc().return_type;
    let instructions = code.instructions.iter().sorted_by_key(|(offset, _)| **offset)
        .map(|(offset, instr)| (*offset, instr)).collect_vec();//todo do this a lot dup
    for (i, (_offset, instruct)) in instructions.iter().enumerate() {
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
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::aastore => {
            three_one_word_in_zero_out(method_frames, current_index);
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
            one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::areturn => {
            top_operand_is_one_word_and_exit(method_frames, current_offset);
        }
        CompressedInstructionInfo::arraylength => {
            one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::astore(n) => {
            one_word_variable_store(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::astore_0 => {
            one_word_variable_store(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::astore_1 => {
            one_word_variable_store(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::astore_2 => {
            one_word_variable_store(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::astore_3 => {
            one_word_variable_store(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::athrow => {
            top_operand_is_one_word_and_exit(method_frames, current_offset);
        }
        CompressedInstructionInfo::baload => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::bastore => {
            three_one_word_in_zero_out(method_frames, current_index);
        }
        CompressedInstructionInfo::bipush(_) => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::caload => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::castore => {
            three_one_word_in_zero_out(method_frames, current_index);
        }
        CompressedInstructionInfo::checkcast(_) => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::d2f => {
            one_two_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::d2i => {
            one_two_word_in_one_word_out(method_frames, current_index)
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
            two_two_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::dcmpl => {
            two_two_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::dconst_0 => {
            two_word_const(method_frames, current_index)
        }
        CompressedInstructionInfo::dconst_1 => {
            two_word_const(method_frames, current_index)
        }
        CompressedInstructionInfo::ddiv => {
            todo!()
        }
        CompressedInstructionInfo::dload(n) => {
            two_word_variable_load(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::dload_0 => {
            two_word_variable_load(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::dload_1 => {
            two_word_variable_load(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::dload_2 => {
            two_word_variable_load(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::dload_3 => {
            two_word_variable_load(method_frames, 3, current_index);
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
        CompressedInstructionInfo::dstore(n) => {
            two_word_variable_store(method_frames, *n, current_index)
        }
        CompressedInstructionInfo::dstore_0 => {
            two_word_variable_store(method_frames, 0, current_index)
        }
        CompressedInstructionInfo::dstore_1 => {
            two_word_variable_store(method_frames, 1, current_index)
        }
        CompressedInstructionInfo::dstore_2 => {
            two_word_variable_store(method_frames, 2, current_index)
        }
        CompressedInstructionInfo::dstore_3 => {
            two_word_variable_store(method_frames, 3, current_index)
        }
        CompressedInstructionInfo::dsub => {
            todo!()
        }
        CompressedInstructionInfo::dup => {
            let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
            current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
            let mut operand_stack = current_frame.operand_stack();
            let _ = operand_stack.pop().unwrap().unwrap();
            operand_stack.push(Some(SimplifiedVType::OneWord));
            operand_stack.push(Some(SimplifiedVType::OneWord));
            next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
        }
        CompressedInstructionInfo::dup_x1 => {
            let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
            current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
            current_frame.assert_operand_stack_entry_is(1, SimplifiedVType::OneWord);
            let mut operand_stack = current_frame.operand_stack();
            operand_stack.pop().unwrap().unwrap();
            operand_stack.pop().unwrap().unwrap();
            operand_stack.push(Some(SimplifiedVType::OneWord));
            operand_stack.push(Some(SimplifiedVType::OneWord));
            operand_stack.push(Some(SimplifiedVType::OneWord));
            next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
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
            one_word_in_two_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::f2i => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::f2l => {
            todo!()
        }
        CompressedInstructionInfo::fadd => {
            todo!()
        }
        CompressedInstructionInfo::faload => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::fastore => {
            three_one_word_in_zero_out(method_frames, current_index);
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
            two_one_word_in_one_word_out(method_frames, current_index);
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
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::fneg => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::frem => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::freturn => {
            top_operand_is_one_word_and_exit(method_frames, current_offset);
        }
        CompressedInstructionInfo::fstore(n) => {
            one_word_variable_store(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::fstore_0 => {
            one_word_variable_store(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::fstore_1 => {
            one_word_variable_store(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::fstore_2 => {
            one_word_variable_store(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::fstore_3 => {
            one_word_variable_store(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::fsub => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::getfield { name: _, desc, target_class: _ } => {
            let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
            current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
            match cpdtype_to_simplified_vtype(&desc.0) {
                SimplifiedVType::OneWord => {
                    let operand_stack = current_frame.operand_stack();
                    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
                }
                SimplifiedVType::TwoWord => {
                    let mut operand_stack = current_frame.operand_stack();
                    operand_stack.pop().unwrap().unwrap();
                    operand_stack.push(Some(SimplifiedVType::TwoWord));
                    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
                }
                SimplifiedVType::Top => {
                    panic!()
                }
            }
        }
        CompressedInstructionInfo::getstatic { name: _, desc, target_class: _ } => {
            match cpdtype_to_simplified_vtype(&desc.0) {
                SimplifiedVType::OneWord => {
                    one_word_const(method_frames, current_index);
                }
                SimplifiedVType::TwoWord => {
                    todo!()
                }
                SimplifiedVType::Top => {
                    panic!()
                }
            }
        }
        CompressedInstructionInfo::goto_(offset) => {
            let res_offset = (current_offset.0 as i32 + *offset as i32) as u16;
            let current_frame = method_frames.nth_frame_mut(current_offset);
            let operand_stack = current_frame.operand_stack();
            let target_frame = method_frames.nth_frame_mut(ByteCodeOffset(res_offset));
            target_frame.assert_operand_stack_is(operand_stack);
        }
        CompressedInstructionInfo::goto_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::i2b => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::i2c => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::i2d => {
            one_word_in_two_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::i2f => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::i2l => {
            one_word_in_two_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::i2s => {
            one_word_in_one_word_out(method_frames, current_index)
        }
        CompressedInstructionInfo::iadd => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::iaload => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::iand => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::iastore => {
            three_one_word_in_zero_out(method_frames, current_index);
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
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::if_acmpeq(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_acmpne(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_icmpeq(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_icmpne(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_icmplt(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_icmpge(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_icmpgt(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::if_icmple(offset) => {
            if_two_one_word(method_frames, current_offset, *offset)
        }
        CompressedInstructionInfo::ifeq(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifne(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::iflt(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifge(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifgt(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifle(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifnonnull(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::ifnull(offset) => {
            if_one_word(method_frames, current_offset, *offset);
        }
        CompressedInstructionInfo::iinc(n) => {
            let (current_frame, next_frame) = method_frames.nth_frame_and_next_mut_offset(current_offset);
            current_frame.assert_local_is(n.index, SimplifiedVType::OneWord);
            let operand_stack = current_frame.operand_stack();
            next_frame.unwrap().assert_operand_stack_is(operand_stack);
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
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::ineg => {
            one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::instanceof(_) => {
            one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::invokedynamic(_) => {
            todo!()
        }
        CompressedInstructionInfo::invokeinterface { method_name: _, descriptor, classname_ref_type: _, count: _ } => {
            invoke(method_frames, i, descriptor, true);
        }
        CompressedInstructionInfo::invokespecial { method_name: _, descriptor, classname_ref_type: _ } => {
            invoke(method_frames, i, descriptor, true);
        }
        CompressedInstructionInfo::invokestatic { method_name: _, descriptor, classname_ref_type: _ } => {
            invoke(method_frames, i, descriptor, false);
        }
        CompressedInstructionInfo::invokevirtual { method_name: _, descriptor, classname_ref_type: _ } => {
            invoke(method_frames, i, descriptor, true);
        }
        CompressedInstructionInfo::ior => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::irem => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::ireturn => {
            top_operand_is_one_word_and_exit(method_frames, current_offset);
        }
        CompressedInstructionInfo::ishl => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::ishr => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::istore(n) => {
            one_word_variable_store(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::istore_0 => {
            one_word_variable_store(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::istore_1 => {
            one_word_variable_store(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::istore_2 => {
            one_word_variable_store(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::istore_3 => {
            one_word_variable_store(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::isub => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::iushr => {
            two_one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::ixor => {
            two_one_word_in_one_word_out(method_frames, current_index);
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
            two_two_word_in_one_word_out(method_frames,current_index)
        }
        CompressedInstructionInfo::lconst_0 => {
            two_word_const(method_frames, current_index)
        }
        CompressedInstructionInfo::lconst_1 => {
            two_word_const(method_frames, current_index)
        }
        CompressedInstructionInfo::ldc(ldc_either) => {
            match ldc_either {
                Either::Left(ldc_left) => {
                    match ldc_left {
                        CompressedLdcW::String { .. } => {
                            one_word_const(method_frames, current_index);
                        }
                        CompressedLdcW::Class { .. } => {
                            one_word_const(method_frames, current_index);
                        }
                        CompressedLdcW::Float { .. } => {
                            one_word_const(method_frames, current_index);
                        }
                        CompressedLdcW::Integer { .. } => {
                            one_word_const(method_frames, current_index);
                        }
                        CompressedLdcW::MethodType { .. } => {
                            todo!()
                        }
                        CompressedLdcW::MethodHandle { .. } => {
                            todo!()
                        }
                        CompressedLdcW::LiveObject(_) => {
                            todo!()
                        }
                    }
                }
                Either::Right(ldc_right) => {
                    match ldc_right {
                        CompressedLdc2W::Long(_) => {
                            todo!()
                        }
                        CompressedLdc2W::Double(_) => {
                            todo!()
                        }
                    }
                }
            }
        }
        CompressedInstructionInfo::ldc_w(wide) => {
            match wide {
                CompressedLdcW::String { .. } => {
                    one_word_const(method_frames, current_index);
                }
                CompressedLdcW::Class { .. } => {
                    one_word_const(method_frames, current_index);
                }
                CompressedLdcW::Float { .. } => {
                    one_word_const(method_frames, current_index);
                }
                CompressedLdcW::Integer { .. } => {
                    one_word_const(method_frames, current_index);
                }
                CompressedLdcW::MethodType { .. } => todo!(),
                CompressedLdcW::MethodHandle { .. } => todo!(),
                CompressedLdcW::LiveObject(_) => todo!(),
            }
        }
        CompressedInstructionInfo::ldc2_w(wide) => {
            match wide {
                CompressedLdc2W::Long(_) => {
                    two_word_const(method_frames, current_index);
                }
                CompressedLdc2W::Double(_) => {
                    two_word_const(method_frames, current_index);
                }
            }
        }
        CompressedInstructionInfo::ldiv => {
            todo!()
        }
        CompressedInstructionInfo::lload(n) => {
            two_word_variable_load(method_frames, *n, current_index);
        }
        CompressedInstructionInfo::lload_0 => {
            two_word_variable_load(method_frames, 0, current_index);
        }
        CompressedInstructionInfo::lload_1 => {
            two_word_variable_load(method_frames, 1, current_index);
        }
        CompressedInstructionInfo::lload_2 => {
            two_word_variable_load(method_frames, 2, current_index);
        }
        CompressedInstructionInfo::lload_3 => {
            two_word_variable_load(method_frames, 3, current_index);
        }
        CompressedInstructionInfo::lmul => {
            todo!()
        }
        CompressedInstructionInfo::lneg => {
            todo!()
        }
        CompressedInstructionInfo::lookupswitch(LookupSwitch{ pairs, default }) => {
            let frame = method_frames.nth_frame_mut(current_offset);
            frame.assert_operand_stack_entry_is(0,SimplifiedVType::OneWord);
            let mut operand_stack = frame.operand_stack();
            operand_stack.pop().unwrap().unwrap();
            let default = ByteCodeOffset((current_offset.0 as i32 + *default) as u16);
            method_frames.nth_frame_mut(default).assert_operand_stack_is(operand_stack.clone());
            for (_,offset) in pairs{
                let offset = ByteCodeOffset((current_offset.0 as i32 + *offset) as u16);
                method_frames.nth_frame_mut(offset).assert_operand_stack_is(operand_stack.clone());
            }
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
        CompressedInstructionInfo::lstore(n) => {
            two_word_variable_store(method_frames, *n, current_index)
        }
        CompressedInstructionInfo::lstore_0 => {
            two_word_variable_store(method_frames, 0, current_index)
        }
        CompressedInstructionInfo::lstore_1 => {
            two_word_variable_store(method_frames, 1, current_index)
        }
        CompressedInstructionInfo::lstore_2 => {
            two_word_variable_store(method_frames, 2, current_index)
        }
        CompressedInstructionInfo::lstore_3 => {
            two_word_variable_store(method_frames, 3, current_index)
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
            one_word_in_zero_out(method_frames, current_index);
        }
        CompressedInstructionInfo::monitorexit => {
            one_word_in_zero_out(method_frames, current_index);
        }
        CompressedInstructionInfo::multianewarray { .. } => {
            todo!()
        }
        CompressedInstructionInfo::new(_) => {
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::newarray(_) => {
            one_word_in_one_word_out(method_frames, current_index);
        }
        CompressedInstructionInfo::nop => {
            todo!()
        }
        CompressedInstructionInfo::pop => {
            one_word_in_zero_out(method_frames, current_index);
        }
        CompressedInstructionInfo::pop2 => {
            todo!()
        }
        CompressedInstructionInfo::putfield { desc, .. } => {
            let field_type = cpdtype_to_simplified_vtype(&desc.0);
            let (frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
            frame.assert_operand_stack_entry_is(0, field_type);
            frame.assert_operand_stack_entry_is(1, SimplifiedVType::OneWord);
            let mut operand_stack = frame.operand_stack();
            operand_stack.pop().unwrap();
            operand_stack.pop().unwrap();
            next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
        }
        CompressedInstructionInfo::putstatic { name: _, desc, target_class: _ } => {
            match cpdtype_to_simplified_vtype(&desc.0) {
                SimplifiedVType::OneWord => {
                    one_word_in_zero_out(method_frames, current_index);
                }
                SimplifiedVType::TwoWord => {
                    todo!()
                }
                SimplifiedVType::Top => {
                    panic!()
                }
            }
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
            one_word_const(method_frames, current_index);
        }
        CompressedInstructionInfo::swap => {
            let (frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
            frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
            frame.assert_operand_stack_entry_is(1, SimplifiedVType::OneWord);
            let operand_stack = frame.operand_stack();
            next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
        }
        CompressedInstructionInfo::tableswitch(box TableSwitch{ default, low:_, high:_, offsets }) => {
            let frame = method_frames.nth_frame_mut(current_offset);
            frame.assert_operand_stack_entry_is(0,SimplifiedVType::OneWord);
            let mut operand_stack = frame.operand_stack();
            operand_stack.pop().unwrap().unwrap();
            let default = ByteCodeOffset((current_offset.0 as i32 + *default) as u16);
            method_frames.nth_frame_mut(default).assert_operand_stack_is(operand_stack.clone());
            for offset in offsets{
                let offset = ByteCodeOffset((current_offset.0 as i32 + *offset) as u16);
                method_frames.nth_frame_mut(offset).assert_operand_stack_is(operand_stack.clone());
            }
        }
        CompressedInstructionInfo::wide(_) => {
            todo!()
        }
        CompressedInstructionInfo::EndOfCode => {
            todo!()
        }
    }
}

fn invoke(method_frames: &mut MethodFrames, i: usize, descriptor: &CMethodDescriptor, include_obj_ref: bool) {
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
                operand_stack.pop().unwrap();
            }
            SimplifiedVType::Top => {
                panic!()
            }
        }
    }
    if include_obj_ref {
        let should_be_one_word = operand_stack.pop().unwrap().unwrap();
        if let SimplifiedVType::OneWord = should_be_one_word {} else {
            panic!()
        }
    }
    if let CPDType::VoidType = descriptor.return_type {} else {
        match cpdtype_to_simplified_vtype(&descriptor.return_type) {
            SimplifiedVType::OneWord => {
                operand_stack.push(Some(SimplifiedVType::OneWord));
            }
            SimplifiedVType::TwoWord => {
                operand_stack.push(Some(SimplifiedVType::TwoWord));
            }
            SimplifiedVType::Top => {
                panic!()
            }
        }
    }
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn three_one_word_in_zero_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    current_frame.assert_operand_stack_entry_is(1, SimplifiedVType::OneWord);
    current_frame.assert_operand_stack_entry_is(2, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    let _ = operand_stack.pop().unwrap().unwrap();
    let _ = operand_stack.pop().unwrap().unwrap();
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn two_one_word_in_one_word_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    current_frame.assert_operand_stack_entry_is(1, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    let _ = operand_stack.pop().unwrap().unwrap();
    operand_stack.push(Some(SimplifiedVType::OneWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn two_two_word_in_one_word_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::TwoWord);
    current_frame.assert_operand_stack_entry_is(1, SimplifiedVType::TwoWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    let _ = operand_stack.pop().unwrap().unwrap();
    operand_stack.push(Some(SimplifiedVType::OneWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn one_two_word_in_one_word_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::TwoWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    operand_stack.push(Some(SimplifiedVType::OneWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}


fn top_operand_is_one_word_and_exit(method_frames: &mut MethodFrames, current_offset: ByteCodeOffset) {
    let current_frame = method_frames.nth_frame_mut(current_offset);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
}

fn one_word_in_one_word_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    let operand_stack = current_frame.operand_stack();
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn one_word_in_two_word_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    operand_stack.pop().unwrap().unwrap();
    operand_stack.push(Some(SimplifiedVType::TwoWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn one_word_in_zero_out(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    operand_stack.pop().unwrap().unwrap();
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
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


fn if_two_one_word(method_frames: &mut MethodFrames, current_offset: ByteCodeOffset, offset: i16) {
    let res_offset = (current_offset.0 as i32 + offset as i32) as u16;
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut_offset(current_offset);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    current_frame.assert_operand_stack_entry_is(1, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
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

fn two_word_const(method_frames: &mut MethodFrames, current_index: ByteCodeIndex) {
    let (frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    let mut operand_stack = frame.operand_stack();
    operand_stack.push(Some(SimplifiedVType::TwoWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn one_word_variable_load(method_frames: &mut MethodFrames, n: u8, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_local_is(n as u16, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    operand_stack.push(Some(SimplifiedVType::OneWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
    next_frame.as_mut().unwrap().assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
}

fn two_word_variable_load(method_frames: &mut MethodFrames, n: u8, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_local_is(n as u16, SimplifiedVType::TwoWord);
    let mut operand_stack = current_frame.operand_stack();
    operand_stack.push(Some(SimplifiedVType::TwoWord));
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn one_word_variable_store(method_frames: &mut MethodFrames, n: u8, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::OneWord);
    next_frame.as_mut().unwrap().assert_local_is(n as u16, SimplifiedVType::OneWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}

fn two_word_variable_store(method_frames: &mut MethodFrames, n: u8, current_index: ByteCodeIndex) {
    let (current_frame, mut next_frame) = method_frames.nth_frame_and_next_mut(current_index);
    current_frame.assert_operand_stack_entry_is(0, SimplifiedVType::TwoWord);
    next_frame.as_mut().unwrap().assert_local_is(n as u16, SimplifiedVType::TwoWord);
    let mut operand_stack = current_frame.operand_stack();
    let _ = operand_stack.pop().unwrap().unwrap();
    next_frame.as_mut().unwrap().assert_operand_stack_is(operand_stack);
}