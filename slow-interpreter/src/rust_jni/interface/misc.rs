use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{JavaVM, jboolean, jclass, jint, JNI_FALSE, JNI_TRUE, JNIEnv, JNIInvokeInterface_, JNINativeMethod, jobject};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classnames::ClassName;
use verification::verifier::filecorrectness::is_assignable;
use verification::VerifierContext;

use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter_state::InterpreterStateGuard;
use crate::interpreter_util::check_inited_class;
use crate::invoke_interface::get_invoke_interface;
use crate::java_values::JavaValue;
use crate::jvm_state::{JVMState, LibJavaLoading};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};

pub unsafe extern "C" fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    //we always have ram. todo
    0 as jint
}

pub unsafe extern "C" fn find_class(env: *mut JNIEnv, c_name: *const ::std::os::raw::c_char) -> jclass {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    //todo maybe parse?
    load_class_constant_by_type(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str(name))));
    let obj = int_state.pop_current_operand_stack().unwrap_object();
    new_local_ref_public(obj, int_state)
}


pub unsafe extern "C" fn get_superclass(env: *mut JNIEnv, sub: jclass) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let super_name = match from_jclass(sub).as_runtime_class().view().super_name() {
        None => return null_mut(),
        Some(n) => n,
    };
    let _inited_class = check_inited_class(jvm, int_state, &super_name.clone().into(), int_state.current_loader(jvm));
    load_class_constant_by_type(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Class(super_name)));
    new_local_ref_public(int_state.pop_current_operand_stack().unwrap_object(), int_state)
}


pub unsafe extern "C" fn is_assignable_from(env: *mut JNIEnv, sub: jclass, sup: jclass) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    // let frame = int_state.current_frame_mut();

    let sub_not_null = from_object(sub).unwrap();
    let sup_not_null = from_object(sup).unwrap();

    let sub_type = JavaValue::Object(sub_not_null.into()).cast_class().as_type();
    let sup_type = JavaValue::Object(sup_not_null.into()).cast_class().as_type();

    let loader = &int_state.current_loader(jvm);
    let sub_vtype = sub_type.to_verification_type(loader);
    let sup_vtype = sup_type.to_verification_type(loader);


    let vf = VerifierContext { live_pool_getter: jvm.get_live_object_pool_getter(), bootstrap_loader: jvm.bootstrap_loader.clone() };
    let res = is_assignable(&vf, &sub_vtype, &sup_vtype).map(|_| true).unwrap_or(false);
    res as jboolean
}


pub unsafe extern "C" fn get_java_vm(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    //todo get rid of this transmute
    let state = get_state(env);
    let int_state = get_interpreter_state(env);//todo maybe this should have an optionable version
    let interface = get_invoke_interface(state, int_state);
    *vm = Box::into_raw(Box::new(transmute::<_, *mut JNIInvokeInterface_>(Box::leak(Box::new(interface)))));//todo do something about this leak
    0 as jint
}


pub unsafe extern "C" fn is_same_object(_env: *mut JNIEnv, obj1: jobject, obj2: jobject) -> jboolean {
    let _1 = from_object(obj1);
    let _2 = from_object(obj2);
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
                Some(_2_) => Arc::ptr_eq(&_1_, &_2_) as u32,
            }
        }
    }) as u8
}

pub unsafe extern "C" fn register_natives(env: *mut JNIEnv,
                                          clazz: jclass,
                                          methods: *const JNINativeMethod,
                                          n_methods: jint) -> jint {
    // println!("Call to register_natives, n_methods: {}", n_methods);
    let jvm = get_state(env);
    for to_register_i in 0..n_methods {
        let method = *methods.offset(to_register_i as isize);
        let expected_name: String = CStr::from_ptr(method.name).to_str().unwrap().to_string().clone();
        let descriptor: String = CStr::from_ptr(method.signature).to_str().unwrap().to_string().clone();
        let runtime_class: Arc<RuntimeClass> = from_jclass(clazz).as_runtime_class();
        let jni_context = &jvm.libjava;
        let view = &runtime_class.view();
        &view.methods().enumerate().for_each(|(i, method_info)| {
            let descriptor_str = method_info.desc_str();
            let current_name = method_info.name();
            if current_name == expected_name && descriptor == descriptor_str {
                jvm.tracing.trace_jni_register(&view.name(), expected_name.as_str());
                register_native_with_lib_java_loading(jni_context, &method, &runtime_class, i)
            }
        });
    }
    0
}


fn register_native_with_lib_java_loading(jni_context: &LibJavaLoading, method: &JNINativeMethod, runtime_class: &Arc<RuntimeClass>, method_i: usize) -> () {
    if jni_context.registered_natives.read().unwrap().contains_key(runtime_class) {
        unsafe {
            jni_context.registered_natives
                .read().unwrap()
                .get(runtime_class)
                .unwrap()
                .write().unwrap()
                .insert(method_i as CPIndex, transmute(method.fnPtr));
        }
    } else {
        let mut map = HashMap::new();
        map.insert(method_i as CPIndex, unsafe { transmute(method.fnPtr) });
        jni_context.registered_natives.write().unwrap().insert(runtime_class.clone(), RwLock::new(map));
    }
}


pub fn get_all_methods<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>) -> Vec<(Arc<RuntimeClass>, usize)> {
    let mut res = vec![];
    // dbg!(&class.class_view.name());
    class.view().methods().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });
    if class.view().super_name().is_none() {
        let object = check_inited_class(jvm, int_state, &ClassName::object().into(), class.loader(jvm).clone());
        object.view().methods().enumerate().for_each(|(i, _)| {
            res.push((object.clone(), i));
        });
    } else {
        let name = class.view().super_name().unwrap();
        let super_ = check_inited_class(jvm, int_state, &name.into(), class.loader(jvm).clone());
        for (c, i) in get_all_methods(jvm, int_state, super_) {
            res.push((c, i));
        }
    }

    res
}

//todo duplication with methods
pub fn get_all_fields<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: Arc<RuntimeClass>) -> Vec<(Arc<RuntimeClass>, usize)> {
    let mut res = vec![];
    class.view().fields().enumerate().for_each(|(i, _)| {
        res.push((class.clone(), i));
    });
    if class.view().super_name().is_none() {
        let object = check_inited_class(jvm, int_state, &ClassName::object().into(), class.loader(jvm).clone());
        object.view().fields().enumerate().for_each(|(i, _)| {
            res.push((object.clone(), i));
        });
    } else {
        let name = class.view().super_name();
        let super_ = check_inited_class(jvm, int_state, &name.unwrap().into(), class.loader(jvm).clone());
        for (c, i) in get_all_fields(jvm, int_state, super_) {
            res.push((c, i));//todo accidental O(n^2)
        }
    }

    res
}

