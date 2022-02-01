use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use crate::ir_to_java_layer::compiler::array_into_iter;

pub fn athrow() -> impl Iterator<Item=IRInstr> {
    array_into_iter([IRInstr::VMExit2 { exit_type: IRVMExitType::Throw {} }])
}
