use std::collections::HashSet;

use another_jit_vm::Register;
use rust_jvm_common::compressed_classfile::CPDType;

pub trait ExitRegisterStruct {
    fn all_registers() -> HashSet<Register>;
}

pub struct AllocateObjectArray;

impl AllocateObjectArray {
    pub const LEN: Register = Register(2);
    pub const TYPE: Register = Register(3);
    pub const RES_PTR: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for AllocateObjectArray {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), Self::RES_PTR, Self::RESTART_IP, Self::LEN, Self::TYPE, Self::JAVA_PC])
    }
}

pub struct MultiAllocateArray;

impl MultiAllocateArray {
    pub const LEN_START: Register = Register(2);
    pub const ELEM_TYPE: Register = Register(3);
    pub const NUM_ARRAYS: Register = Register(4);
    pub const RES_PTR: Register = Register(5);
    pub const RESTART_IP: Register = Register(6);
    pub const JAVA_PC: Register = Register(7);
}

impl ExitRegisterStruct for MultiAllocateArray {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0),
            Self::LEN_START,
            Self::ELEM_TYPE,
            Self::NUM_ARRAYS,
            Self::RES_PTR,
            Self::RESTART_IP,
            Self::JAVA_PC
        ])
    }
}

pub struct AllocateObject;

impl AllocateObject {
    pub const TYPE: Register = Register(3);
    pub const RES_PTR: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);

}

impl ExitRegisterStruct for AllocateObject {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), AllocateObject::TYPE, AllocateObject::RES_PTR, AllocateObject::RESTART_IP, Self::JAVA_PC])
    }
}

pub struct RunStaticNative;

impl RunStaticNative {
    pub const RES: Register = Register(1);
    pub const ARG_START: Register = Register(2);
    pub const NUM_ARGS: Register = Register(3);
    pub const METHODID: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for RunStaticNative{
    fn all_registers() -> HashSet<Register> {
        todo!()
    }
}

pub struct RunStaticNativeNew;

impl RunStaticNativeNew{
    pub const METHOD_ID: Register = Register(2);
}

impl ExitRegisterStruct for RunStaticNativeNew{
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), Self::METHOD_ID])
    }
}

pub struct RunNativeVirtual;

impl RunNativeVirtual {
    pub const RES_PTR: Register = Register(2);
    pub const ARG_START: Register = Register(3);
    pub const METHODID: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for RunNativeVirtual {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), Self::RES_PTR, Self::ARG_START, Self::METHODID, Self::RESTART_IP])
    }
}

pub struct RunNativeSpecial;

impl RunNativeSpecial {
    pub const RES_PTR: Register = Register(2);
    pub const ARG_START: Register = Register(3);
    pub const METHODID: Register = Register(4);
    pub const RESTART_IP: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for RunNativeSpecial {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0),
            Self::RES_PTR,
            Self::ARG_START,
            Self::METHODID,
            Self::RESTART_IP,
            Self::JAVA_PC
        ])
    }
}

pub struct TopLevelReturn;

impl TopLevelReturn {
    pub const RES: Register = Register(2);
    pub const JAVA_PC: Register = Register(3);

}

pub struct PutStatic;

impl PutStatic {
    pub const FIELD_ID: Register = Register(2);
    pub const VALUE_PTR: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);

}

impl ExitRegisterStruct for PutStatic {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), PutStatic::VALUE_PTR, PutStatic::FIELD_ID, PutStatic::RESTART_IP, Self::JAVA_PC])
    }
}


pub struct GetStatic;

impl GetStatic {
    pub const FIELD_NAME: Register = Register(2);
    pub const RES_VALUE_PTR: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const CPDTYPE_ID: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for GetStatic {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), Self::FIELD_NAME, Self::RES_VALUE_PTR, Self::RESTART_IP, Self::CPDTYPE_ID, Self::JAVA_PC])
    }
}

pub struct Throw;

