use std::ffi::c_void;

use another_jit_vm::{DoubleRegister, FloatRegister, MMRegister, Register};
use gc_memory_layout_common::FramePointerOffset;
use rust_jvm_common::MethodId;

use crate::{IRMethodID, IRVMExitType};

#[derive(Debug, Clone)]
pub enum IRInstr {
    LoadFPRelative { from: FramePointerOffset, to: Register },
    LoadFPRelativeFloat { from: FramePointerOffset, to: FloatRegister },
    LoadFPRelativeDouble { from: FramePointerOffset, to: DoubleRegister },
    StoreFPRelative { from: Register, to: FramePointerOffset },
    StoreFPRelativeFloat { from: FloatRegister, to: FramePointerOffset },
    StoreFPRelativeDouble { from: DoubleRegister, to: FramePointerOffset },
    FloatToIntegerConvert { from: FloatRegister, temp: MMRegister, to: Register },
    DoubleToIntegerConvert { from: DoubleRegister, temp: MMRegister, to: Register },
    FloatToDoubleConvert { from: FloatRegister, to: DoubleRegister },
    IntegerToFloatConvert { to: FloatRegister, temp: MMRegister, from: Register },
    IntegerToDoubleConvert { to: DoubleRegister, temp: MMRegister, from: Register },
    Load { to: Register, from_address: Register },
    Load32 { to: Register, from_address: Register },
    Store { to_address: Register, from: Register },
    CopyRegister { from: Register, to: Register },
    Add { res: Register, a: Register },
    IntCompare { res: Register, value1: Register, value2: Register, temp1: Register, temp2: Register, temp3: Register },
    AddFloat { res: FloatRegister, a: FloatRegister },
    Sub { res: Register, to_subtract: Register },
    Div { res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register },
    DivFloat { res: FloatRegister, divisor: FloatRegister },
    Mod { res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register },
    Mul { res: Register, a: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register },
    MulFloat { res: FloatRegister, a: FloatRegister },
    MulDouble { res: DoubleRegister, a: DoubleRegister },
    MulConst { res: Register, a: i32 },
    ArithmeticShiftLeft { res: Register, a: Register, cl_aka_register_2: Register },
    LogicalShiftRight { res: Register, a: Register, cl_aka_register_2: Register },
    ArithmeticShiftRight { res: Register, a: Register, cl_aka_register_2: Register },
    BinaryBitAnd { res: Register, a: Register },
    BinaryBitXor { res: Register, a: Register },
    BinaryBitOr { res: Register, a: Register },
    ForwardBitScan { to_scan: Register, res: Register },
    Const16bit { to: Register, const_: u16 },
    Const32bit { to: Register, const_: u32 },
    Const64bit { to: Register, const_: u64 },
    BranchToLabel { label: LabelName },
    LoadLabel { label: LabelName, to: Register },
    LoadRBP { to: Register },
    WriteRBP { from: Register },
    BranchEqual { a: Register, b: Register, label: LabelName },
    BranchNotEqual { a: Register, b: Register, label: LabelName },
    BranchAGreaterB { a: Register, b: Register, label: LabelName },
    FloatCompare { value1: FloatRegister, value2: FloatRegister, res: Register, temp1: Register, temp2: Register, temp3: Register, compare_mode: FloatCompareMode },
    BranchAGreaterEqualB { a: Register, b: Register, label: LabelName },
    BranchALessB { a: Register, b: Register, label: LabelName },
    BoundsCheck { length: Register, index: Register },
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
        current_frame_size: usize,
    },
    NOP,
    DebuggerBreakpoint,
    Label(IRLabel),
}

#[derive(Debug, Clone)]
pub enum FloatCompareMode {
    G,
    L,
}

