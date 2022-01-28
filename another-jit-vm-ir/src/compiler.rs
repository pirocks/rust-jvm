use std::ffi::c_void;

use another_jit_vm::Register;
use gc_memory_layout_common::FramePointerOffset;
use rust_jvm_common::MethodId;

use crate::{IRMethodID, IRVMExitType};

#[derive(Debug)]
pub enum IRInstr {
    LoadFPRelative { from: FramePointerOffset, to: Register },
    StoreFPRelative { from: Register, to: FramePointerOffset },
    Load { to: Register, from_address: Register },
    Store { to_address: Register, from: Register },
    CopyRegister { from: Register, to: Register },
    Add { res: Register, a: Register },
    Sub { res: Register, to_subtract: Register },
    Div { res: Register, divisor: Register },
    Mod { res: Register, divisor: Register },
    Mul { res: Register, a: Register },
    BinaryBitAnd { res: Register, a: Register },
    ForwardBitScan { to_scan: Register, res: Register },
    Const32bit { to: Register, const_: u32 },
    Const64bit { to: Register, const_: u64 },
    BranchToLabel { label: LabelName },
    LoadLabel { label: LabelName, to: Register },
    LoadRBP { to: Register },
    WriteRBP { from: Register },
    BranchEqual { a: Register, b: Register, label: LabelName },
    BranchNotEqual { a: Register, b: Register, label: LabelName },
    Return { return_val: Option<Register>, temp_register_1: Register, temp_register_2: Register, temp_register_3: Register, temp_register_4: Register, frame_size: usize },
    // VMExit { before_exit_label: LabelName, after_exit_label: Option<LabelName>, exit_type: VMExitTypeWithArgs },
    RestartPoint(RestartPointID),
    VMExit2 { exit_type: IRVMExitType },
    NPECheck { possibly_null: Register, temp_register: Register, npe_exit_type: IRVMExitType },
    GrowStack { amount: usize },
    LoadSP { to: Register },
    // WithAssembler { function: Box<dyn FnOnce(&mut CodeAssembler) -> ()> },
    IRNewFrame {
        current_frame_size: usize,
        temp_register: Register,
        return_to_rip: Register,
    },
    IRCall {
        temp_register_1: Register,
        temp_register_2: Register,
        arg_from_to_offsets: Vec<(FramePointerOffset, FramePointerOffset)>,
        return_value: Option<FramePointerOffset>,
        target_address: IRCallTarget,
    },
    NOP,
    DebuggerBreakpoint,
    Label(IRLabel),
}

#[derive(Debug)]
pub enum IRCallTarget {
    Constant {
        address: *const c_void,
        ir_method_id: IRMethodID,
        method_id: MethodId,
        new_frame_size: usize,
    },
    Variable{
        address: Register,
        ir_method_id: Register,
        method_id: Register,
        new_frame_size: Register,
    },
}

