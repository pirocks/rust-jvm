#![allow(unused_unsafe)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![feature(c_variadic)]
#![feature(thread_local)]
#![feature(box_syntax)]
#![feature(vec_into_raw_parts)]
#![feature(core_intrinsics)]
#![feature(thread_id_value)]
#![feature(unboxed_closures)]
#![feature(exclusive_range_pattern)]
#![feature(step_trait)]
#![feature(never_type)]
#![feature(box_patterns)]
#![feature(once_cell)]
#![feature(is_sorted)]
#![feature(allocator_api)]
#![feature(print_internals)]
#![feature(fmt_internals)]

extern crate alloc;
extern crate core;

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use itertools::Itertools;
use wtf8::Wtf8Buf;

use classfile_view::view::{ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;


use stdlib::java::lang::string::JString;
use stdlib::java::lang::system::System;
use stdlib::java::NewAsObjectOrJavaValue;
use stdlib::sun::misc::launcher::Launcher;

use crate::better_java_stack::frames::PushableFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::class_loading::{check_initing_or_inited_class, check_loaded_class, check_loaded_class_force_loader};
use crate::exceptions::WasException;
use crate::interpreter::run_function;
use crate::java_values::JavaValue;
use crate::jit::MethodResolverImpl;
use crate::jvm_state::JVMState;
use crate::new_java_values::{NewJavaValue, NewJavaValueHandle};
use crate::new_java_values::allocated_objects::AllocatedHandle;
use crate::new_java_values::java_value_common::JavaValueCommon;
use crate::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray};
use crate::stack_entry::{StackEntry, StackEntryPush};
use crate::threading::thread_state::ThreadState;
use crate::utils::pushable_frame_todo;

pub mod function_call_targets_updating;
pub mod java_values;
pub mod utils;
pub mod class_loading;
pub mod class_objects;
pub mod field_table;
pub mod interpreter;
pub mod interpreter_state;
pub mod interpreter_util;
pub mod jvm_state;
pub mod loading;
pub mod method_table;
pub mod native_allocation;
pub mod options;
pub mod rust_jni;
pub mod stack_entry;
pub mod threading;
pub mod tracing;
pub mod runtime_class;
pub mod jit;
pub mod ir_to_java_layer;
pub mod new_java_values;
pub mod string_exit_cache;
pub mod function_instruction_count;
pub mod better_java_stack;
pub mod exceptions;
pub mod stdlib;
pub mod leaked_interface_arrays;
pub mod string_intern;
pub mod extra_intrinsics;
pub mod static_vars;
pub mod accessor_ext;

pub fn run_main<'gc, 'l>(args: Vec<String>, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), Box<dyn Error>> {
    let launcher = Launcher::get_launcher(jvm, int_state).expect("todo");
    let loader_obj = launcher.get_loader(jvm, int_state).expect("todo");
    let main_loader = loader_obj.to_jvm_loader(jvm);

    ThreadState::debug_assertions(jvm, int_state, loader_obj);

    let main = check_loaded_class_force_loader(jvm, int_state, &jvm.config.main_class_name.clone().into(), main_loader).expect("failed to load main class");
    let main = match check_initing_or_inited_class(jvm, int_state, main.cpdtype()) {
        Ok(main) => main,
        Err(WasException{exception_obj}) => {
            exception_obj.print_stack_trace(jvm, int_state).expect("exception printing exception");
            panic!("failed to load main class");
        },
    };
    check_loaded_class(jvm, int_state, main.cpdtype()).expect("failed to init main class");
    let main_view = main.view();
    let main_i = locate_main_method(&jvm.string_pool, &main_view);
    let main_thread = jvm.thread_state.get_main_thread();
    assert!(Arc::ptr_eq(&jvm.thread_state.get_current_thread(), &main_thread));
    let num_vars = main_view.method_view_i(main_i as u16).code_attribute().unwrap().max_locals;
    let main_method_id = jvm.method_table.write().unwrap().get_method_id(main.clone(), main_i);
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: main_loader }, main_method_id, false);
    let mut initial_local_var_array = vec![NewJavaValue::Top; num_vars as usize];
    let local_var_array = setup_program_args(&jvm, int_state, args);
    jvm.program_args_array.set(local_var_array.duplicate_discouraged()).unwrap();
    initial_local_var_array[0] = local_var_array.new_java_value();
    let java_frame_push = StackEntryPush::new_java_frame(jvm, main.clone(), main_i as u16, initial_local_var_array);
    let _: Result<(), WasException<'gc>> = int_state.push_frame_java(java_frame_push, |java_native| {
        jvm.include_name_field.store(true, Ordering::SeqCst);
        match run_function(&jvm, java_native) {
            Ok(_) => {
                if !jvm.config.compiled_mode_active {
                    todo!()// int_state.pop_frame(jvm, main_frame_guard, false);
                }

                return Ok(())
                // panic!();
            }
            Err(WasException { exception_obj }) => {
                //todo should be allowing catching in main
                let exception_string = exception_obj.to_string(jvm,java_native).unwrap().unwrap().to_rust_string(jvm);
                dbg!(exception_string);
                exception_obj.print_stack_trace(jvm, java_native).unwrap();
                dbg!("main exited with exception");
                // dbg!(throwable.to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
                // int_state.debug_print_stack_trace(jvm);
                todo!()
            }
        }
    });
    Ok(())
}

fn setup_program_args<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, args: Vec<String>) -> AllocatedHandle<'gc> {
    let mut arg_strings: Vec<NewJavaValueHandle<'gc>> = vec![];
    for arg_str in args {
        arg_strings.push(JString::from_rust(jvm, int_state, Wtf8Buf::from_string(arg_str)).expect("todo").new_java_value_handle());
    }
    let elems = arg_strings.iter().map(|handle| handle.as_njv()).collect_vec();
    jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray {
        whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(CClassName::string().into())).unwrap(),
        elems,
    }))
}

fn set_properties<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
    let loader = int_state.current_loader(jvm);
    let frame_to_push = StackEntryPush::new_completely_opaque_frame(jvm, loader, vec![], "properties setting frame");
    int_state.push_frame_opaque(frame_to_push, |opaque_frame| {
        let prop_obj = System::props(jvm, opaque_frame);
        for (key, value) in &jvm.properties {
            let key = JString::from_rust(jvm, opaque_frame, Wtf8Buf::from_string(key.to_string())).expect("todo");
            let value = JString::from_rust(jvm, opaque_frame, Wtf8Buf::from_string(value.to_string())).expect("todo");
            prop_obj.set_property(jvm, opaque_frame, key, value)?;
        }
        Ok(())
    })
}

fn locate_main_method(pool: &CompressedClassfileStringPool, main: &Arc<dyn ClassView>) -> u16 {
    let string_name = CClassName::string();
    let string_class = CPDType::Class(string_name);
    let string_array = CPDType::array(string_class);
    let psvms = main.lookup_method_name(MethodName(pool.add_name(&"main".to_string(), false)));
    for m in psvms {
        let desc = m.desc();
        if m.is_static() && desc.arg_types == vec![string_array.clone()] && desc.return_type == CPDType::VoidType {
            return m.method_i();
        }
    }
    //todo validate that main class isn't an array class
    let main_class_name = pool.lookup(main.name().unwrap_object_name().0);
    panic!("No psvms found in class: {}", main_class_name);
}