impl Throw {
    pub const EXCEPTION_PTR: Register = Register(2);
    pub const JAVA_PC: Register = Register(3);
}

impl ExitRegisterStruct for Throw {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), Throw::EXCEPTION_PTR, Self::JAVA_PC])
    }
}

pub struct InitClassAndRecompile;

impl InitClassAndRecompile {
    pub const CPDTYPE_ID: Register = Register(2);
    pub const TO_RECOMPILE: Register = Register(3);
    pub const RESTART_POINT_ID: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

pub struct CompileFunctionAndRecompileCurrent;

impl CompileFunctionAndRecompileCurrent {
    pub const CURRENT: Register = Register(2);
    pub const TO_RECOMPILE: Register = Register(3);
    pub const RESTART_POINT_ID: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

impl ExitRegisterStruct for CompileFunctionAndRecompileCurrent {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), CompileFunctionAndRecompileCurrent::TO_RECOMPILE, CompileFunctionAndRecompileCurrent::CURRENT, CompileFunctionAndRecompileCurrent::RESTART_POINT_ID, Self::JAVA_PC])
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct LoadClassAndRecompileStaticArgs {
    class_type: CPDType,
}

pub struct LoadClassAndRecompile;

impl LoadClassAndRecompile {
    pub const CPDTYPE_ID: Register = Register(2);
    pub const TO_RECOMPILE: Register = Register(3);
    pub const RESTART_POINT_ID: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

pub struct LogFramePointerOffsetValue;

impl LogFramePointerOffsetValue {
    pub const VALUE: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const JAVA_PC: Register = Register(4);
}

pub struct LogWholeFrame;

impl LogWholeFrame {
    pub const RESTART_IP: Register = Register(2);
    pub const JAVA_PC: Register = Register(3);
}

pub struct TraceInstructionBefore;

impl TraceInstructionBefore {
    pub const METHOD_ID: Register = Register(2);
    pub const BYTECODE_OFFSET: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

impl ExitRegisterStruct for TraceInstructionBefore {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), TraceInstructionBefore::METHOD_ID, TraceInstructionBefore::BYTECODE_OFFSET, TraceInstructionBefore::RESTART_IP, Self::JAVA_PC])
    }
}

pub struct TraceInstructionAfter;

impl TraceInstructionAfter {
    pub const METHOD_ID: Register = Register(2);
    pub const BYTECODE_OFFSET: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

impl ExitRegisterStruct for TraceInstructionAfter {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), TraceInstructionAfter::METHOD_ID, TraceInstructionAfter::BYTECODE_OFFSET, TraceInstructionAfter::RESTART_IP, Self::JAVA_PC])
    }
}

pub struct BeforeReturn;

impl BeforeReturn {
    pub const FRAME_SIZE: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const JAVA_PC: Register = Register(4);
}

pub struct NewString;

impl NewString {
    pub const COMPRESSED_WTF8: Register = Register(2);
    pub const RES: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

impl ExitRegisterStruct for NewString {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), NewString::RES, NewString::RESTART_IP, NewString::COMPRESSED_WTF8, Self::JAVA_PC])
    }
}

pub struct NewClass;

impl NewClass {
    pub const CPDTYPE_ID: Register = Register(2);
    pub const RES: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const JAVA_PC: Register = Register(5);
}

impl ExitRegisterStruct for NewClass {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), NewClass::RES, NewClass::CPDTYPE_ID, NewClass::RESTART_IP, Self::JAVA_PC])
    }
}

pub struct InstanceOf;

impl InstanceOf {
    pub const VALUE_PTR: Register = Register(2);
    pub const RES_VALUE_PTR: Register = Register(3);
    pub const RESTART_IP: Register = Register(4);
    pub const CPDTYPE_ID: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for InstanceOf {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0),
            Self::VALUE_PTR,
            Self::RES_VALUE_PTR,
            Self::RESTART_IP,
            Self::CPDTYPE_ID,
            Self::JAVA_PC
        ])
    }
}

