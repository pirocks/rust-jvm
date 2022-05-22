use std::ffi::c_void;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use another_jit_vm::{DoubleRegister, FloatRegister, FramePointerOffset, IRMethodID, MMRegister, Register};
use rust_jvm_common::MethodId;
use crate::changeable_const::ChangeableConstID;

use crate::{IRVMExitType};
use crate::skipable_exits::SkipableExitID;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Size {
    Byte,
    X86Word,
    X86DWord,
    X86QWord,
}

impl Size {
    pub const fn int() -> Self {
        Self::X86DWord
    }

    pub const fn float() -> Self {
        Self::X86DWord
    }

    pub const fn short() -> Self {
        Self::X86Word
    }

    pub const fn char() -> Self {
        Self::X86Word
    }

    pub const fn byte() -> Self {
        Self::Byte
    }

    pub const fn pointer() -> Self {
        Self::X86QWord
    }

    pub const fn double() -> Self {
        Self::X86QWord
    }

    pub const fn long() -> Self {
        Self::X86QWord
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Signed {
    Signed,
    Unsigned,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BitwiseLogicType {
    Arithmetic,
    Logical,
}

pub struct ChangeableConst64Entries {
    entries: Mutex<Vec<ChangeableConst64Entry>>,
}

impl ChangeableConst64Entries {
    pub fn new_entry(&self, entry: u64) -> ChangeableConst64Entry {
        let mut mutex_guard = self.entries.lock().unwrap();
        let res_ref = Box::leak(box AtomicU64::new(entry));
        mutex_guard.push(ChangeableConst64Entry(res_ref));
        ChangeableConst64Entry(res_ref)
    }
}

#[derive(Debug, Clone)]
pub struct ChangeableConst64Entry(&'static AtomicU64);

#[derive(Debug, Clone)]
pub enum IRInstr {
    LoadFPRelative { from: FramePointerOffset, to: Register, size: Size },
    LoadFPRelativeFloat { from: FramePointerOffset, to: FloatRegister },
    LoadFPRelativeDouble { from: FramePointerOffset, to: DoubleRegister },
    StoreFPRelative { from: Register, to: FramePointerOffset, size: Size },
    StoreFPRelativeFloat { from: FloatRegister, to: FramePointerOffset },
    StoreFPRelativeDouble { from: DoubleRegister, to: FramePointerOffset },
    FloatToIntegerConvert { from: FloatRegister, temp: MMRegister, to: Register },
    DoubleToIntegerConvert { from: DoubleRegister, temp: MMRegister, to: Register },
    DoubleToLongConvert { from: DoubleRegister, to: Register },
    FloatToDoubleConvert { from: FloatRegister, to: DoubleRegister },
    IntegerToFloatConvert { to: FloatRegister, temp: MMRegister, from: Register },
    LongToFloatConvert { to: FloatRegister, from: Register },
    LongToDoubleConvert { to: FloatRegister, from: Register },
    IntegerToDoubleConvert { to: DoubleRegister, temp: MMRegister, from: Register },
    Load { to: Register, from_address: Register, size: Size },
    Store { to_address: Register, from: Register, size: Size },
    CopyRegister { from: Register, to: Register },
    Add { res: Register, a: Register, size: Size },
    IntCompare { res: Register, value1: Register, value2: Register, temp1: Register, temp2: Register, temp3: Register, size: Size },
    AddFloat { res: FloatRegister, a: FloatRegister },
    SubFloat { res: FloatRegister, a: FloatRegister },
    SubDouble { res: DoubleRegister, a: DoubleRegister },
    AddDouble { res: DoubleRegister, a: DoubleRegister },
    Sub { res: Register, to_subtract: Register, size: Size },
    Div { res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: Signed },
    DivFloat { res: FloatRegister, divisor: FloatRegister },
    Mod { res: Register, divisor: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: Signed },
    Mul { res: Register, a: Register, must_be_rax: Register, must_be_rbx: Register, must_be_rcx: Register, must_be_rdx: Register, size: Size, signed: Signed },
    MulFloat { res: FloatRegister, a: FloatRegister },
    MulDouble { res: DoubleRegister, a: DoubleRegister },
    MulConst { res: Register, a: i32, size: Size, signed: Signed },
    ShiftLeft { res: Register, a: Register, cl_aka_register_2: Register, size: Size, signed: BitwiseLogicType },
    ShiftRight { res: Register, a: Register, cl_aka_register_2: Register, size: Size, signed: BitwiseLogicType },
    BinaryBitAnd { res: Register, a: Register, size: Size },
    BinaryBitXor { res: Register, a: Register, size: Size },
    BinaryBitOr { res: Register, a: Register, size: Size },
    Const16bit { to: Register, const_: u16 },
    Const32bit { to: Register, const_: u32 },
    Const64bit { to: Register, const_: u64 },
    ChangeableConst64bit { to: Register, const_id: ChangeableConstID },
    // OneTimeChangeablePutField {},
    SignExtend { from: Register, to: Register, from_size: Size, to_size: Size },
    ZeroExtend { from: Register, to: Register, from_size: Size, to_size: Size },
    BranchToLabel { label: LabelName },
    LoadLabel { label: LabelName, to: Register },
    LoadRBP { to: Register },
    WriteRBP { from: Register },
    FloatCompare { value1: FloatRegister, value2: FloatRegister, res: Register, temp1: Register, temp2: Register, temp3: Register, compare_mode: FloatCompareMode },
    DoubleCompare { value1: DoubleRegister, value2: DoubleRegister, res: Register, temp1: Register, temp2: Register, temp3: Register, compare_mode: FloatCompareMode },
    BranchEqual { a: Register, b: Register, label: LabelName, size: Size },
    BranchNotEqual { a: Register, b: Register, label: LabelName, size: Size },
    BranchAGreaterB { a: Register, b: Register, label: LabelName, size: Size },
    BranchAGreaterEqualB { a: Register, b: Register, label: LabelName, size: Size },
    BranchALessB { a: Register, b: Register, label: LabelName, size: Size },
    BoundsCheck { length: Register, index: Register, size: Size },
    Return { return_val: Option<Register>, temp_register_1: Register, temp_register_2: Register, temp_register_3: Register, temp_register_4: Register, frame_size: usize },
    RestartPoint(RestartPointID),
    VTableLookupOrExit {
        resolve_exit: IRVMExitType
    },
    VMExit2 {
        exit_type: IRVMExitType,
        skipable_exit_id: Option<SkipableExitID>,
    },
    NPECheck { possibly_null: Register, temp_register: Register, npe_exit_type: IRVMExitType },
    IRCall {
        temp_register_1: Register,
        temp_register_2: Register,
        arg_from_to_offsets: Vec<(FramePointerOffset, FramePointerOffset)>,
        return_value: Option<FramePointerOffset>,
        target_address: IRCallTarget,
        current_frame_size: usize,
    },
    IRStart {
        temp_register: Register,
        ir_method_id: IRMethodID,
        method_id: MethodId,
        frame_size: usize,
        num_locals: usize,
    },
    NOP,
    DebuggerBreakpoint,
    Label(IRLabel),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FloatCompareMode {
    G,
    L,
}

#[derive(Debug, Copy, Clone)]
pub enum IRCallTarget {
    Constant {
        address: *const c_void,
        method_id: MethodId,
    },
    Variable {
        address: Register,
    },
    RegisteredUnknown {
        method_id: MethodId
    },
    UnRegistered{
        changeable_const: ChangeableConstID
    }
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
                format!("RestartPoint #{}", id.0)
            }
            IRInstr::VMExit2 { exit_type, skipable_exit_id: _ } => {
                format!("VMExit2-{}", match exit_type {
                    IRVMExitType::AllocateObjectArray_ { .. } => { "AllocateObjectArray_" }
                    IRVMExitType::NPE { .. } => { "NPE" }
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
                    IRVMExitType::MultiAllocateObjectArray_ { .. } => {
                        "MultiAllocateObjectArray_"
                    }
                    IRVMExitType::RunStaticNativeNew { .. } => {
                        "RunStaticNativeNew"
                    }
                    IRVMExitType::RunSpecialNativeNew { .. } => {
                        "RunSpecialNativeNew"
                    }
                })
            }
            IRInstr::NPECheck { .. } => {
                "NPECheck".to_string()
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
            IRInstr::BinaryBitXor { .. } => {
                "BinaryBitXor".to_string()
            }
            IRInstr::DivFloat { .. } => {
                "DivFloat".to_string()
            }
            IRInstr::AddFloat { .. } => {
                "AddFloat".to_string()
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
            IRInstr::ShiftLeft { .. } => {
                "ShiftLeft".to_string()
            }
            IRInstr::ShiftRight { .. } => {
                "ShiftRight".to_string()
            }
            IRInstr::SignExtend { .. } => {
                "SignExtend".to_string()
            }
            IRInstr::LongToFloatConvert { .. } => {
                "LongToFloatConvert".to_string()
            }
            IRInstr::AddDouble { .. } => {
                "AddDouble".to_string()
            }
            IRInstr::DoubleToLongConvert { .. } => {
                "DoubleToLongConvert".to_string()
            }
            IRInstr::LongToDoubleConvert { .. } => {
                "LongToDoubleConvert".to_string()
            }
            IRInstr::ZeroExtend { .. } => {
                "ZeroExtend".to_string()
            }
            IRInstr::DoubleCompare { .. } => {
                "DoubleCompare".to_string()
            }
            IRInstr::SubFloat { .. } => {
                "SubFloat".to_string()
            }
            IRInstr::VTableLookupOrExit { .. } => {
                "VTableLookupOrExit".to_string()
            }
            IRInstr::SubDouble { .. } => {
                "SubDouble".to_string()
            }
            IRInstr::IRStart { .. } => {
                "IRStart".to_string()
            }
            /*IRInstr::ChangeableConst64bit { .. } => {
                "ChangeableConst64bit".to_string()
            }*/
            /*IRInstr::OneTimeChangeablePutField { .. } => {
                "OneTimeChangeablePutField".to_string()
            }*/
            IRInstr::ChangeableConst64bit { .. } => {
                "ChangeableConst64bit".to_string()
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
