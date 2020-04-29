use crate::rust_jni::native_util::{from_object, get_state, get_frame, to_object};
use jvmti_jni_bindings::{jobject, jboolean, jclass, JNIEnv, jmethodID, jint, JavaVM, JNIInvokeInterface_, jthrowable};
use crate::interpreter_util::{push_new_object, check_inited_class};
use crate::rust_jni::MethodId;
use std::ffi::CStr;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::rust_jni::interface::util::runtime_class_from_object;


use rust_jvm_common::classnames::ClassName;
use std::intrinsics::transmute;
use crate::instructions::invoke::special::invoke_special_impl;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::java_values::JavaValue;
use verification::verifier::filecorrectness::is_assignable;
use verification::VerifierContext;
use crate::invoke_interface::get_invoke_interface;
use std::ops::Deref;

pub unsafe extern "C" fn ensure_local_capacity(_env: *mut JNIEnv, _capacity: jint) -> jint {
    //we always have ram. todo
    0 as jint
}

pub unsafe extern "C" fn find_class(env: *mut JNIEnv, c_name: *const ::std::os::raw::c_char) -> jclass {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let state = get_state(env);
    let frame = get_frame(env);
    //todo maybe parse?
    load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str(name))));
    let obj = frame.pop().unwrap_object();
    to_object(obj)
}


pub unsafe extern "C" fn get_superclass(env: *mut JNIEnv, sub: jclass) -> jclass {
    let frame = get_frame(env);
    let state = get_state(env);
    let super_name = match runtime_class_from_object(sub,state,&frame).unwrap().classfile.super_class_name() {
        None => { return to_object(None); }
        Some(n) => n,
    };
//    frame.print_stack_trace();
    let _inited_class = check_inited_class(state, &super_name, frame.class_pointer.loader.clone());
    load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(super_name)));
    to_object(frame.pop().unwrap_object())
}


pub unsafe extern "C" fn is_assignable_from(env: *mut JNIEnv, sub: jclass, sup: jclass) -> jboolean {
    //todo impl later
    let state = get_state(env);
    let frame  = get_frame(env);

    let sub_not_null = from_object(sub).unwrap();
    let sup_not_null = from_object(sup).unwrap();
    let sub_temp_refcell = sub_not_null.unwrap_normal_object().class_object_ptype.clone();
    let sup_temp_refcell = sup_not_null.unwrap_normal_object().class_object_ptype.clone();

    let sub_type = sub_temp_refcell.as_ref().unwrap();
    let sup_type = sup_temp_refcell.as_ref().unwrap();

    let loader = &frame.class_pointer.loader;
    let sub_vtype = sub_type.to_verification_type(loader);
    let sup_vtype = sup_type.to_verification_type(loader);



    let vf = VerifierContext { live_pool_getter: state.get_live_object_pool_getter(), bootstrap_loader: state.bootstrap_loader.clone() };
    let res = is_assignable(&vf, &sub_vtype, &sup_vtype).map(|_|true).unwrap_or(false);
    res as jboolean
}

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ::va_list::VaList) -> jobject {
    //todo dup
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame_temp = state.get_current_frame();
    let frame = frame_temp.deref();
    let classview = &method_id.class.class_view;
    let method = &classview.method_view_i(method_id.method_i);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(state,frame, &method_id.class);
    let obj = frame.pop();
    frame.push(obj.clone());
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => unimplemented!(),
            PTypeView::CharType => unimplemented!(),
            PTypeView::DoubleType => unimplemented!(),
            PTypeView::FloatType => unimplemented!(),
            PTypeView::IntType => unimplemented!(),
            PTypeView::LongType => unimplemented!(),
            PTypeView::Ref(_) => {
                let native_object: jobject = transmute(l.get::<usize>());
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            PTypeView::ShortType => unimplemented!(),
            PTypeView::BooleanType => unimplemented!(),
            PTypeView::VoidType => unimplemented!(),
            PTypeView::TopType => unimplemented!(),
            PTypeView::NullType => unimplemented!(),
            PTypeView::Uninitialized(_) => unimplemented!(),
            PTypeView::UninitializedThis => unimplemented!(),
            PTypeView::UninitializedThisOrClass(_) => panic!()
        }
    }
    invoke_special_impl(
        state,
        &frame,
        &parsed,
        method_id.method_i,
        method_id.class.clone(),
        &classview.method_view_i(method_id.method_i),
    );
    to_object(obj.unwrap_object())
}

pub unsafe extern "C" fn new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame_temp = get_frame(env);
    let frame = frame_temp.deref();
    let classview = &method_id.class.class_view;
    let method = &classview.method_view_i(method_id.method_i);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(state,frame.clone(), &method_id.class);
    let obj = frame.pop();
    frame.push(obj.clone());
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => unimplemented!(),
            PTypeView::CharType => unimplemented!(),
            PTypeView::DoubleType => unimplemented!(),
            PTypeView::FloatType => unimplemented!(),
            PTypeView::IntType => unimplemented!(),
            PTypeView::LongType => unimplemented!(),
            PTypeView::Ref(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            PTypeView::ShortType => unimplemented!(),
            PTypeView::BooleanType => unimplemented!(),
            PTypeView::VoidType => unimplemented!(),
            PTypeView::TopType => unimplemented!(),
            PTypeView::NullType => unimplemented!(),
            PTypeView::Uninitialized(_) => unimplemented!(),
            PTypeView::UninitializedThis => unimplemented!(),
            PTypeView::UninitializedThisOrClass(_) => panic!()
        }
    }
    invoke_special_impl(
        state,
        &frame,
        &parsed,
        method_id.method_i,
        method_id.class.clone(),
        &classview.method_view_i(method_id.method_i),
    );
    to_object(obj.unwrap_object())
}


pub unsafe extern "C" fn get_java_vm(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    //todo get rid of this transmute
    let state = get_state(env);
    let interface = get_invoke_interface(state);
    *vm = Box::into_raw(Box::new(transmute::<_,*mut JNIInvokeInterface_>(Box::leak(Box::new(interface)))));//todo do something about this leak
    0 as jint
}

pub(crate) unsafe extern "C" fn throw(env: *mut JNIEnv, obj: jthrowable) -> jint{
    let state = get_state(env);
    state.get_current_thread().interpreter_state.throw.replace(from_object(obj));
    0 as jint
}