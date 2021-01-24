#![feature(c_variadic)]
#![feature(thread_local)]
#![feature(box_syntax)]
#![feature(vec_into_raw_parts)]
#![feature(unsafe_cell_get_mut)]
#![feature(core_intrinsics)]
#![feature(entry_insert)]
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

use classfile_view::loading::LoaderName;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType;

use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter::run_function;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java_values::{ArrayObject, JavaValue};
use crate::java_values::Object::Array;
use crate::jvm_state::JVMState;
use crate::runtime_class::RuntimeClass;
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
    // if jvm.unittest_mode {
    //     run_tests(jvm, int_state);
    //     Result::Ok(())
    // } else {
    dbg!(&jvm.main_class_name);

    let launcher = Launcher::get_launcher(jvm, int_state);
    let loader_obj = launcher.get_loader(jvm, int_state);
    let main_loader = loader_obj.to_jvm_loader(jvm);
    dbg!(loader_obj.to_string(jvm, int_state).to_rust_string());
    dbg!(main_loader);
    let main = assert_inited_or_initing_class(jvm, int_state, jvm.main_class_name.clone().into());
    let main_view = main.view();
    let main_i = locate_main_method(&main_view.backing_class());
    let main_thread = jvm.thread_state.get_main_thread();
    assert!(Arc::ptr_eq(&jvm.thread_state.get_current_thread(), &main_thread));
    let num_vars = main_view.method_view_i(main_i).code_attribute().unwrap().max_locals;
    let stack_entry = StackEntry::new_java_frame(main.clone(), main_i as u16, vec![JavaValue::Top; num_vars as usize]);
    let main_frame_guard = int_state.push_frame(stack_entry);

    dbg!(int_state.current_loader());
    setup_program_args(&jvm, int_state, args);
    assert_ne!(int_state.current_loader(), LoaderName::BootstrapLoader);
    run_function(&jvm, int_state);
    if int_state.throw().is_some() || *int_state.terminate() {
        int_state.print_stack_trace();
        unimplemented!()
    }
    int_state.pop_frame(main_frame_guard);
    Result::Ok(())
    // }
}


fn setup_program_args(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: Vec<String>) {
    let mut arg_strings: Vec<JavaValue> = vec![];
    for arg_str in args {
        arg_strings.push(JString::from_rust(jvm, int_state, arg_str.clone()).java_value());
    }
    let arg_array = JavaValue::Object(Some(Arc::new(Array(ArrayObject::new_array(
        jvm,
        int_state,
        arg_strings,
        PTypeView::Ref(ReferenceTypeView::Class(ClassName::string())),
        jvm.thread_state.new_monitor("arg array monitor".to_string()),
        int_state.current_loader()
    )))));
    let local_vars = int_state.current_frame_mut().local_vars_mut();
    local_vars[0] = arg_array;
}


fn set_properties(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let frame_for_properties = int_state.push_frame(StackEntry::new_completely_opaque_frame(int_state.current_loader()));
    let properties = &jvm.properties;
    let prop_obj = System::props(jvm, int_state);
    assert_eq!(properties.len() % 2, 0);
    for i in 0..properties.len() / 2 {
        let key_i = 2 * i;
        let value_i = 2 * i + 1;
        let key = JString::from_rust(jvm, int_state, properties[key_i].clone());
        let value = JString::from_rust(jvm, int_state, properties[value_i].clone());
        prop_obj.set_property(jvm, int_state, key, value);
    }
    int_state.pop_frame(frame_for_properties);
}


fn locate_init_system_class(system: &Arc<RuntimeClass>) -> MethodView {
    let system_class = system.view();
    let method_views = system_class.lookup_method_name(&"initializeSystemClass".to_string());
    method_views.first().unwrap().clone()
}

fn locate_main_method(main: &Arc<Classfile>) -> usize {
    let string_name = ClassName::string();
    let string_class = PTypeView::Ref(ReferenceTypeView::Class(string_name));
    let string_array = PTypeView::Ref(ReferenceTypeView::Array(string_class.into()));
    let psvms = main.lookup_method_name(&"main".to_string());
    for (i, m) in psvms {
        let desc = MethodDescriptor::from_legacy(m, main);
        if m.is_static() && desc.parameter_types == vec![string_array.to_ptype()] && desc.return_type == PType::VoidType {
            return i;
        }
    }
    panic!();
}