#[derive(Debug, Clone)]
pub enum IRCallTarget {
    Constant {
        address: *const c_void,
        ir_method_id: IRMethodID,
        method_id: MethodId,
        new_frame_size: usize,
    },
    Variable {
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
            IRInstr::RestartPoint(id) => {
                format!("RestartPoint #{}",id.0)
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
                    IRVMExitType::InvokeVirtualResolve { .. } => { "InvokeVirtualResolve" }
                    IRVMExitType::MonitorEnter { .. } => { "MonitorEnter" }
                    IRVMExitType::MonitorExit { .. } => { "MonitorExit" }
                    IRVMExitType::Throw { .. } => { "Throw" }
                    IRVMExitType::GetStatic { .. } => { "GetStatic" }
                    IRVMExitType::Todo => { "Todo" }
                    IRVMExitType::InstanceOf { .. } => { "InstanceOf" }
                    IRVMExitType::CheckCast { .. } => { "CheckCast" }
                    IRVMExitType::RunNativeVirtual { .. } => { "RunNativeVirtual" }
                    IRVMExitType::RunNativeSpecial { .. } => { "RunNativeSpecial" }
                    IRVMExitType::InvokeInterfaceResolve { .. } => { "InvokeInterfaceResolve" }
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
            IRInstr::Load32 { .. } => {
                "Load32".to_string()
            }
            IRInstr::Const16bit { .. } => {
                "Const16bit".to_string()
            }
            IRInstr::BranchAGreaterB { .. } => {
                "BranchAGreaterB".to_string()
            }
            IRInstr::BranchALessB { .. } => {
                "BranchALessB".to_string()
            }
            IRInstr::BranchAGreaterEqualB { .. } => {
                "BranchAGreaterEqualB".to_string()
            }
            IRInstr::ArithmeticShiftLeft { .. } => {
                "LogicalShiftLeft".to_string()
            }
            IRInstr::BoundsCheck { .. } => {
                "BoundsCheck".to_string()
            }
            IRInstr::MulConst { .. } => {
                "MulConst".to_string()
            }
            IRInstr::LoadFPRelativeFloat { .. } => {
                "LoadFPRelativeFloat".to_string()
            }
            IRInstr::StoreFPRelativeFloat { .. } => {
                "StoreFPRelativeFloat".to_string()
            }
            IRInstr::FloatToIntegerConvert { .. } => {
                "FloatToIntegerConvert".to_string()
            }
            IRInstr::IntegerToFloatConvert { .. } => {
                "IntegerToFloatConvert".to_string()
            }
            IRInstr::FloatCompare { .. } => {
                "FloatCompare".to_string()
            }
            IRInstr::MulFloat { .. } => {
                "MulFloat".to_string()
            }
            IRInstr::LogicalShiftRight { .. } => {
                "LogicalShiftRight".to_string()
            }
            IRInstr::BinaryBitXor { .. } => {
                "BinaryBitXor".to_string()
            }
            IRInstr::DivFloat { .. } => {
                "DivFloat".to_string()
            }
            IRInstr::AddFloat { .. } => {
                "AddFloat".to_string()
            }
            IRInstr::ArithmeticShiftRight { .. } => {
                "ArithmeticShiftRight".to_string()
            }
            IRInstr::IntCompare { .. } => {
                "IntCompare".to_string()
            }
            IRInstr::BinaryBitOr { .. } => {
                "BinaryBitOr".to_string()
            }
            IRInstr::DoubleToIntegerConvert { .. } => {
                "DoubleToIntegerConvert".to_string()
            }
            IRInstr::IntegerToDoubleConvert { .. } => {
                "IntegerToDoubleConvert".to_string()
            }
            IRInstr::LoadFPRelativeDouble { .. } => {
                "LoadFPRelativeDouble".to_string()
            }
            IRInstr::StoreFPRelativeDouble { .. } => {
                "StoreFPRelativeDouble".to_string()
            }
            IRInstr::FloatToDoubleConvert { .. } => {
                "FloatToDoubleConvert".to_string()
            }
            IRInstr::MulDouble { .. } => {
                "MulDouble".to_string()
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