impl IRInstr {
    pub fn debug_string(&self) -> String {
        match self {
            IRInstr::LoadFPRelative { .. } => {
                "LoadFPRelative".to_string()
            }
            IRInstr::StoreFPRelative { .. } => {
                "StoreFPRelative".to_string()
            }
            IRInstr::Load { .. } => {
                "Load".to_string()
            }
            IRInstr::Store { .. } => {
                "Store".to_string()
            }
            IRInstr::CopyRegister { .. } => {
                "CopyRegister".to_string()
            }
            IRInstr::Add { .. } => {
                "Add".to_string()
            }
            IRInstr::Sub { .. } => {
                "Sub".to_string()
            }
            IRInstr::Div { .. } => {
                "Div".to_string()
            }
            IRInstr::Mod { .. } => {
                "Mod".to_string()
            }
            IRInstr::Mul { .. } => {
                "Mul".to_string()
            }
            IRInstr::BinaryBitAnd { .. } => {
                "BinaryBitAnd".to_string()
            }
            IRInstr::ForwardBitScan { .. } => {
                "ForwardBitScan".to_string()
            }
            IRInstr::Const32bit { .. } => {
                "Const32bit".to_string()
            }
            IRInstr::Const64bit { .. } => {
                "Const64bit".to_string()
            }
            IRInstr::BranchToLabel { .. } => {
                "BranchToLabel".to_string()
            }
            IRInstr::LoadLabel { .. } => {
                "LoadLabel".to_string()
            }
            IRInstr::LoadRBP { .. } => {
                "LoadRBP".to_string()
            }
            IRInstr::WriteRBP { .. } => {
                "WriteRBP".to_string()
            }
            IRInstr::BranchEqual { .. } => {
                "BranchEqual".to_string()
            }
            IRInstr::BranchNotEqual { .. } => {
                "BranchNotEqual".to_string()
            }
            IRInstr::Return { .. } => {
                "Return".to_string()
            }
            IRInstr::RestartPoint(_) => {
                "RestartPoint".to_string()
            }
            IRInstr::VMExit2 { exit_type } => {
                format!("VMExit2-{}", match exit_type {
                    IRVMExitType::AllocateObjectArray_ { .. } => { "AllocateObjectArray_" }
                    IRVMExitType::NPE => { "NPE" }
                    IRVMExitType::LoadClassAndRecompile { .. } => { "LoadClassAndRecompile" }
                    IRVMExitType::InitClassAndRecompile { .. } => { "InitClassAndRecompile" }
                    IRVMExitType::RunStaticNative { .. } => { "RunStaticNative" }
                    IRVMExitType::CompileFunctionAndRecompileCurrent { .. } => { "CompileFunctionAndRecompileCurrent" }
                    IRVMExitType::TopLevelReturn => { "TopLevelReturn" }
                    IRVMExitType::PutStatic { .. } => { "PutStatic" }
                    IRVMExitType::LogFramePointerOffsetValue { .. } => { "LogFramePointerOffsetValue" }
                    IRVMExitType::LogWholeFrame { .. } => { "LogWholeFrame" }
                    IRVMExitType::TraceInstructionBefore { .. } => { "TraceInstructionBefore" }
                    IRVMExitType::TraceInstructionAfter { .. } => { "TraceInstructionAfter" }
                    IRVMExitType::BeforeReturn { .. } => { "BeforeReturn" }
                    IRVMExitType::AllocateObject { .. } => { "AllocateObject" }
                    IRVMExitType::NewString { .. } => { "NewString" }
                    IRVMExitType::NewClass { .. } => { "NewClass" }
                    IRVMExitType::InvokeVirtualResolve { .. } => {"InvokeVirtualResolve"}
                })
            }
            IRInstr::NPECheck { .. } => {
                "NPECheck".to_string()
            }
            IRInstr::GrowStack { .. } => {
                "GrowStack".to_string()
            }
            IRInstr::LoadSP { .. } => {
                "LoadSP".to_string()
            }
            IRInstr::IRNewFrame { .. } => {
                "IRNewFrame".to_string()
            }
            IRInstr::IRCall { .. } => {
                "IRCall".to_string()
            }
            IRInstr::NOP => {
                "FNOP".to_string()
            }
            IRInstr::Label(_) => {
                "Label".to_string()
            }
            IRInstr::DebuggerBreakpoint => {
                "DebuggerBreakpoint".to_string()
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct RestartPointID(pub(crate) u64);


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LabelName(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRLabel {
    pub name: LabelName,
}


pub struct RestartPointGenerator {
    current_max_restart_point: RestartPointID,
}

impl RestartPointGenerator {
    pub fn new() -> Self {
        Self {
            current_max_restart_point: RestartPointID(0)
        }
    }

    pub fn new_restart_point(&mut self) -> RestartPointID {
        let res = self.current_max_restart_point;
        self.current_max_restart_point.0 += 1;
        res
    }
}
