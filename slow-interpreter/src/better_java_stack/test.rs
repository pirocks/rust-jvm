use std::env;
use std::mem::transmute;
use std::path::PathBuf;

use crossbeam::thread::Scope;

use another_jit_vm_ir::ir_stack::OwnedIRStack;
use gc_memory_layout_common::early_startup::get_regions;
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;
use xtask::{load_xtask_config, XTaskConfig};

use crate::better_java_stack::{FramePointer, JavaStack};
use crate::java_values::GC;
use crate::JVMState;
use crate::loading::Classpath;
use crate::options::JVMOptions;

pub fn with_jvm<'gc>(xtask: &XTaskConfig, func: impl FnOnce(&'gc JVMState<'gc>)) {
    let gc: GC<'gc> = GC::new(get_regions());
    let string_pool = CompressedClassfileStringPool::new();
    crossbeam::scope(|scope: Scope<'gc>| {
        let gc_ref: &'gc GC = unsafe { transmute(&gc) };//todo why do I need this?
        let mut jvm_options = JVMOptions::test_options();
        jvm_options.classpath = Classpath::from_dirs(vec![xtask.classes().into_boxed_path()]);
        let (args, jvm) = JVMState::new(jvm_options, scope, gc_ref, string_pool);
        unsafe { func(transmute(&jvm)); }
    })
        .expect("idk why this would happen");
}

fn this_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set?"))
}

fn workspace_dir() -> PathBuf {
    this_dir().parent().unwrap().to_path_buf()
}

#[test]
pub fn test() {
    let workspace_dir: PathBuf = workspace_dir();
    let xtask = load_xtask_config(&workspace_dir).unwrap().expect("No xtask config found.");
    with_jvm(&xtask,|jvm| {
        let mut new_stack = JavaStack::new(jvm, OwnedIRStack::new().unwrap());
        new_stack.assert_interpreter_frame_operand_stack_depths_sorted();
        assert!(new_stack.throw.is_none());
        let frame_pointer = FramePointer(new_stack.owned_ir_stack.native.mmaped_top);
        let interpreter_frame = new_stack.new_interpreter_frame(frame_pointer);
    })
}

