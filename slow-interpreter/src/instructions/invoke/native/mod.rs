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
use classfile_parser::parse_class_file;
use verification::{verify, VerifierContext};
use classfile_view::view::ClassView;
use std::fs::File;
use std::io::Write;
use crate::instructions::ldc::load_class_constant_by_type;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};

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
                    } else if &mangled == "Java_sun_misc_Unsafe_shouldBeInitialized" {
                        //todo this isn't totally correct b/c there's a distinction between initialized and initializing.
                        shouldBeInitialized(state, &mut args)
                    } else if &mangled == "Java_sun_misc_Unsafe_ensureClassInitialized" {
                        if shouldBeInitialized(state, &mut args).unwrap().unwrap_int() != 1 {
                            panic!()
                        }
                        None
                    } else if &mangled == "Java_sun_misc_Unsafe_defineAnonymousClass" {
                        let _parent_class = &args[1];//todo idk what this is for which is potentially problematic
                        let byte_array:Vec<u8> = args[2].unwrap_array().unwrap_byte_array().iter().map(|b| *b as u8 ).collect();
                        //todo for debug, delete later
                        let cloned=  byte_array.clone();
                        let cp_entry_patches = args[3].unwrap_array().unwrap_object_array();
                        if !cp_entry_patches.is_empty() {
                            assert!(cp_entry_patches.iter().all(|x|x.is_none()))
                            // unimplemented!()
                        }

                        // let loader = parent_class.unwrap_normal_object().class_pointer.loader.clone();//todo so we aren't meant to use any loader but verify needs something?
                        let parsed= parse_class_file(&mut byte_array.as_slice());
                        //todo maybe have an anon loader for this
                        let bootstrap_loader = state.bootstrap_loader.clone();

                        let vf = VerifierContext { bootstrap_loader: bootstrap_loader.clone() };
                        let class_view = ClassView::from(parsed.clone());

                        File::create(format!("{}.class",class_view.name().get_referred_name().replace("/","_"))).unwrap().write(cloned.as_slice()).unwrap();
                        bootstrap_loader.add_pre_loaded(&class_view.name(),&parsed);
                        match verify(&vf, class_view.clone(), bootstrap_loader.clone()){
                            Ok(_) => {},
                            Err(_) => panic!(),
                        };
                        load_class_constant_by_type(state,&frame,&PTypeView::Ref(ReferenceTypeView::Class(class_view.name())));
                        frame.pop().into()
                    }

                    else {
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