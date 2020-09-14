use std::sync::Arc;
use std::sync::atomic::Ordering;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::ACC_SYNCHRONIZED;
use rust_jvm_common::classfile::{Class, Classfile, ConstantInfo, ConstantKind, Utf8};
use rust_jvm_common::classnames::ClassName;
use verification::{VerifierContext, verify};

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::instructions::invoke::native::mhn_temp::*;
use crate::instructions::invoke::native::mhn_temp::init::MHN_init;
use crate::instructions::invoke::native::mhn_temp::resolve::MHN_resolve;
use crate::instructions::invoke::native::system_temp::system_array_copy;
use crate::instructions::invoke::native::unsafe_temp::*;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter::monitor_for_function;
use crate::interpreter_util::check_inited_class;
use crate::java::lang::reflect::field::Field;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, Object};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::{call, call_impl, mangling};
use crate::sun::misc::unsafe_::Unsafe;

pub fn run_native_method<'l>(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    class: Arc<RuntimeClass>,
    method_i: usize,
    _debug: bool,
) {
    let view = &class.view();
    let before = int_state.current_frame().operand_stack().len();
    check_inited_class(jvm, int_state, &view.name().into(), class.loader(jvm));
    assert_eq!(before, int_state.current_frame().operand_stack().len());
    let method = &view.method_view_i(method_i);
    if !method.is_static() {
        assert_ne!(before, 0);
    }
    assert!(method.is_native());
    let parsed = method.desc();
    let mut args = vec![];
    //todo should have some setup args functions
    // dbg!(int_state.current_frame().operand_stack());
    // dbg!(method.name());
    if method.is_static() {
        for _ in &parsed.parameter_types {
            args.push(int_state.pop_current_operand_stack());
        }
        args.reverse();
    } else if method.is_native() {
        for _ in &parsed.parameter_types {
            args.push(int_state.pop_current_operand_stack());
        }
        args.reverse();
        args.insert(0, int_state.pop_current_operand_stack());
    } else {
        panic!();
    }
    let native_call_frame = int_state.push_frame(StackEntry::new_native_frame(class.clone(), method_i as u16, args.clone()));
    assert!(int_state.current_frame_mut().is_native());

    let monitor = monitor_for_function(jvm, int_state, method, method.access_flags() & ACC_SYNCHRONIZED as u16 > 0, &class.view().name());
    monitor.as_ref().map(|m| m.lock(jvm));
    if _debug {
        // dbg!(&args);
        // dbg!(&frame.operand_stack);
    }
    // println!("CALL BEGIN NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
    let meth_name = method.name();
    let debug = false;//meth_name.contains("isAlive");
    let result = if meth_name == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        JavaValue::Boolean(0).into()
    } else if meth_name == "arraycopy".to_string() {
        system_array_copy(&mut args);
        None
    } else {
        if jvm.libjava.registered_natives.read().unwrap().contains_key(&class) &&
            jvm.libjava.registered_natives.read().unwrap().get(&class).unwrap().read().unwrap().contains_key(&(method_i as u16))
        {
            //todo dup
            let res_fn = {
                let reg_natives = jvm.libjava.registered_natives.read().unwrap();
                let reg_natives_for_class = reg_natives.get(&class).unwrap().read().unwrap();
                reg_natives_for_class.get(&(method_i as u16)).unwrap().clone()
            };
            call_impl(jvm, int_state, class.clone(), args, parsed, &res_fn, !method.is_static(), debug)
        } else {
            let res = match call(jvm, int_state, class.clone(), method_i, args.clone(), parsed) {
                Ok(r) => r,
                Err(_) => {
                    let mangled = mangling::mangle(class.clone(), method_i);
                    // state.tracing.trace_dynmaic_link()
                    //todo actually impl these at some point
                    if mangled == "Java_sun_misc_Unsafe_allocateMemory".to_string() {
                        allocate_memory(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_putLong__JJ".to_string() {
                        putLong__JJ(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_getByte__J".to_string() {
                        getByte__J(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_freeMemory".to_string() {
                        freeMemory(&mut args)
                        //todo all these unsafe function thingys are getting a tad excessive
                    } else if mangled == "Java_sun_misc_Unsafe_getObjectVolatile".to_string() {
                        get_object_volatile(jvm, &mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives" {
                        //todo
                        None
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getConstant" {
                        MHN_getConstant()
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_resolve" {
                        MHN_resolve(jvm, int_state, &mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_init" {
                        MHN_init(jvm, int_state, &mut args)
                    } else if &mangled == "Java_sun_misc_Unsafe_shouldBeInitialized" {
                        //todo this isn't totally correct b/c there's a distinction between initialized and initializing.
                        shouldBeInitialized(jvm, &mut args)
                    } else if &mangled == "Java_sun_misc_Unsafe_ensureClassInitialized" {
                        if shouldBeInitialized(jvm, &mut args).unwrap().unwrap_int() != 1 {
                            panic!()
                        }
                        None
                    } else if &mangled == "Java_sun_misc_Unsafe_defineAnonymousClass" {
                        Java_sun_misc_Unsafe_defineAnonymousClass(jvm, int_state, &mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
                        Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm, int_state, &mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
                        Java_java_lang_invoke_MethodHandleNatives_getMembers(&mut args)
                    } else {
                        int_state.print_stack_trace();
                        dbg!(mangled);
                        panic!()
                    }
                }
            };
            res
        }
    };
    monitor.as_ref().map(|m| m.unlock(jvm));
    int_state.pop_frame(native_call_frame);
    match result {
        None => {}
        Some(res) => {
            int_state.push_current_operand_stack(res)
        }
    }
}


pub mod mhn_temp;
pub mod unsafe_temp;
pub mod system_temp;