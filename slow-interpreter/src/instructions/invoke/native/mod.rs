use std::collections::HashSet;
use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::JVM_ACC_SYNCHRONIZED;
use rust_jvm_common::compressed_classfile::names::CClassName;

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

pub fn run_native_method(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, class: Arc<RuntimeClass<'gc_life>>, method_i: u16, args: Vec<JavaValue<'gc_life>>) -> Result<Option<JavaValue<'gc_life>>, WasException> {
    let view = &class.view();
    // let before = int_state.current_frame().operand_stack(jvm).len();
    assert_inited_or_initing_class(jvm, view.type_());
    // assert_eq!(before, int_state.current_frame().operand_stack(jvm).len());
    let method = view.method_view_i(method_i);
    /*if !method.is_static() {
        assert_ne!(before, 0);
    }*/
    assert!(method.is_native());

    let method_as_string = method.name().0.to_str(&jvm.string_pool);
    let noise = vec![
        "arraycopy",
        "getClass",
        "hashCode",
        "getComponentType",
        "getSuperclass",
        "newArray",
        "clone",
        "compareAndSwapObject",
        "identityHashCode",
        "nanoTime",
        "getObjectVolatile",
        "compareAndSwapLong",
        "intern",
        "doPrivileged",
        "invoke0",
        "getDeclaredMethods0",
        "putObjectVolatile",
        "desiredAssertionStatus0",
        "getName0",
        "compareAndSwapInt",
        "getCallerClass",
        "getModifiers",
        "isInstance",
        "findBootstrapClass",
        "findLoadedClass0",
        "getStackTraceElement",
        "objectFieldOffset",
        "getDeclaringClass0",
        "getEnclosingMethod0",
        "isArray",
        "init",
        "isPrimitive",
        "isInterface",
    ]
        .into_iter()
        .collect::<HashSet<_>>();
    if !noise.contains(method_as_string.as_str()) {
        // int_state.debug_print_stack_trace(jvm);
    }
    let parsed = method.desc();
    /*let mut args = vec![];
    if method.is_static() {
        for parameter_type in parsed.arg_types.iter().rev() {
            let rtpye = parameter_type.to_runtime_type().unwrap();
            args.push(int_state.pop_current_operand_stack(Some(rtpye)));
        }
        args.reverse();
    } else if method.is_native() {
        for parameter_type in parsed.arg_types.iter().rev() {
            let rtype = parameter_type.to_runtime_type().unwrap();
            args.push(int_state.pop_current_operand_stack(Some(rtype)));
        }
        args.reverse();
        args.insert(0, int_state.pop_current_operand_stack(Some(CClassName::object().into())));
    } else {
        panic!();
    }*/
    let native_call_frame = int_state.push_frame(StackEntry::new_native_frame(jvm, class.clone(), method_i as u16, args.clone()));
    assert!(int_state.current_frame().is_native());
    let monitor = monitor_for_function(jvm, int_state, &method, method.access_flags() & JVM_ACC_SYNCHRONIZED as u16 > 0);
    if let Some(m) = monitor.as_ref() {
        m.lock(jvm, int_state).unwrap();
    }

    let result = if jvm.native_libaries.registered_natives.read().unwrap().contains_key(&ByAddress(class.clone())) && jvm.native_libaries.registered_natives.read().unwrap().get(&ByAddress(class.clone())).unwrap().read().unwrap().contains_key(&(method_i as u16)) {
        //todo dup
        let res_fn = {
            let reg_natives = jvm.native_libaries.registered_natives.read().unwrap();
            let reg_natives_for_class = reg_natives.get(&ByAddress(class.clone())).unwrap().read().unwrap();
            *reg_natives_for_class.get(&(method_i as u16)).unwrap()
        };
        match call_impl(jvm, int_state, class.clone(), args, parsed.clone(), &res_fn, !method.is_static()) {
            Ok(call_res) => call_res,
            Err(WasException {}) => {
                int_state.pop_frame(jvm, native_call_frame, true);
                return Err(WasException);
            }
        }
    } else {
        match match call(jvm, int_state, class.clone(), method.clone(), args.clone(), parsed.clone()) {
            Ok(call_res) => call_res,
            Err(WasException {}) => {
                int_state.pop_frame(jvm, native_call_frame, true);
                return Err(WasException);
            }
        } {
            Some(r) => r,
            None => match special_call_overrides(jvm, int_state, &class.view().method_view_i(method_i), args) {
                Ok(res) => res,
                Err(_) => None,
            },
        }
    };
    if let Some(m) = monitor.as_ref() {
        m.unlock(jvm, int_state).unwrap();
    }
    let was_exception = int_state.throw().is_some();
    int_state.pop_frame(jvm, native_call_frame, was_exception);
    if was_exception {
        Err(WasException)
    } else {
        Ok(result)
    }
}

fn special_call_overrides(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, method_view: &MethodView, args: Vec<JavaValue<'gc_life>>) -> Result<Option<JavaValue<'gc_life>>, WasException> {
    let mangled = mangling::mangle(&jvm.string_pool, method_view);
    //todo actually impl these at some point
    Ok(if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives" {
        //todo
        None
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getConstant" {
        MHN_getConstant()?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_resolve" {
        MHN_resolve(jvm, int_state, args)?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_init" {
        MHN_init(jvm, int_state, args)?;
        None
    } else if &mangled == "Java_sun_misc_Unsafe_shouldBeInitialized" {
        //todo this isn't totally correct b/c there's a distinction between initialized and initializing.
        shouldBeInitialized(jvm, int_state, args)?.into()
    } else if &mangled == "Java_sun_misc_Unsafe_ensureClassInitialized" {
        let jclass = match args[1].cast_class() {
            None => {
                throw_npe_res(jvm, int_state)?;
                unreachable!()
            }
            Some(class) => class,
        };
        let ptype = jclass.as_runtime_class(jvm).cpdtype();
        check_initing_or_inited_class(jvm, int_state, ptype)?;
        None
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
        Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm, int_state, args)?.into()
    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
        Java_java_lang_invoke_MethodHandleNatives_getMembers(jvm, int_state, args)?.into()
    } else if &mangled == "Java_sun_misc_Unsafe_putObjectVolatile" {
        unimplemented!()
    } else if &mangled == "Java_sun_misc_Perf_registerNatives" {
        //todo not really sure what to do here, for now nothing
        None
    } else if &mangled == "Java_sun_misc_Perf_createLong" {
        Some(HeapByteBuffer::new(jvm, int_state, vec![0, 0, 0, 0, 0, 0, 0, 0], 0, 8)?.java_value())
        //todo this is incorrect and should be implemented properly.
    } else if &mangled == "Java_sun_misc_Unsafe_pageSize" {
        Some(JavaValue::Int(4096)) //todo actually get page size
    } else {
        int_state.debug_print_stack_trace(jvm);
        dbg!(mangled);
        panic!()
    })
}

pub mod mhn_temp;
pub mod unsafe_temp;