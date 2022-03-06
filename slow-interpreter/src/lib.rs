#![allow(dead_code)]
#![allow(unused)]
#![feature(c_variadic)]
#![feature(thread_local)]
#![feature(box_syntax)]
#![feature(vec_into_raw_parts)]
#![feature(core_intrinsics)]
#![feature(in_band_lifetimes)]
#![feature(thread_id_value)]
#![feature(unboxed_closures)]
#![feature(exclusive_range_pattern)]
#![feature(step_trait)]
#![feature(generic_associated_types)]
#![feature(never_type)]
extern crate errno;
extern crate libc;
extern crate libloading;
extern crate lock_api;
extern crate nix;
extern crate parking_lot;
extern crate regex;
extern crate va_list;

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;

use wtf8::Wtf8Buf;

use classfile_view::view::{ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::{CompressedClassfileStringPool, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::class_loading::{check_initing_or_inited_class, check_loaded_class, check_loaded_class_force_loader};
use crate::interpreter::{run_function, WasException};
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java::NewAsObjectOrJavaValue;
use crate::java_values::{ArrayObject, JavaValue};
use crate::java_values::Object::Array;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValue;
use crate::stack_entry::{StackEntry, StackEntryPush};
use crate::sun::misc::launcher::Launcher;
use crate::threading::JavaThread;

#[macro_use]
pub mod java_values;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
#[macro_use]
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
#[macro_use]
pub mod runtime_class;
pub mod jit;
pub mod jit_common;
pub mod native_to_ir_layer;
pub mod ir_to_java_layer;
pub mod native_tracing;
pub mod opaque_ids;
pub mod inheritance_method_ids;
pub mod inheritance_vtable;
pub mod static_breakpoints;
pub mod new_java_values;
pub mod known_type_to_address_mappings;

pub fn run_main(args: Vec<String>, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), Box<dyn Error>> {
    let launcher = Launcher::get_launcher(jvm, int_state).expect("todo");
    let loader_obj = launcher.get_loader(jvm, int_state).expect("todo");
    let main_loader = loader_obj.to_jvm_loader(jvm);

    let main = check_loaded_class_force_loader(jvm, int_state, &jvm.config.main_class_name.clone().into(), main_loader).expect("failed to load main class");
    let main = check_initing_or_inited_class(jvm, int_state, main.cpdtype()).expect("failed to load main class");
    check_loaded_class(jvm, int_state, main.cpdtype()).expect("failed to init main class");
    let main_view = main.view();
    let main_i = locate_main_method(&jvm.string_pool, &main_view);
    let main_thread = jvm.thread_state.get_main_thread();
    assert!(Arc::ptr_eq(&jvm.thread_state.get_current_thread(), &main_thread));
    let num_vars = main_view.method_view_i(main_i as u16).code_attribute().unwrap().max_locals;
    let stack_entry = StackEntryPush::new_java_frame(jvm, main.clone(), main_i as u16, vec![todo!()/*JavaValue::Top*/; num_vars as usize]);
    let mut main_frame_guard = int_state.push_frame(stack_entry);

    setup_program_args(&jvm, int_state, args);
    jvm.include_name_field.store(true, Ordering::SeqCst);
    match run_function(&jvm, int_state, &mut main_frame_guard) {
        Ok(_) => {
            if !jvm.config.compiled_mode_active {
                int_state.pop_frame(jvm, main_frame_guard, false);
            }
            sleep(Duration::new(100, 0)); //todo need to wait for other threads or something
        }
        Err(WasException {}) => {
            int_state.debug_print_stack_trace(jvm);
            todo!()
        }
    }
    Result::Ok(())
}

fn setup_program_args(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, args: Vec<String>) {
    let mut arg_strings: Vec<JavaValue<'gc_life>> = vec![];
    for arg_str in args {
        arg_strings.push(JString::from_rust(jvm, int_state, Wtf8Buf::from_string(arg_str)).expect("todo").new_java_value_handle().to_jv());
    }
    let arg_array = NewJavaValue::AllocObject(todo!()/*jvm.allocate_object(todo!()/*Array(ArrayObject::new_array(jvm, int_state, arg_strings, CPDType::Ref(CPRefType::Class(CClassName::string())), jvm.thread_state.new_monitor("arg array monitor".to_string())).expect("todo"))*/)*/);
    let mut current_frame_mut = int_state.current_frame_mut();
    let mut local_vars = current_frame_mut.local_vars_mut();
    local_vars.set(0, arg_array);
}

fn set_properties(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<(), WasException> {
    let frame_for_properties = int_state.push_frame(StackEntryPush::new_completely_opaque_frame(jvm, int_state.current_loader(jvm), vec![], "properties setting frame"));
    let properties = &jvm.properties;
    let prop_obj = System::props(jvm, int_state);
    assert_eq!(properties.len() % 2, 0);
    for i in 0..properties.len() / 2 {
        let key_i = 2 * i;
        let value_i = 2 * i + 1;
        let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(properties[key_i].clone())).expect("todo");
        let value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(properties[value_i].clone())).expect("todo");
        prop_obj.set_property(jvm, int_state, key, value)?;
    }
    int_state.pop_frame(jvm, frame_for_properties, false);
    Ok(())
}

fn locate_main_method(pool: &CompressedClassfileStringPool, main: &Arc<dyn ClassView>) -> u16 {
    let string_name = CClassName::string();
    let string_class = CPDType::Ref(CPRefType::Class(string_name));
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
