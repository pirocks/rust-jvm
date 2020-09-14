#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//all of these functions should be implemented in libjvm
use std::mem::transmute;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::{Class, Classfile, ConstantInfo, ConstantKind, Utf8};
use rust_jvm_common::classnames::ClassName;
use verification::{VerifierContext, verify};

use crate::field_table::FieldId;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::{JavaValue, Object};
use crate::JVMState;
use crate::stack_entry::StackEntry;

pub fn get_object_volatile(jvm: &'static JVMState, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    match args[1].unwrap_object() {
        None => {
            let field_id = args[2].unwrap_long() as FieldId;
            let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
            let field_view = runtime_class.view().field(i as usize);
            assert!(field_view.is_static());
            let name = field_view.field_name();
            let res = runtime_class.static_vars().get(&name).unwrap().clone();
            res.into()
        }
        Some(object_to_read) => {
            match object_to_read.deref() {
                Object::Array(arr) => {
                    let array_idx = args[2].unwrap_long() as usize;
                    let res = &arr.elems.borrow()[array_idx];
                    res.clone().into()
                }
                Object::Object(_) => unimplemented!(),
            }
        }
    }
}

pub fn freeMemory(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        libc::free(transmute(args[1].unwrap_long()))
    };
    None
}

pub fn getByte__J(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        let ptr: *mut i8 = transmute(args[1].unwrap_long());
        JavaValue::Byte(ptr.read()).into()
    }
}

pub fn putLong__JJ(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        let ptr: *mut i64 = transmute(args[1].unwrap_long());
        let val = args[2].unwrap_long();
        ptr.write(val);
    }
    None
}

pub fn allocate_memory(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let res: i64 = unsafe {
        transmute(libc::malloc(transmute(args[1].unwrap_long())))
    };
    JavaValue::Long(res).into()
}


pub fn shouldBeInitialized(state: &'static JVMState, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let class_name_to_check = args[1].cast_class().as_type();
    JavaValue::Boolean(state.classes.initialized_classes.read().unwrap().get(&class_name_to_check).is_some() as u8).into()
}


pub fn Java_sun_misc_Unsafe_defineAnonymousClass(jvm: &JVMState, int_state: &mut InterpreterStateGuard, mut args: &mut Vec<JavaValue>) -> Option<JavaValue> {
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
}


fn patch_all(state: &JVMState, frame: &StackEntry, args: &mut Vec<JavaValue>, unpatched: &mut Classfile) {
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
