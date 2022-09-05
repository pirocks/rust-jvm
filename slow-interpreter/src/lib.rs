#![allow(dead_code)]
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
#![feature(generic_associated_types)]
#![feature(never_type)]
#![feature(box_patterns)]
#![feature(once_cell)]
#![feature(is_sorted)]
#![feature(allocator_api)]


use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;

use itertools::Itertools;
use wtf8::Wtf8Buf;


use classfile_view::view::{ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::{CompressedClassfileStringPool, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use crate::better_java_stack::frames::PushableFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;

use crate::class_loading::{check_initing_or_inited_class, check_loaded_class, check_loaded_class_force_loader};
use crate::exceptions::WasException;
use crate::interpreter::{run_function};
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java::NewAsObjectOrJavaValue;
use crate::java_values::JavaValue;
use crate::jit::MethodResolverImpl;
use crate::jvm_state::JVMState;
use crate::new_java_values::{NewJavaValue, NewJavaValueHandle};
use crate::new_java_values::allocated_objects::AllocatedHandle;
use crate::new_java_values::java_value_common::JavaValueCommon;
use crate::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray};
use crate::stack_entry::{StackEntry, StackEntryPush};
use crate::sun::misc::launcher::Launcher;
use crate::threading::{JavaThread, ThreadState};
use crate::utils::pushable_frame_todo;

pub mod function_call_targets_updating;
pub mod java_values;
pub mod java;
pub mod sun;
pub mod utils;
pub mod class_loading;
pub mod class_objects;
pub mod field_table;
pub mod instructions;
pub mod interpreter;
pub mod interpreter_state;
pub mod interpreter_util;
pub mod invoke_interface;
pub mod jvm_state;
pub mod jvmti;
pub mod loading;
pub mod method_table;
pub mod native_allocation;
pub mod options;
mod resolvers;
pub mod rust_jni;
pub mod stack_entry;
pub mod threading;
pub mod tracing;
pub mod runtime_class;
pub mod jit;
pub mod jit_common;
pub mod native_to_ir_layer;
pub mod ir_to_java_layer;
pub mod new_java_values;
pub mod known_type_to_address_mappings;
pub mod string_exit_cache;
pub mod function_instruction_count;
pub mod better_java_stack;
pub mod exceptions;

pub fn run_main<'gc, 'l>(args: Vec<String>, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), Box<dyn Error>> {
    let launcher = Launcher::get_launcher(jvm, int_state).expect("todo");
    let loader_obj = launcher.get_loader(jvm, int_state).expect("todo");
    let main_loader = loader_obj.to_jvm_loader(jvm);

    ThreadState::debug_assertions(jvm,int_state, loader_obj);

    let main = check_loaded_class_force_loader(jvm, int_state, &jvm.config.main_class_name.clone().into(), main_loader).expect("failed to load main class");
    let main = check_initing_or_inited_class(jvm, int_state, main.cpdtype()).expect("failed to load main class");
    check_loaded_class(jvm, int_state, main.cpdtype()).expect("failed to init main class");
    let main_view = main.view();
    let main_i = locate_main_method(&jvm.string_pool, &main_view);
    let main_thread = jvm.thread_state.get_main_thread();
    assert!(Arc::ptr_eq(&jvm.thread_state.get_current_thread(), &main_thread));
    let num_vars = main_view.method_view_i(main_i as u16).code_attribute().unwrap().max_locals;
    let main_method_id = jvm.method_table.write().unwrap().get_method_id(main.clone(), main_i);
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: main_loader }, main_method_id,false);
    let mut initial_local_var_array = vec![NewJavaValue::Top; num_vars as usize];
    let local_var_array = setup_program_args(&jvm, todo!()/*int_state*/, args);
    jvm.local_var_array.set(local_var_array.duplicate_discouraged()).unwrap();
    initial_local_var_array[0] = local_var_array.new_java_value();
    let java_frame_push = StackEntryPush::new_java_frame(jvm, main.clone(), main_i as u16, initial_local_var_array);
    let _: Result<(), WasException<'gc>> = int_state.push_frame_java(java_frame_push, |java_native|{
        jvm.include_name_field.store(true, Ordering::SeqCst);
        match run_function(&jvm, java_native) {
            Ok(_) => {
                if !jvm.config.compiled_mode_active {
                    todo!()// int_state.pop_frame(jvm, main_frame_guard, false);
                }
                loop {
                    sleep(Duration::new(100, 0)); //todo need to wait for other threads or something
                }
                // panic!();
            }
            Err(WasException { exception_obj }) => {
                todo!();// let throwable = int_state.throw().unwrap().duplicate_discouraged().cast_throwable();
                // int_state.set_throw(None);
                // throwable.print_stack_trace(jvm, int_state).unwrap();
                // dbg!(throwable.to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
                // int_state.debug_print_stack_trace(jvm);
                todo!()
            }
        }
    });
    Ok(())
}

fn setup_program_args<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, '_>, args: Vec<String>) -> AllocatedHandle<'gc> {
    let mut arg_strings: Vec<NewJavaValueHandle<'gc>> = vec![];
    for arg_str in args {
        arg_strings.push(JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string(arg_str)).expect("todo").new_java_value_handle());
    }
    let elems = arg_strings.iter().map(|handle| handle.as_njv()).collect_vec();
    let mut temp : OpaqueFrame<'gc, '_> = todo!();
    jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray {
        whole_array_runtime_class: check_initing_or_inited_class(jvm, pushable_frame_todo()/*int_state*/, CPDType::array(CClassName::string().into())).unwrap(),
        elems,
    }))
}

fn set_properties<'gc>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, '_>) -> Result<(), WasException<'gc>> {
    let frame_for_properties = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm, int_state.current_loader(jvm), vec![], "properties setting frame")*/);
    let properties = &jvm.properties;
    let prop_obj = System::props(jvm, int_state);
    assert_eq!(properties.len() % 2, 0);
    for i in 0..properties.len() / 2 {
        let key_i = 2 * i;
        let value_i = 2 * i + 1;
        let key = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string(properties[key_i].clone())).expect("todo");
        let value = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string(properties[value_i].clone())).expect("todo");
        prop_obj.set_property(jvm, pushable_frame_todo()/*int_state*/, key, value)?;
    }
    int_state.pop_frame(jvm, frame_for_properties, false);
    Ok(())
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
