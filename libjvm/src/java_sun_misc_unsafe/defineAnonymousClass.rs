use std::collections::HashMap;
use std::env::current_exe;
use std::fs::File;
use std::io::Write;
use std::ops::DerefMut;
use std::panic::resume_unwind;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::AcqRel;

use by_address::ByAddress;
use wtf8::Wtf8Buf;

use another_jit_vm_ir::WasException;
use classfile_parser::parse_class_file;
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jbyteArray, jclass, JNIEnv, jobject, jobjectArray};
use rust_jvm_common::classfile::{Class, Classfile, ConstantInfo, ConstantKind, Utf8};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::compressed_classfile::code::LiveObjectIndex;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::class_loading::create_class_object;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::java_values::{GcManagedObject, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::runtime_class::{initialize_class, prepare_class};
use slow_interpreter::rust_jni::interface::define_class_safe;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::stack_entry::{StackEntry, StackEntryRef};
use verification::{VerifierContext, verify};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineAnonymousClass(env: *mut JNIEnv, the_unsafe: jobject, parent_class: jobject, byte_array: jbyteArray, patches: jobjectArray) -> jclass {
    //todo, big open question here is what if the class has same name as already existing. Also apparently this class should not be part of any loader

    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut args = vec![];
    args.push(JavaValue::Object(from_object(jvm, the_unsafe)));
    args.push(JavaValue::Object(from_object(jvm, parent_class)));
    args.push(JavaValue::Object(from_object(jvm, byte_array)));
    args.push(JavaValue::Object(from_object(jvm, patches)));


    to_object(defineAnonymousClass(jvm, int_state, &mut args).unwrap_object())
    //todo local ref
}

pub fn defineAnonymousClass<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, mut args: &mut Vec<JavaValue<'gc>>) -> JavaValue<'gc> {
    let _parent_class = &args[1]; //todo idk what this is for which is potentially problematic
    let byte_array: Vec<u8> = args[2].unwrap_array().unwrap_byte_array(jvm).iter().map(|b| *b as u8).collect();
    let mut unpatched = parse_class_file(&mut byte_array.as_slice()).expect("todo error handling and verification");
    if args[3].unwrap_object().is_some() {
        patch_all(jvm, int_state.current_frame(), &mut args, &mut unpatched);
    }
    let parsed = Arc::new(unpatched);
    //todo maybe have an anon loader for this
    let current_loader = int_state.current_loader(jvm);

    let class_view = ClassBackedView::from(parsed.clone(), &jvm.string_pool);
    if jvm.config.store_generated_classes {
        File::create(PTypeView::from_compressed(class_view.type_(), &jvm.string_pool).class_name_representation()).unwrap().write_all(byte_array.clone().as_slice()).unwrap();
    }
    match define_class_safe(jvm, int_state, parsed, current_loader, class_view) {
        Ok(res) => res.to_jv(),
        Err(_) => todo!(),
    }
}

fn patch_all<'gc>(jvm: &'gc JVMState<'gc>, frame: StackEntryRef, args: &mut Vec<JavaValue<'gc>>, unpatched: &mut Classfile) {
    let cp_entry_patches = args[3].unwrap_array().unwrap_object_array(jvm);
    assert_eq!(cp_entry_patches.len(), unpatched.constant_pool.len());
    cp_entry_patches.iter().enumerate().for_each(|(i, maybe_patch)| match maybe_patch {
        None => {}
        Some(patch) => {
            patch_single(patch, jvm, &frame, unpatched, i);
        }
    });
    let old_name_temp = class_name(&unpatched);
    let old_name = old_name_temp.get_referred_name();
    let new_name = Wtf8Buf::from_string(format!("{}/{}", old_name, jvm.classes.read().unwrap().anon_classes.len()));
    let name_index = unpatched.constant_pool.len() as u16;
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Utf8(Utf8 { length: new_name.len() as u16, string: new_name }) });
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Class(Class { name_index }) });
    unpatched.this_class = (unpatched.constant_pool.len() - 1) as u16;
}

fn patch_single<'gc>(patch: &GcManagedObject<'gc>, state: &JVMState<'gc>, _frame: &StackEntryRef, unpatched: &mut Classfile, i: usize) {
    let class_name = JavaValue::Object(patch.clone().into()).to_type();

    // Integer, Long, Float, Double: the corresponding wrapper object type from java.lang
    // Utf8: a string (must have suitable syntax if used as signature or name)
    // Class: any java.lang.Class object
    // String: any object (not just a java.lang.String)
    // InterfaceMethodRef: (NYI) a method handle to invoke on that call site's arguments//nyi means not yet implemented
    let _kind = if class_name == CClassName::string().into() {
        unimplemented!()
    } else {
        let mut classes_guard = state.classes.write().unwrap();
        let mut anon_class_write_guard = &mut classes_guard.anon_class_live_object_ldc_pool;
        let live_object_i = anon_class_write_guard.len();
        anon_class_write_guard.push(todo!()/*patch.clone()*/);
        unpatched.constant_pool[i] = ConstantKind::LiveObject(LiveObjectIndex(live_object_i)).into();
    };
}