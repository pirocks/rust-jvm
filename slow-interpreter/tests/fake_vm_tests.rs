use rust_jvm_common::classfile::Classfile;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::options::JVMOptions;

pub fn fake_main_class() -> Classfile{
    Classfile{
        magic: 0,
        minor_version: 0,
        major_version: 0,
        constant_pool: vec![],
        access_flags: 0,
        this_class: 0,
        super_class: 0,
        interfaces: vec![],
        fields: vec![],
        methods: vec![],
        attributes: vec![]
    }
}

pub fn create_vm<'gc_life>() -> JVMState<'gc_life> {
    // JVMState::new(JVMOptions::new())
    //todo
    todo!()
}