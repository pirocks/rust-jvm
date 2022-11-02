use std::env;
use std::mem::transmute;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::thread::Scope;

use another_jit_vm_ir::ir_stack::OwnedIRStack;
use gc_memory_layout_common::early_startup::get_regions;
use jvmti_jni_bindings::jobject;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use thread_signal_handler::SignalAccessibleJavaStackData;
use xtask::{load_xtask_config, XTaskConfig};


use crate::{JVMState, StackEntryPush};
use crate::better_java_stack::{JavaStack, JavaStackGuard};
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::java_values::GC;
use crate::loading::Classpath;
use crate::options::JVMOptions;
use crate::stack_entry::JavaFramePush;

pub fn with_jvm(xtask: &XTaskConfig, func: impl for<'gc> FnOnce(&'gc JVMState<'gc>)) {
    let gc: GC = GC::new(get_regions());
    let string_pool = CompressedClassfileStringPool::new();
    std::thread::scope(|scope: &Scope| {
        within(scope, xtask, &gc, string_pool, func)
    });
}

pub fn within<'gc>(scope: &'_ Scope<'_, 'gc>, xtask: &XTaskConfig, gc: &GC, string_pool: CompressedClassfileStringPool, func: impl for<'other> FnOnce(&'other JVMState<'other>)) {
    let gc_ref: &'gc GC<'gc> = unsafe { transmute(&gc) };//todo why do I need this?
    let scope_ref: &'gc Scope<'gc, 'gc> = unsafe { transmute(scope) };
    let mut jvm_options = JVMOptions::test_options();
    jvm_options.classpath = Classpath::from_dirs(vec![xtask.classes().into_boxed_path()]);
    let (args, jvm): (Vec<String>, JVMState<'gc>) = JVMState::new(jvm_options, scope_ref, gc_ref, string_pool);
    unsafe {
        let jvm: &'gc JVMState<'gc> = transmute(&jvm);
        jvm.java_vm_state.init(jvm);
        func(jvm);
    }
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
    unsafe {
        with_jvm(&xtask, |jvm| {
            let new_stack = Mutex::new(JavaStack::new(OwnedIRStack::new().unwrap(), Arc::new(SignalAccessibleJavaStackData::new())));
            JavaStackGuard::new_from_empty_stack(jvm, transmute(&new_stack), |opaque_frame| {
                opaque_frame.debug_assert();
                opaque_frame.push_frame(StackEntryPush::Java(JavaFramePush {
                    method_id: 0,
                    local_vars: vec![],
                    operand_stack: vec![],
                }), |new_guard| {
                    let res: jobject = null_mut();
                    Ok(res)
                }).unwrap();
                Ok(())
            }).unwrap();
        })
    }
}

