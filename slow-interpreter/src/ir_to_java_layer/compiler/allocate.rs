use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::ir_to_java_layer::compiler::array_into_iter;
use crate::ir_to_java_layer::vm_exit_abi::IRVMExitType;
use crate::jit::ir::IRInstr;
use crate::jit::MethodResolver;

pub fn new(resolver: &MethodResolver<'vm_life>, ccn: CClassName) -> impl Iterator<Item=IRInstr>{
    match resolver.lookup_type_loaded(&(ccn).into()) {
        None => {
            array_into_iter([IRInstr::VMExit2 {
                exit_type: IRVMExitType::LoadClassAndRecompile { class: todo!() },
            }])
        }
        Some((loaded_class, loader)) => {
            todo!()
        }
    }
}
