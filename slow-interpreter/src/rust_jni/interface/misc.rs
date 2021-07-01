use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

use by_address::ByAddress;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{JavaVM, jboolean, jclass, jint, JNI_ERR, JNI_FALSE, JNI_OK, JNI_TRUE, JNIEnv, JNINativeMethod, jobject};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::parse_field_type;
use verification::verifier::filecorrectness::is_assignable;
use verification::VerifierContext;

use crate::class_loading::{assert_loaded_class, check_initing_or_inited_class};
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::invoke_interface::get_invoke_interface;
use crate::java_values::{GcManagedObject, JavaValue};
use crate::jvm_state::{JVMState, LibJavaLoading};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use crate::utils::throw_npe;

pub unsafe extern "C" fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    //we always have ram, blocking on gc.
    0 as jint
}

pub unsafe extern "C" fn find_class(env: *mut JNIEnv, c_name: *const ::std::os::raw::c_char) -> jclass {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let (remaining, type_) = parse_field_type(name.as_str()).unwrap();
    assert!(remaining.is_empty());
    if let Err(WasException {}) = load_class_constant_by_type(jvm, int_state, PTypeView::from_ptype(&type_)) {
        return null_mut();
    };
    let obj = int_state.pop_current_operand_stack(ClassName::object().into()).unwrap_object();
    new_local_ref_public(obj, int_state)
}


pub unsafe extern "C" fn get_superclass(env: *mut JNIEnv, sub: jclass) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let super_name = match from_jclass(jvm, sub).as_runtime_class(jvm).view().super_name() {
        None => return null_mut(),
        Some(n) => n,
    };
    let _inited_class = assert_loaded_class(jvm, super_name.clone().into());
    if let Err(WasException {}) = load_class_constant_by_type(jvm, int_state, PTypeView::Ref(ReferenceTypeView::Class(super_name))) {
        return null_mut();
    };
    new_local_ref_public(int_state.pop_current_operand_stack(ClassName::object().into()).unwrap_object(), int_state)
}


pub unsafe extern "C" fn is_assignable_from(env: *mut JNIEnv, sub: jclass, sup: jclass) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let sub_not_null = match from_object(jvm, sub) {
        Some(x) => x,
        None => return throw_npe(jvm, int_state),
    };
    let sup_not_null = match from_object(jvm, sup) {
        Some(x) => x,
        None => return throw_npe(jvm, int_state),
    };

    let sub_type = JavaValue::Object(sub_not_null.into()).cast_class().unwrap().as_type(jvm);
    let sup_type = JavaValue::Object(sup_not_null.into()).cast_class().unwrap().as_type(jvm);

    let loader = &int_state.current_loader();
    let sub_vtype = sub_type.to_verification_type(&loader);
    let sup_vtype = sup_type.to_verification_type(&loader);


    //todo should this be current loader?
    let vf = VerifierContext { live_pool_getter: jvm.get_live_object_pool_getter(), classfile_getter: jvm.get_class_getter(int_state.current_loader()), current_loader: loader.clone(), verification_types: Default::default(), debug: false };
    let res = is_assignable(&vf, &sub_vtype, &sup_vtype).map(|_| true).unwrap_or(false);
    res as jboolean
}


pub unsafe extern "C" fn get_java_vm(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    let state = get_state(env);
    let int_state = get_interpreter_state(env);//todo maybe this should have an optionable version
    let interface = get_invoke_interface(state, int_state);
    *vm = Box::into_raw(box interface);//todo do something about this leak
    0 as jint
}


pub unsafe extern "C" fn is_same_object(env: *mut JNIEnv, obj1: jobject, obj2: jobject) -> jboolean {
    let jvm = get_state(env);
    let _1 = from_object(jvm, obj1);
    let _2 = from_object(jvm, obj2);
    (match _1 {
        None => {
            match _2 {
                None => JNI_TRUE,
                Some(_) => JNI_FALSE,
            }
        }
        Some(_1_) => {
            match _2 {
                None => JNI_FALSE,
                Some(_2_) => GcManagedObject::ptr_eq(&_1_, &_2_) as u32,
            }
        }
    }) as u8
}


///jint UnregisterNatives(JNIEnv *env, jclass clazz);
//
// Unregisters native methods of a class. The class goes back to the state before it was linked or registered with its native method functions.
//
// This function should not be used in normal native code. Instead, it provides special programs a way to reload and relink native libraries.
// LINKAGE:
// Index 216 in the JNIEnv interface function table.
// PARAMETERS:
//
// env: the JNI interface pointer.
//
// clazz: a Java class object.
// RETURNS:
//
// Returns “0” on success; returns a negative value on failure.
pub unsafe extern "C" fn unregister_natives(env: *mut JNIEnv, clazz: jclass) -> jint {
    let jvm = get_state(env);
    let rc = from_jclass(jvm, clazz).as_runtime_class(jvm);
    if let None = jvm.libjava.registered_natives.write().unwrap().remove(&ByAddress(rc)) {
        return JNI_ERR;
    }
    JNI_OK as i32
}


