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
use crate::instructions::invoke::native::system_temp::system_array_copy;
use crate::instructions::invoke::native::unsafe_temp::*;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter::monitor_for_function;
use crate::java::lang::reflect::field::Field;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, Object};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::{call, call_impl, mangling};
use crate::sun::misc::unsafe_::Unsafe;

pub fn run_native_method<'l>(
    jvm: &'static JVMState,
    int_state: &mut InterpreterStateGuard,
    class: Arc<RuntimeClass>,
    method_i: usize,
    _debug: bool,
) {
    let view = &class.view();
    let method = &view.method_view_i(method_i);
    assert!(method.is_native());
    let parsed = method.desc();
    let mut args = vec![];
    //todo should have some setup args functions
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
    let monitor = monitor_for_function(jvm, int_state, method, method.access_flags() & ACC_SYNCHRONIZED as u16 > 0, &class.view().name());
    monitor.as_ref().map(|m| m.lock(jvm));
    if _debug {
        // dbg!(&args);
        // dbg!(&frame.operand_stack);
    }
    // println!("CALL BEGIN NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
    let meth_name = method.name();
    let debug = false;//meth_name.contains("isAlive");
    if meth_name == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        int_state.push_current_operand_stack(JavaValue::Boolean(0))
    } else if meth_name == "arraycopy".to_string() {
        system_array_copy(&mut args)
    } else {
        let result = if jvm.libjava.registered_natives.read().unwrap().contains_key(&class) &&
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
                        get_object_volatile(&mut args)
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
                        let _parent_class = &args[1];//todo idk what this is for which is potentially problematic
                        let byte_array: Vec<u8> = args[2].unwrap_array().unwrap_byte_array().iter().map(|b| *b as u8).collect();
                        //todo for debug, delete later
                        let mut unpatched = parse_class_file(&mut byte_array.as_slice());

                        patch_all(jvm, &int_state.current_frame_mut(), &mut args, &mut unpatched);
                        let parsed = Arc::new(unpatched);
                        //todo maybe have an anon loader for this
                        let bootstrap_loader = jvm.bootstrap_loader.clone();

                        let vf = VerifierContext { live_pool_getter: jvm.get_live_object_pool_getter(), bootstrap_loader: bootstrap_loader.clone() };
                        let class_view = ClassView::from(parsed.clone());
                        // File::create(class_view.name().get_referred_name().replace("/",".")).unwrap().write(byte_array.clone().as_slice()).unwrap();
                        let class_name = class_view.name();
                        bootstrap_loader.add_pre_loaded(&class_name, &parsed);
                        // frame.print_stack_trace();
                        match verify(&vf, &class_view, bootstrap_loader.clone()) {
                            Ok(_) => {}
                            Err(_) => panic!(),
                        };
                        load_class_constant_by_type(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Class(class_name)));
                        int_state.pop_current_operand_stack().into()
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
                        let member_name = args[0].cast_member_name();
                        let name = member_name.get_name(jvm, int_state);
                        let clazz = member_name.clazz();
                        let field_type = member_name.get_field_type(jvm, int_state);
                        let empty_string = JString::from(jvm, int_state, "".to_string());
                        let field = Field::init(jvm, int_state, clazz, name, field_type, 0, 0, empty_string, vec![]);
                        Unsafe::the_unsafe(jvm, int_state).object_field_offset(jvm, int_state, field).into()
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
//static native int getMembers(Class<?> defc, String matchName, String matchSig,
// //          int matchFlags, Class<?> caller, int skip, MemberName[] results);
                        dbg!(args);
                        //todo nyi
                        // unimplemented!()
                        Some(JavaValue::Int(0))
                    } else if &mangled == "Java_java_lang_Thread_currentThread" {
                        Some(jvm.thread_state.get_current_thread().thread_object().java_value())
                    } else if &mangled == "Java_java_lang_Thread_setPriority0" {
                        None//todo for now unimplemented
                    } else if &mangled == "Java_java_lang_Thread_isAlive" {
                        let maybe_java_thread = args[0].cast_thread().try_get_java_thread(jvm);
                        match maybe_java_thread {
                            None => {
                                Some(JavaValue::Boolean(false as u8))
                            },
                            Some(java_thread) => {
                                let is_alive = java_thread.is_java_alive();
                                Some(JavaValue::Boolean(is_alive as u8))
                            },
                        }
                    } else if &mangled == "Java_java_lang_Thread_start0" {
                        int_state.print_stack_trace();
                        let java_thread = args[0].cast_thread().get_java_thread(jvm);
                        dbg!(args[0].cast_thread().name().to_rust_string());
                        jvm.thread_state.start_thread_from_obj(jvm, java_thread.thread_object(), int_state, false);
                        None
                    } else {
                        int_state.print_stack_trace();
                        dbg!(mangled);
                        panic!()
                    }
                }
            };
            res
        };
        match result {
            None => {}
            Some(res) => {
                if debug {
                    // dbg!(&frame.operand_stack);
                    dbg!(&res);
                }
                int_state.push_current_operand_stack(res)
            }
        }
    }
    monitor.as_ref().map(|m| m.unlock(jvm));
    // println!("CALL END NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
}

