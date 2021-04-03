#![feature(c_variadic)]
#![feature(thread_local)]
#![feature(box_syntax)]
#![feature(vec_into_raw_parts)]
#![feature(core_intrinsics)]
#![feature(entry_insert)]
#![feature(assoc_char_funcs)]
#![feature(duration_zero)]
#![feature(try_trait)]
extern crate errno;
extern crate futures_intrusive;
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

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType;

use crate::class_loading::{check_loaded_class, check_loaded_class_force_loader};
use crate::interpreter::{run_function, WasException};
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java_values::{ArrayObject, JavaValue};
use crate::java_values::Object::Array;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntry;
use crate::sun::misc::launcher::Launcher;
use crate::threading::JavaThread;

#[macro_use]
pub mod java_values;
#[macro_use]
pub mod runtime_class;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
#[macro_use]
pub mod utils;
pub mod interpreter_state;
pub mod options;
pub mod jvm_state;
pub mod instructions;
pub mod interpreter_util;
pub mod rust_jni;
pub mod loading;
pub mod jvmti;
pub mod invoke_interface;
pub mod stack_entry;
pub mod class_objects;
pub mod tracing;
pub mod interpreter;
pub mod method_table;
pub mod field_table;
pub mod native_allocation;
pub mod threading;
mod resolvers;
pub mod class_loading;

pub fn run_main(args: Vec<String>, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), Box<dyn Error>> {
    let launcher = Launcher::get_launcher(jvm, int_state).expect("todo");
    let loader_obj = launcher.get_loader(jvm, int_state).expect("todo");
    let main_loader = loader_obj.to_jvm_loader(jvm);

    let main = check_loaded_class_force_loader(jvm, int_state, &jvm.main_class_name.clone().into(), main_loader).expect("failed to load main class");
    check_loaded_class(jvm, int_state, main.ptypeview()).expect("failed to init main class");
    let main_view = main.view();
    let main_i = locate_main_method(&main_view);
    let main_thread = jvm.thread_state.get_main_thread();
    assert!(Arc::ptr_eq(&jvm.thread_state.get_current_thread(), &main_thread));
    let num_vars = main_view.method_view_i(main_i).code_attribute().unwrap().max_locals;
    let stack_entry = StackEntry::new_java_frame(jvm, main.clone(), main_i as u16, vec![JavaValue::Top; num_vars as usize]);
    let main_frame_guard = int_state.push_frame(stack_entry);

    setup_program_args(&jvm, int_state, args);
    jvm.include_name_field.store(true, Ordering::SeqCst);
    match run_function(&jvm, int_state) {
        Ok(_) => {
            int_state.pop_frame(jvm, main_frame_guard, false);
            sleep(Duration::new(100, 0));//todo need to wait for other threads or something
        }
        Err(WasException {}) => {
            int_state.debug_print_stack_trace();
            todo!()
        }
    }
    Result::Ok(())
}


fn setup_program_args(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: Vec<String>) {
    let mut arg_strings: Vec<JavaValue> = vec![];
    for arg_str in args {
        arg_strings.push(JString::from_rust(jvm, int_state, arg_str.clone()).expect("todo").java_value());
    }
    let arg_array = JavaValue::Object(Some(Arc::new(Array(ArrayObject::new_array(
        jvm,
        int_state,
        arg_strings,
        PTypeView::Ref(ReferenceTypeView::Class(ClassName::string())),
        jvm.thread_state.new_monitor("arg array monitor".to_string()),
    ).expect("todo")))));
    let local_vars = int_state.current_frame_mut().local_vars_mut();
    local_vars[0] = arg_array;
}


fn set_properties(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
    let frame_for_properties = int_state.push_frame(StackEntry::new_completely_opaque_frame(int_state.current_loader()));
    let properties = &jvm.properties;
    let prop_obj = System::props(jvm, int_state);
    assert_eq!(properties.len() % 2, 0);
    for i in 0..properties.len() / 2 {
        let key_i = 2 * i;
        let value_i = 2 * i + 1;
        let key = JString::from_rust(jvm, int_state, properties[key_i].clone()).expect("todo");
        let value = JString::from_rust(jvm, int_state, properties[value_i].clone()).expect("todo");
        prop_obj.set_property(jvm, int_state, key, value)?;
    }
    int_state.pop_frame(jvm, frame_for_properties, false);
    Ok(())
}


fn locate_main_method(main: &Arc<dyn ClassView>) -> usize {
    let string_name = ClassName::string();
    let string_class = PTypeView::Ref(ReferenceTypeView::Class(string_name));
    let string_array = PTypeView::Ref(ReferenceTypeView::Array(string_class.into()));
    let psvms = main.lookup_method_name(&"main".to_string());
    for m in psvms {
        let desc = m.desc();
        if m.is_static() && desc.parameter_types == vec![string_array.to_ptype()] && desc.return_type == PType::VoidType {
            return m.method_i();
        }
    }
    //todo validate that main class isn't an array class
    panic!("No psvms found in class: {}", main.name().unwrap_name().get_referred_name());
}

