use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::JVM_ACC_SYNCHRONIZED;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::instructions::invoke::native::mhn_temp::*;
use crate::instructions::invoke::native::mhn_temp::init::MHN_init;
use crate::instructions::invoke::native::mhn_temp::resolve::MHN_resolve;
use crate::instructions::invoke::native::unsafe_temp::*;
use crate::interpreter::{monitor_for_function, WasException};
use crate::java::nio::heap_byte_buffer::HeapByteBuffer;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::{call, call_impl, mangling};
use crate::utils::throw_npe_res;

pub fn run_native_method(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    class: Arc<RuntimeClass>,
    method_i: usize) -> Result<(), WasException> {
    let view = &class.view();
    let before = int_state.current_frame().operand_stack().len();
    assert_inited_or_initing_class(jvm, int_state, view.type_());
    assert_eq!(before, int_state.current_frame().operand_stack().len());
    let method = view.method_view_i(method_i);
    if !method.is_static() {
        assert_ne!(before, 0);
    }
    assert!(method.is_native());
    let parsed = method.desc();
    let mut args = vec![];
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
    let native_call_frame = int_state.push_frame(StackEntry::new_native_frame(jvm, class.clone(), method_i as u16, args.clone()));
    assert!(int_state.current_frame_mut().is_native());

    let monitor = monitor_for_function(jvm, int_state, &method, method.access_flags() & JVM_ACC_SYNCHRONIZED as u16 > 0);
    if let Some(m) = monitor.as_ref() {
        m.lock(jvm)
    }

    let result = if jvm.libjava.registered_natives.read().unwrap().contains_key(&ByAddress(class.clone())) &&
        jvm.libjava.registered_natives.read().unwrap().get(&ByAddress(class.clone())).unwrap().read().unwrap().contains_key(&(method_i as u16))
    {
        //todo dup
        let res_fn = {
            let reg_natives = jvm.libjava.registered_natives.read().unwrap();
            let reg_natives_for_class = reg_natives.get(&ByAddress(class.clone())).unwrap().read().unwrap();
            *reg_natives_for_class.get(&(method_i as u16)).unwrap()
        };
        match call_impl(jvm, int_state, class.clone(), args, parsed, &res_fn, !method.is_static()) {
            Ok(call_res) => call_res,
            Err(WasException {}) => {
                int_state.pop_frame(jvm, native_call_frame, true);
                return Err(WasException);
            }
        }
    } else {
        match match call(jvm, int_state, class.clone(), method.clone(), args.clone(), parsed) {
            Ok(call_res) => call_res,
            Err(WasException {}) => {
                int_state.pop_frame(jvm, native_call_frame, true);
                return Err(WasException);
            }
        } {
            Ok(r) => r,
            Err(_) => {
                match special_call_overrides(jvm, int_state, &class.view().method_view_i(method_i), &mut args) {
                    Ok(res) => res,
                    Err(_) => None
                }
            }
        }
    };
    if let Some(m) = monitor.as_ref() { m.unlock(jvm) }
    let was_exception = int_state.throw().is_some();
    int_state.pop_frame(jvm, native_call_frame, was_exception);
    if was_exception {
        Err(WasException)
    } else {
        if let Some(res) = result {
            int_state.push_current_operand_stack(res);
        }
        Ok(())
    }
}

fn special_call_overrides(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView, mut args: &mut Vec<JavaValue>) -> Result<Option<JavaValue>, WasException> {
    let mangled = mangling::mangle(method_view);
    //todo actually impl these at some point
    Ok(if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives" {
        //todo
        None
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getConstant" {
        MHN_getConstant()?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_resolve" {
        MHN_resolve(jvm, int_state, &mut args)?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_init" {
        MHN_init(jvm, int_state, &mut args)?;
        None
    } else if &mangled == "Java_sun_misc_Unsafe_shouldBeInitialized" {
        //todo this isn't totally correct b/c there's a distinction between initialized and initializing.
        shouldBeInitialized(jvm, &mut args)?.into()
    } else if &mangled == "Java_sun_misc_Unsafe_ensureClassInitialized" {
        let jclass = match args[1].cast_class() {
            None => {
                throw_npe_res(jvm, int_state)?;
                unreachable!()
            }
            Some(class) => class
        };
        let ptype = jclass.as_runtime_class(jvm).ptypeview();
        check_initing_or_inited_class(jvm, int_state, ptype)?;
        None
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
        Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm, int_state, &mut args)?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
        Java_java_lang_invoke_MethodHandleNatives_getMembers(jvm, int_state, &mut args)?.into()
    } else if &mangled == "Java_sun_misc_Unsafe_putObjectVolatile" {
        unimplemented!()
    } else if &mangled == "Java_sun_misc_Perf_registerNatives" {
        //todo not really sure what to do here, for now nothing
        None
    } else if &mangled == "Java_sun_misc_Perf_createLong" {
        Some(HeapByteBuffer::new(jvm, int_state, vec![0, 0, 0, 0, 0, 0, 0, 0], 0, 8)?.java_value())//todo this is incorrect and should be implemented properly.
    } else {
        int_state.debug_print_stack_trace();
        dbg!(mangled);
        panic!()
    })
}


pub mod mhn_temp;
pub mod unsafe_temp;