fn patch_all(state: &'static JVMState, frame: &StackEntry, args: &mut Vec<JavaValue>, unpatched: &mut Classfile) {
    let cp_entry_patches = args[3].unwrap_array().unwrap_object_array();
    assert_eq!(cp_entry_patches.len(), unpatched.constant_pool.len());
    cp_entry_patches.iter().enumerate().for_each(|(i, maybe_patch)| {
        match maybe_patch {
            None => {}
            Some(patch) => {
                patch_single(patch, state, frame, unpatched, i);
            }
        }
    });
    let new_name = format!("java/lang/invoke/LambdaForm$DMH/{}", state.classes.anon_class_counter.fetch_add(1, Ordering::SeqCst));
    let name_index = unpatched.constant_pool.len() as u16;
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Utf8(Utf8 { length: new_name.len() as u16, string: new_name }) });
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Class(Class { name_index }) });
    unpatched.this_class = (unpatched.constant_pool.len() - 1) as u16;
}

fn patch_single(
    patch: &Arc<Object>,
    state: &'static JVMState,
    _frame: &StackEntry,
    unpatched: &mut Classfile,
    i: usize,
) {
    let class_name = patch.unwrap_normal_object().class_pointer.view().name();

    // Integer, Long, Float, Double: the corresponding wrapper object type from java.lang
    // Utf8: a string (must have suitable syntax if used as signature or name)
    // Class: any java.lang.Class object
    // String: any object (not just a java.lang.String)
    // InterfaceMethodRef: (NYI) a method handle to invoke on that call site's arguments//nyi means not yet implemented
    // dbg!(&class_name);
    let _kind = /*if class_name == ClassName::int() {
        let int_val = JavaValue::Object(patch.clone().into()).cast_integer().value();
        unpatched.constant_pool[i] = ConstantKind::Integer(Integer { bytes: int_val as u32 }).into();
    } else*/ /*if
    class_name == ClassName::long() ||
        class_name == ClassName::float() ||
        class_name == ClassName::double() {
        frame.print_stack_trace();

        unimplemented!()
    } else*/ if class_name == ClassName::string() {
        unimplemented!()
    } /*else if class_name == ClassName::class() {
        unimplemented!()
    }*/ /*else if class_name == ClassName::method_handle() || class_name == ClassName::direct_method_handle() {//todo should be using innstanceof here
        dbg!(&unpatched.constant_pool[i]);
        dbg!(&unpatched.constant_pool.iter().enumerate().collect::<Vec<_>>());
        if class_name == ClassName::direct_method_handle() {
            let patch_fields = patch.unwrap_normal_object().fields.borrow_mut();
            let member_name_obj = patch_fields.get("member").unwrap();
            let member_name_obj_fields = member_name_obj.unwrap_normal_object().fields.borrow();
            let name_i = {
                let name = member_name_obj_fields.get("name").unwrap();
                let member_name = string_obj_to_string(name.unwrap_object());
                let res_i = unpatched.constant_pool.len();
                unpatched.constant_pool.push(ConstantKind::Utf8(Utf8 {
                    length: member_name.len() as u16,
                    string: member_name,
                }).into());
                res_i
            };
            let class_i = {
                let clazz = member_name_obj_fields.get("clazz").unwrap();
                let clazz_ptype_borrow = clazz.unwrap_normal_object().class_object_ptype.borrow();
                let clazz_name_as_class_name = clazz_ptype_borrow.as_ref().unwrap().unwrap_class_type();
                let clazz_name = clazz_name_as_class_name.get_referred_name();
                let utf_i = unpatched.constant_pool.len();
                unpatched.constant_pool.push(ConstantKind::Utf8(Utf8 {
                    length: clazz_name.len() as u16,
                    string: clazz_name.to_string(),
                }).into());
                let class_i = unpatched.constant_pool.len();
                unpatched.constant_pool.push(ConstantKind::Class(Class { name_index: utf_i as u16 }).into());
                class_i
            };
            let descriptor_i = {
                let type_ = member_name_obj_fields.get("type").unwrap();
                let method_type = type_.unwrap_normal_object().cast_method_type();
                let method_descriptor = method_type.to_string(state,frame.clone()).to_rust_string();


                let descriptor_i = unpatched.constant_pool.len();
                unpatched.constant_pool.push(ConstantKind::Utf8(Utf8 {
                    length: method_descriptor.len() as u16,
                    string: method_descriptor,
                }).into());
                descriptor_i
            };

            let nt_i = unpatched.constant_pool.len();
            unpatched.constant_pool.push(ConstantKind::NameAndType(NameAndType {
                name_index: name_i as u16,
                descriptor_index: descriptor_i as u16,
            }).into());

            unpatched.constant_pool[i] = ConstantKind::InterfaceMethodref(InterfaceMethodref {
                class_index: class_i as u16,
                nt_index: nt_i as u16
            }).into();
        } else {
            unimplemented!()
        }
    }*/ else {
        // dbg!(&class_name);
        // assert!(class_name == ClassName::unsafe_() || class_name == ClassName::direct_method_handle());//for now keep a white list of allowed classes here until the above are properly implemented
        let mut anon_class_write_guard = state.classes.anon_class_live_object_ldc_pool.write().unwrap();
        let live_object_i = anon_class_write_guard.len();
        anon_class_write_guard.push(patch.clone());
        unpatched.constant_pool[i] = ConstantKind::LiveObject(live_object_i).into();
    };
}


pub mod mhn_temp;
pub mod unsafe_temp;
pub mod system_temp;