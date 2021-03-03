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
use crate::instructions::invoke::native::system_temp::system_array_copy;
use crate::instructions::invoke::native::unsafe_temp::*;
use crate::interpreter::monitor_for_function;
use crate::java::nio::heap_byte_buffer::HeapByteBuffer;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::{call, call_impl, mangling};

pub fn run_native_method(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    class: Arc<RuntimeClass>,
    method_i: usize) {
    let view = &class.view();
    let before = int_state.current_frame().operand_stack().len();
    assert_inited_or_initing_class(jvm, int_state, view.name().into());
    assert_eq!(before, int_state.current_frame().operand_stack().len());
    let method = &view.method_view_i(method_i);
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

    let monitor = monitor_for_function(jvm, int_state, method, method.access_flags() & JVM_ACC_SYNCHRONIZED as u16 > 0, &class.view().name());
    if let Some(m) = monitor.as_ref() {
        m.lock(jvm)
    }

    let meth_name = method.name();
    let result = if meth_name == *"desiredAssertionStatus0" {//todo and descriptor matches and class matches
        JavaValue::Boolean(0).into()
    } else if meth_name == *"arraycopy" {
        system_array_copy(&mut args);
        None
    } else if jvm.libjava.registered_natives.read().unwrap().contains_key(&ByAddress(class.clone())) &&
        jvm.libjava.registered_natives.read().unwrap().get(&ByAddress(class.clone())).unwrap().read().unwrap().contains_key(&(method_i as u16))
    {
        //todo dup
        let res_fn = {
            let reg_natives = jvm.libjava.registered_natives.read().unwrap();
            let reg_natives_for_class = reg_natives.get(&ByAddress(class.clone())).unwrap().read().unwrap();
            *reg_natives_for_class.get(&(method_i as u16)).unwrap()
        };
        call_impl(jvm, int_state, class.clone(), args, parsed, &res_fn, !method.is_static())
    } else {
        match call(jvm, int_state, class.clone(), method_i, args.clone(), parsed) {
            Ok(r) => r,
            Err(_) => {
                let mangled = mangling::mangle(class.clone(), method_i);
                // state.tracing.trace_dynmaic_link()
                //todo actually impl these at some point
                if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives" {
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
                    let jclass = args[1].cast_class();
                    let ptype = jclass.as_runtime_class(jvm).ptypeview();
                    check_initing_or_inited_class(jvm, int_state, ptype).unwrap();
                    None
                } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
                    Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm, int_state, &mut args)
                } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
                    Java_java_lang_invoke_MethodHandleNatives_getMembers(&mut args)
                } else if &mangled == "Java_sun_misc_Unsafe_putObjectVolatile" {
                    unimplemented!()
                } else if &mangled == "Java_sun_misc_Perf_registerNatives" {
                    //todo not really sure what to do here, for now nothing
                    None
                } else if &mangled == "Java_sun_misc_Perf_createLong" {
                    Some(HeapByteBuffer::new(jvm, int_state, vec![0, 0, 0, 0, 0, 0, 0, 0], 0, 8).java_value())//todo this is incorrect and should be implemented properly.
                } else {
                    int_state.debug_print_stack_trace();
                    dbg!(mangled);
                    panic!()
                }
            }
        }
    }
        ;
    if let Some(m) = monitor.as_ref() { m.unlock(jvm) }
    let was_exception = int_state.throw().is_some();
    int_state.pop_frame(jvm, native_call_frame, was_exception);
    //todo need to check excpetion here
    match result {
        None => {}
        Some(res) => {
            int_state.push_current_operand_stack(res)
        }
    }
}


//todo needed?
pub fn call_signature_polymorphic(_jvm: &JVMState,
                                  _int_state: &mut InterpreterStateGuard,
                                  _method_view: &MethodView,
) {
    unimplemented!()
}

pub mod mhn_temp;
pub mod unsafe_temp;
pub mod system_temp;