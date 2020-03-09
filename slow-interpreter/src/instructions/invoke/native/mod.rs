use rust_jvm_common::classnames::class_name;
use runtime_common::java_values::JavaValue;
use crate::rust_jni::{mangling, call_impl, call};
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC};
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use std::sync::Arc;

use classfile_view::view::descriptor_parser::MethodDescriptor;
use crate::instructions::invoke::native::system_temp::system_array_copy;
use crate::instructions::invoke::native::mhn_temp::*;
use crate::instructions::invoke::native::unsafe_temp::*;

pub fn run_native_method(
    state: &mut InterpreterState,
    frame: Rc<StackEntry>,
    class: Arc<RuntimeClass>,
    method_i: usize,
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags & ACC_NATIVE > 0);
    let parsed = MethodDescriptor::from_legacy(method, classfile);
    let mut args = vec![];
    //todo should have some setup args functions
    if method.access_flags & ACC_STATIC > 0 {
        for _ in &parsed.parameter_types {
            args.push(frame.pop());
        }
        args.reverse();
    } else {
        if method.access_flags & ACC_NATIVE > 0 {
            for _ in &parsed.parameter_types {
                args.push(frame.pop());
            }
            args.reverse();
            args.insert(0, frame.pop());
        } else {
            panic!();
        }
    }
    println!("CALL BEGIN NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
    if method.method_name(classfile) == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.push(JavaValue::Boolean(false))
    } else if method.method_name(classfile) == "arraycopy".to_string() {
        system_array_copy(&mut args)
    } else {
        let result = if state.jni.registered_natives.borrow().contains_key(&class) &&
            state.jni.registered_natives.borrow().get(&class).unwrap().borrow().contains_key(&(method_i as u16))
        {
            //todo dup
            let res_fn = {
                let reg_natives = state.jni.registered_natives.borrow();
                let reg_natives_for_class = reg_natives.get(&class).unwrap().borrow();
                reg_natives_for_class.get(&(method_i as u16)).unwrap().clone()
            };
            call_impl(state, frame.clone(), class.clone(), args, parsed, &res_fn, !method.is_static())
        } else {
            let res = match call(state, frame.clone(), class.clone(), method_i, args.clone(), parsed) {
                Ok(r) => r,
                Err(_) => {
                    let mangled = mangling::mangle(class.clone(), method_i);
                    //todo actually impl these at some point
                    if mangled == "Java_sun_misc_Unsafe_objectFieldOffset".to_string() {
                        object_field_offset(state,&frame,&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_getIntVolatile".to_string() {
                        get_int_volatile(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_compareAndSwapInt".to_string() {
                        compare_and_swap_int(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_allocateMemory".to_string() {
                        allocate_memory(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_putLong__JJ".to_string() {
                        putLong__JJ(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_getByte__J".to_string() {
                        getByte__J(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_freeMemory".to_string() {
                        freeMemory(&mut args)
                        //todo all these unsafe function thingys are getting a tad excessive
                    } else if mangled == "Java_sun_misc_Unsafe_getObjectVolatile".to_string() {
                        get_object_volatile(&mut args)
                    } else if mangled == "Java_sun_misc_Unsafe_compareAndSwapLong".to_string() {
                        compare_and_swap_long(&mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_registerNatives" {
                        //todo
                        None
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getConstant" {
                        MHN_getConstant()
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_resolve" {
                        MHN_resolve(state, &frame, &mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_init"{
                        MHN_init(state, &frame, &mut args)
                    } else {
                        frame.print_stack_trace();
                        dbg!(mangled);
                        panic!()
                    }
                }
            };
            res
        };
        match result {
            None => {}
            Some(res) => frame.push(res),
        }
    }
    println!("CALL END NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
}

pub mod mhn_temp;
pub mod unsafe_temp;
pub mod system_temp;