pub struct CheckCast;

impl CheckCast {
    pub const VALUE_PTR: Register = Register(2);
    pub const RESTART_IP: Register = Register(4);
    pub const CPDTYPE_ID: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
}

impl ExitRegisterStruct for CheckCast {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), CheckCast::RESTART_IP, CheckCast::VALUE_PTR, CheckCast::CPDTYPE_ID, Self::JAVA_PC])
    }
}

pub struct InvokeVirtualResolve;

impl InvokeVirtualResolve {
    pub const OBJECT_REF_PTR: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const METHOD_NUMBER: Register = Register(4);
    pub const NATIVE_RETURN_PTR: Register = Register(5);
    pub const JAVA_PC: Register = Register(6);
    pub const METHOD_SHAPE_ID: Register = Register(8);
    pub const NATIVE_RESTART_POINT: Register = Register(9);
    pub const METHOD_ID_RES: Register = Register(6);
    pub const IR_METHOD_ID_RES: Register = Register(5);
    pub const NEW_FRAME_SIZE_RES: Register = Register(7);
    pub const ADDRESS_RES: Register = Register(4);
}

impl ExitRegisterStruct for InvokeVirtualResolve {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([
            Register(0),
            Self::OBJECT_REF_PTR,
            Self::RESTART_IP,
            Self::ADDRESS_RES,
            Self::IR_METHOD_ID_RES,
            Self::METHOD_ID_RES,
            Self::NEW_FRAME_SIZE_RES,
            Self::METHOD_SHAPE_ID,
            Self::NATIVE_RESTART_POINT,
            Self::NATIVE_RETURN_PTR,
            Self::METHOD_NUMBER,
            Self::JAVA_PC
        ])
    }
}


pub struct InvokeInterfaceResolve;

impl InvokeInterfaceResolve {
    pub const OBJECT_REF: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const JAVA_PC: Register = Register(4);
    pub const NATIVE_RETURN_PTR: Register = Register(5);
    pub const TARGET_METHOD_ID: Register = Register(8);
    pub const NATIVE_RESTART_POINT: Register = Register(9);
    pub const ADDRESS_RES: Register = Register(4);
    pub const NEW_FRAME_SIZE_RES: Register = Register(7);
    pub const IR_METHOD_ID_RES: Register = Register(5);
    pub const METHOD_ID_RES: Register = Register(6);
}

impl ExitRegisterStruct for InvokeInterfaceResolve {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([
            Register(0),
            Self::OBJECT_REF,
            Self::RESTART_IP,
            Self::ADDRESS_RES,
            Self::IR_METHOD_ID_RES,
            Self::METHOD_ID_RES,
            Self::NEW_FRAME_SIZE_RES,
            Self::TARGET_METHOD_ID,
            Self::NATIVE_RESTART_POINT,
            Self::NATIVE_RETURN_PTR,
            Self::JAVA_PC
        ])
    }
}

pub struct MonitorEnter;

impl MonitorEnter {
    pub const OBJ_ADDR: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const JAVA_PC: Register = Register(4);
}

impl ExitRegisterStruct for MonitorEnter {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), MonitorEnter::RESTART_IP, MonitorEnter::OBJ_ADDR, Self::JAVA_PC])
    }
}

pub struct MonitorExit;

impl MonitorExit {
    pub const OBJ_ADDR: Register = Register(2);
    pub const RESTART_IP: Register = Register(3);
    pub const JAVA_PC: Register = Register(4);
}

impl ExitRegisterStruct for MonitorExit {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), MonitorExit::RESTART_IP, MonitorExit::OBJ_ADDR, Self::JAVA_PC])
    }
}


pub struct NPE;

impl NPE {
    pub const JAVA_PC: Register = Register(4);
}

impl ExitRegisterStruct for NPE {
    fn all_registers() -> HashSet<Register> {
        HashSet::from([Register(0), Self::JAVA_PC])
    }
}