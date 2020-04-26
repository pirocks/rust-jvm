use rust_jvm_common::classnames::ClassName;
use crate::rust_jni::{mangling, call_impl, call};
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC, ConstantInfo, ConstantKind, Class, Utf8, Classfile};

use std::sync::Arc;

use crate::instructions::invoke::native::system_temp::system_array_copy;
use crate::instructions::invoke::native::mhn_temp::*;
use crate::instructions::invoke::native::unsafe_temp::*;
use classfile_parser::parse_class_file;
use verification::{verify, VerifierContext};
use classfile_view::view::ClassView;
use crate::instructions::ldc::load_class_constant_by_type;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::{JVMState, StackEntry};
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue};
use std::fs::File;
use std::io::Write;
use descriptor_parser::MethodDescriptor;
use crate::java::lang::reflect::field::Field;
use crate::java::lang::string::JString;
use crate::sun::misc::unsafe_::Unsafe;
use std::sync::atomic::Ordering;

pub fn run_native_method(
    state: & JVMState,
    frame: &StackEntry,
    class: Arc<RuntimeClass>,
    method_i: usize,
    _debug: bool,
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
    } else if method.access_flags & ACC_NATIVE > 0 {
        for _ in &parsed.parameter_types {
            args.push(frame.pop());
        }
        args.reverse();
        args.insert(0, frame.pop());
    } else {
        panic!();
    }

    if _debug {
        // dbg!(&args);
        // dbg!(&frame.operand_stack);
    }
    // println!("CALL BEGIN NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
    let meth_name = method.method_name(classfile);
    let debug = false;//meth_name.contains("isAlive");
    if meth_name == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.push(JavaValue::Boolean(false))
    } else if meth_name == "arraycopy".to_string() {
        system_array_copy(&mut args)
    } else {
        let result = if state.jni.registered_natives.read().unwrap().contains_key(&class) &&
            state.jni.registered_natives.read().unwrap().get(&class).unwrap().read().unwrap().contains_key(&(method_i as u16))
        {
            //todo dup
            let res_fn = {
                let reg_natives = state.jni.registered_natives.read().unwrap();
                let reg_natives_for_class = reg_natives.get(&class).unwrap().read().unwrap();
                reg_natives_for_class.get(&(method_i as u16)).unwrap().clone()
            };
            call_impl(state, frame.clone(), class.clone(), args, parsed, &res_fn, !method.is_static(), debug)
        } else {
            let res = match call(state, frame.clone(), class.clone(), method_i, args.clone(), parsed) {
                Ok(r) => r,
                Err(_) => {
                    let mangled = mangling::mangle(class.clone(), method_i);
                    //todo actually impl these at some point
                    if mangled == "Java_sun_misc_Unsafe_objectFieldOffset".to_string() {
                        object_field_offset(state, &frame, &mut args)
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
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_init" {
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
                        let byte_array: Vec<u8> = args[2].unwrap_array().unwrap_byte_array().iter().map(|b| *b as u8).collect();
                        //todo for debug, delete later
                        let mut unpatched = parse_class_file(&mut byte_array.as_slice());
                        File::create("unpatched.class").unwrap().write(byte_array.clone().as_slice()).unwrap();
                        patch_all(state, &frame, &mut args, &mut unpatched);
                        let parsed = Arc::new(unpatched);
                        //todo maybe have an anon loader for this
                        let bootstrap_loader = state.bootstrap_loader.clone();

                        let vf = VerifierContext { live_pool_getter: state.get_live_object_pool_getter(), bootstrap_loader: bootstrap_loader.clone() };
                        let class_view = ClassView::from(parsed.clone());

                        bootstrap_loader.add_pre_loaded(&class_view.name(), &parsed);
                        // frame.print_stack_trace();
                        match verify(&vf, class_view.clone(), bootstrap_loader.clone()) {
                            Ok(_) => {}
                            Err(_) => panic!(),
                        };
                        load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(class_view.name())));
                        frame.pop().into()
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset" {
                        let member_name = args[0].cast_member_name();
                        let name = member_name.get_name(state, frame);
                        let clazz = member_name.clazz();
                        let field_type = member_name.get_field_type(state, frame.clone());
                        let empty_string = JString::from(state, &frame, "".to_string());
                        let field = Field::init(state, &frame, clazz, name, field_type, 0, 0, empty_string, vec![]);

                        let mut args = vec![Unsafe::the_unsafe(state, &frame).java_value(), field.java_value()];
                        object_field_offset(state, &frame, &mut args)
                    } else if &mangled == "Java_java_lang_invoke_MethodHandleNatives_getMembers" {
//static native int getMembers(Class<?> defc, String matchName, String matchSig,
// //          int matchFlags, Class<?> caller, int skip, MemberName[] results);
                        dbg!(args);
                        //todo nyi
                        // unimplemented!()
                        Some(JavaValue::Int(0))
                    } else {
                        // frame.print_stack_trace();
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
                    dbg!(frame.operand_stack.borrow());
                    dbg!(&res);
                }
                frame.push(res)
            }
        }
    }
    // println!("CALL END NATIVE:{} {} {}", class_name(classfile).get_referred_name(), method.method_name(classfile), frame.depth());
}

fn patch_all(state: & JVMState, frame: & StackEntry, args: &mut Vec<JavaValue>, unpatched: &mut Classfile) {
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
    let new_name = format!("java/lang/invoke/LambdaForm$DMH/{}", state.anon_class_counter.fetch_add(1,Ordering::SeqCst));
    let name_index = unpatched.constant_pool.len() as u16;
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Utf8(Utf8 { length: new_name.len() as u16, string: new_name }) });
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Class(Class { name_index }) });
    unpatched.this_class = (unpatched.constant_pool.len() - 1) as u16;
}

fn patch_single(
    patch: &Arc<Object>,
    state: & JVMState,
    _frame: & StackEntry,
    unpatched: &mut Classfile,
    i: usize,
) {
    let class_name = patch.unwrap_normal_object().class_pointer.class_view.name();

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
        let mut anon_class_write_guard = state.anon_class_live_object_ldc_pool.write().unwrap();
        let live_object_i = anon_class_write_guard.len();
        anon_class_write_guard.push(patch.clone());
        unpatched.constant_pool[i] = ConstantKind::LiveObject(live_object_i).into();
    };
}


pub mod mhn_temp;
pub mod unsafe_temp;
pub mod system_temp;