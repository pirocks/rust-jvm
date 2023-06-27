use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use itertools::Itertools;
use another_jit_vm::IRMethodID;
use compiler_common::JavaCompilerMethodAndFrameData;
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::runtime_type::RuntimeType;
use strict_bi_hashmap::StrictBiHashMap;
use crate::ir_compiler_common::{BranchToLabelID, DoubleValueToken, FloatValueToken, IntegerValueToken, LongValueToken, PointerValueToken, Stage1IRInstr, TargetLabelID, TargetLabelIDInternal, ValueToken};
use crate::native_compiler_common::GeneralRegister;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRCompilerPosition {
    current_pc: ByteCodeOffset,
    index: ByteCodeIndex,
}

pub struct PendingExit{
    target_label_id: TargetLabelID,
    vm_exit_instr: Stage1IRInstr
}

pub struct TokenGen{
    current_token: u32
}

impl TokenGen{
    pub fn new() -> Self{
        Self{
            current_token: 0,
        }
    }

    pub fn new_pointer_token(&mut self) -> PointerValueToken{
        self.current_token += 1;
        PointerValueToken(self.current_token)
    }

    pub fn new_integer_token(&mut self) -> IntegerValueToken{
        self.current_token += 1;
        IntegerValueToken(self.current_token)
    }

    pub fn new_double_token(&mut self) -> DoubleValueToken{
        self.current_token += 1;
        DoubleValueToken(self.current_token)
    }

    pub fn new_long_token(&mut self) -> LongValueToken{
        self.current_token += 1;
        LongValueToken(self.current_token)
    }

    pub fn new_float_token(&mut self) -> FloatValueToken {
        self.current_token += 1;
        FloatValueToken(self.current_token)
    }
}

pub struct IRCompilerState<'l> {
    //res:
    pub(crate) res: Vec<Stage1IRInstr>,
    //current_position:
    current: Option<IRCompilerPosition>,
    //method_ids:
    method_id: MethodId,
    ir_method_id: IRMethodID,
    //pending exits:
    pending_exits: Vec<PendingExit>,
    //options:
    should_trace_instructions: bool,
    //frame_data
    method_frame_data: &'l JavaCompilerMethodAndFrameData,
    //label ids:
    pub(crate) labels: StrictBiHashMap<TargetLabelIDInternal, BranchToLabelID>,
    //token state:
    initial_arg_tokens: Vec<ValueToken>,
    pub(crate) current_local_var_tokens: Vec<ValueToken>,
    pub(crate) current_operand_stack_tokens: Vec<ValueToken>,
    res_token: OnceCell<ValueToken>,
    token_gen: TokenGen
}

impl <'l> IRCompilerState<'l> {
    pub fn new(
        method_id: MethodId,
        ir_method_id: IRMethodID,
        method_frame_data: &'l JavaCompilerMethodAndFrameData,
        desc: &CMethodDescriptor,
        should_trace_instructions: bool
    ) -> Self {
        let mut token_gen = TokenGen::new();
        let vec_capacity = method_frame_data.index_by_bytecode_offset.len()*2;
        let mut current_local_var_tokens = (0..method_frame_data.local_vars).map(|_| ValueToken::Top).collect_vec();
        let mut initial_arg_tokens = vec![];
        for (i, arg_type) in desc.arg_types.iter().enumerate() {
            let value_token = match arg_type.to_runtime_type() {
                None => ValueToken::Top,
                Some(RuntimeType::IntType) => ValueToken::Integer(token_gen.new_integer_token()),
                Some(RuntimeType::FloatType) => ValueToken::Float(token_gen.new_float_token()),
                Some(RuntimeType::DoubleType) => ValueToken::Double(token_gen.new_double_token()),
                Some(RuntimeType::LongType) => ValueToken::Long(token_gen.new_long_token()),
                Some(RuntimeType::Ref(_)) => ValueToken::Pointer(token_gen.new_pointer_token()),
                Some(RuntimeType::TopType) => ValueToken::Top,
            };
            initial_arg_tokens.push(value_token);
            current_local_var_tokens[i] = value_token;
        }
        Self {
            method_frame_data,
            current: None,
            method_id,
            ir_method_id,
            res: Vec::with_capacity(vec_capacity),
            pending_exits: vec![],
            should_trace_instructions,
            labels: StrictBiHashMap::new(),
            initial_arg_tokens,
            current_local_var_tokens,
            current_operand_stack_tokens: vec![],
            res_token: OnceCell::new(),
            token_gen,
        }
    }

    pub fn complete(self) -> Vec<Stage1IRInstr> {
        todo!()
    }

    pub fn emit_ir_start(&mut self) {
        self.res.push(Stage1IRInstr::IRStart {
            ir_method_id: self.ir_method_id,
            method_id: self.method_id,
            frame_size: self.method_frame_data.full_frame_size(),//todo actual frame size needs to be calculated after the fact todo.
        })
    }

    pub fn emit_ir_end(&mut self, return_val: ValueToken) {
        self.res_token.set(return_val).expect("todo");
        self.res.push(todo!());
    }

    pub fn emit_native_function_start(&mut self, _registers_to_save: HashSet<GeneralRegister>) -> HashMap<GeneralRegister, PointerValueToken>{
        todo!()
    }

    pub fn emit_monitor_enter(&mut self, obj: PointerValueToken) {
        self.res.push(Stage1IRInstr::MonitorEnter { java_pc: self.current.unwrap().current_pc, obj })
    }

    pub fn emit_get_class_object(&mut self) -> PointerValueToken {
        todo!()
    }

    pub fn emit_load_arg_pointer(&mut self, _arg_num: u16) -> PointerValueToken {
        todo!()
    }

    pub fn notify_before_instruction(&mut self, current_pc: ByteCodeOffset, byte_code_index: ByteCodeIndex) {
        if self.should_trace_instructions{
            todo!("emit tracing exits")
        }
        self.current = Some(IRCompilerPosition{ current_pc, index: byte_code_index })
    }

    pub fn notify_after_instruction(&mut self, _byte_code_offset: ByteCodeOffset) {
        todo!()
    }

    // fn lookup_local_var_token_pointer(&self, local_var_index: u16) -> PointerValueToken{
    //     self.current_local_var_tokens[local_var_index as usize].unwrap_pointer()
    // }
}