pub unsafe extern "C" fn register_natives<'gc_life>(env: *mut JNIEnv,
                                                    clazz: jclass,
                                                    methods: *const JNINativeMethod,
                                                    n_methods: jint) -> jint {
    let jvm = get_state(env);
    for to_register_i in 0..n_methods {
        let method = *methods.offset(to_register_i as isize);
        let expected_name: String = CStr::from_ptr(method.name).to_str().unwrap().to_string().clone();
        let descriptor: String = CStr::from_ptr(method.signature).to_str().unwrap().to_string().clone();
        let runtime_class: Arc<RuntimeClass<'gc_life>> = from_jclass(jvm, clazz).as_runtime_class(jvm);
        let class_name = match runtime_class.ptypeview().try_unwrap_class_type() {
            None => { return JNI_ERR; }
            Some(cn) => cn,
        };
        let view = runtime_class.view();
        let jni_context = &jvm.libjava;
        view.methods().enumerate().for_each(|(i, method_info)| {
            let descriptor_str = method_info.desc_str();
            let current_name = method_info.name();
            if current_name == expected_name && descriptor == descriptor_str {
                jvm.tracing.trace_jni_register(&class_name, expected_name.as_str());
                register_native_with_lib_java_loading(jni_context, &method, &runtime_class, i)
            }
        });
    }
    0
}


fn register_native_with_lib_java_loading(jni_context: &LibJavaLoading<'gc_life>, method: &JNINativeMethod, runtime_class: &Arc<RuntimeClass<'gc_life>>, method_i: usize) {
    if jni_context.registered_natives.read().unwrap().contains_key(&ByAddress(runtime_class.clone())) {
        unsafe {
            jni_context.registered_natives
                .read().unwrap()
                .get(&ByAddress(runtime_class.clone()))
                .unwrap()
                .write().unwrap()
                .insert(method_i as CPIndex, transmute(method.fnPtr));
        }
    } else {
        let mut map = HashMap::new();
        map.insert(method_i as CPIndex, unsafe { transmute(method.fnPtr) });
        jni_context.registered_natives.write().unwrap().insert(ByAddress(runtime_class.clone()), RwLock::new(map));
    }
}


pub fn get_all_methods(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class: Arc<RuntimeClass<'gc_life>>, include_interface: bool) -> Result<Vec<(Arc<RuntimeClass<'gc_life>>, u16)>, WasException> {
    let mut res = vec![];
    get_all_methods_impl(jvm, int_state, class, &mut res, include_interface)?;
    Ok(res)
}

fn get_all_methods_impl(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class: Arc<RuntimeClass<'gc_life>>, res: &mut Vec<(Arc<RuntimeClass<'gc_life>>, u16)>, include_interface: bool) -> Result<(), WasException> {
    class.view().methods().for_each(|m| {
        res.push((class.clone(), m.method_i()));
    });
    match class.view().super_name() {
        None => {
            let object = check_initing_or_inited_class(jvm, int_state, ClassName::object().into())?;
            object.view().methods().for_each(|m| {
                res.push((object.clone(), m.method_i()));
            });
        }
        Some(super_name) => {
            let super_ = check_initing_or_inited_class(jvm, int_state, super_name.into())?;
            get_all_methods_impl(jvm, int_state, super_, res, include_interface)?;
        }
    }
    if include_interface {
        let view = class.view();
        let interfaces = view.interfaces();
        for interface in interfaces {
            let interface = check_initing_or_inited_class(jvm, int_state, interface.interface_name().into())?;
            interface.view().methods().for_each(|m| {
                res.push((interface.clone(), m.method_i()));
            });
        }
    }
    Ok(())
}

pub fn get_all_fields(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class: Arc<RuntimeClass<'gc_life>>, include_interface: bool) -> Result<Vec<(Arc<RuntimeClass<'gc_life>>, usize)>, WasException> {
    let mut res = vec![];
    get_all_fields_impl(jvm, int_state, class, &mut res, include_interface)?;
    Ok(res)
}

fn get_all_fields_impl(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class: Arc<RuntimeClass<'gc_life>>, res: &mut Vec<(Arc<RuntimeClass<'gc_life>>, usize)>, include_interface: bool) -> Result<(), WasException> {
    class.view().fields().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });

    match class.view().super_name() {
        None => {
            let object = check_initing_or_inited_class(jvm, int_state, ClassName::object().into())?;
            object.view().fields().enumerate().for_each(|(i, _)| {
                res.push((object.clone(), i));
            });
        }
        Some(super_name) => {
            let super_ = check_initing_or_inited_class(jvm, int_state, super_name.into())?;
            get_all_fields_impl(jvm, int_state, super_, res, include_interface)?
        }
    }

    if include_interface {
        for interface in class.view().interfaces() {
            let interface = check_initing_or_inited_class(jvm, int_state, interface.interface_name().into())?;
            interface.view().fields().enumerate().for_each(|(i, _)| {
                res.push((interface.clone(), i));
            });
        }
    }
    Ok(())
}

