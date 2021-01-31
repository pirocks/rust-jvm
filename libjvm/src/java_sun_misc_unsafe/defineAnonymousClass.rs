use std::collections::HashMap;
use std::env::current_exe;
use std::fs::File;
use std::io::Write;
use std::ops::DerefMut;
use std::panic::resume_unwind;
use std::sync::{Arc, RwLock};
use std::sync::atomic::Ordering;
use std::sync::atomic::Ordering::AcqRel;

use by_address::ByAddress;

use classfile_parser::parse_class_file;
use classfile_view::loading::LoaderName;
use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jbyteArray, jclass, JNIEnv, jobject, jobjectArray};
use rust_jvm_common::classfile::{Class, Classfile, ConstantInfo, ConstantKind, Utf8};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::class_loading::create_class_object;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::jvm_state::{ClassStatus, JVMState};
use slow_interpreter::runtime_class::{initialize_class, prepare_class, RuntimeClass, RuntimeClassClass};
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::stack_entry::StackEntry;
use verification::{VerifierContext, verify};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_defineAnonymousClass(env: *mut JNIEnv, the_unsafe: jobject, parent_class: jobject, byte_array: jbyteArray, patches: jobjectArray) -> jclass {
    //todo, big open question here is what if the class has same name as already existing. Also apparently this class should not be part of any loader

    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut args = vec![];
    args.push(JavaValue::Object(from_object(the_unsafe)));
    args.push(JavaValue::Object(from_object(parent_class)));
    args.push(JavaValue::Object(from_object(byte_array)));
    args.push(JavaValue::Object(from_object(patches)));

    to_object(defineAnonymousClass(jvm, int_state, &mut args).unwrap_object())//todo local ref
}

pub fn defineAnonymousClass(jvm: &JVMState, int_state: &mut InterpreterStateGuard, mut args: &mut Vec<JavaValue>) -> JavaValue {
    let _parent_class = &args[1];//todo idk what this is for which is potentially problematic
    let byte_array: Vec<u8> = args[2].unwrap_array().unwrap_byte_array().iter().map(|b| *b as u8).collect();
    //todo for debug, delete later
    let mut unpatched = parse_class_file(&mut byte_array.as_slice());
    // dbg!(unpatched.constant_pool[unpatched.methods[0].name_index as usize].extract_string_from_utf8());
    // int_state.print_stack_trace();
    if args[3].unwrap_object().is_some() {
        patch_all(jvm, &int_state.current_frame_mut(), &mut args, &mut unpatched);
    }
    let parsed = Arc::new(unpatched);
    //todo maybe have an anon loader for this
    let current_loader = int_state.current_loader();

    let vf = VerifierContext { live_pool_getter: jvm.get_live_object_pool_getter(), classfile_getter: jvm.get_class_getter(int_state.current_loader()), current_loader };
    let class_view = ClassView::from(parsed.clone());
    File::create(class_view.name().get_referred_name().replace("/", ".")).unwrap().write(byte_array.clone().as_slice()).unwrap();
    let class_name = class_view.name();
    let runtime_class = Arc::new(RuntimeClass::Object(RuntimeClassClass {
        class_view: Arc::new(class_view),
        static_vars: Default::default(),
        status: RwLock::new(ClassStatus::UNPREPARED),
    }));
    let class_object = create_class_object(jvm, int_state, None, current_loader);
    let mut classes = jvm.classes.write().unwrap();
    let current_loader = int_state.current_loader();
    classes.initiating_loaders.insert(class_name.clone().into(), (current_loader, runtime_class.clone()));
    classes.loaded_classes_by_type.entry(current_loader).or_insert(HashMap::new()).entry(class_name.clone().into()).insert(runtime_class.clone());
    classes.class_object_pool.insert(ByAddress(class_object), ByAddress(runtime_class.clone()));
    drop(classes);
    prepare_class(jvm, parsed.clone(), &mut *runtime_class.static_vars());
    runtime_class.set_status(ClassStatus::PREPARED);
    runtime_class.set_status(ClassStatus::INITIALIZING);
    initialize_class(runtime_class.clone(), jvm, int_state).unwrap();
    runtime_class.set_status(ClassStatus::INITIALIZED);
    JavaValue::Object(get_or_create_class_object(jvm, class_name.into(), int_state).unwrap().into())
}


fn patch_all(jvm: &JVMState, frame: &StackEntry, args: &mut Vec<JavaValue>, unpatched: &mut Classfile) {
    let cp_entry_patches = args[3].unwrap_array().unwrap_object_array();
    assert_eq!(cp_entry_patches.len(), unpatched.constant_pool.len());
    cp_entry_patches.iter().enumerate().for_each(|(i, maybe_patch)| {
        match maybe_patch {
            None => {}
            Some(patch) => {
                patch_single(patch, jvm, frame, unpatched, i);
            }
        }
    });
    let old_name_temp = class_name(&unpatched);
    let old_name = old_name_temp.get_referred_name();
    let new_name = format!("{}/{}", old_name, jvm.classes.read().unwrap().anon_classes.read().unwrap().len());
    let name_index = unpatched.constant_pool.len() as u16;
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Utf8(Utf8 { length: new_name.len() as u16, string: new_name }) });
    unpatched.constant_pool.push(ConstantInfo { kind: ConstantKind::Class(Class { name_index }) });
    unpatched.this_class = (unpatched.constant_pool.len() - 1) as u16;
}

fn patch_single(
    patch: &Arc<Object>,
    state: &JVMState,
    _frame: &StackEntry,
    unpatched: &mut Classfile,
    i: usize,
) {
    let class_name = JavaValue::Object(patch.clone().into()).to_type();

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
} else*/ if class_name == ClassName::string().into() {
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
        let classes_guard = state.classes.write().unwrap();
        let mut anon_class_write_guard = classes_guard.anon_class_live_object_ldc_pool.write().unwrap();
        let live_object_i = anon_class_write_guard.len();
        anon_class_write_guard.push(patch.clone());
        unpatched.constant_pool[i] = ConstantKind::LiveObject(live_object_i).into();
    };
}
