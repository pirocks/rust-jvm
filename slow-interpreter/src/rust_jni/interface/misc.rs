use std::ffi::CStr;
use std::intrinsics::transmute;
use std::ops::Deref;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{JavaVM, jboolean, jclass, jint, jmethodID, JNIEnv, JNIInvokeInterface_, jobject, jthrowable};
use rust_jvm_common::classnames::ClassName;
use verification::verifier::filecorrectness::is_assignable;
use verification::VerifierContext;

use crate::instructions::invoke::special::invoke_special_impl;
use crate::instructions::ldc::load_class_constant_by_type;
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::invoke_interface::get_invoke_interface;
use crate::java_values::JavaValue;
use crate::method_table::MethodId;
use crate::rust_jni::native_util::{from_jclass, from_object, get_frame, get_state, to_object};

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
    let jvm = get_state(env);
    let super_name = match from_jclass(sub).as_runtime_class().view().super_name() {
        None => { return to_object(None); }
        Some(n) => n,
    };
//    frame.print_stack_trace();
    let _inited_class = check_inited_class(jvm, &super_name.clone().into(), frame.class_pointer.loader(jvm).clone());
    load_class_constant_by_type(jvm, &frame, &PTypeView::Ref(ReferenceTypeView::Class(super_name)));
    to_object(frame.pop().unwrap_object())
}


pub unsafe extern "C" fn is_assignable_from(env: *mut JNIEnv, sub: jclass, sup: jclass) -> jboolean {
    //todo impl later
    let jvm = get_state(env);
    let frame = get_frame(env);

    let sub_not_null = from_object(sub).unwrap();
    let sup_not_null = from_object(sup).unwrap();

    let sub_type = JavaValue::Object(sub_not_null.into()).cast_class().as_type();
    let sup_type = JavaValue::Object(sup_not_null.into()).cast_class().as_type();

    let loader = &frame.class_pointer.loader(jvm);
    let sub_vtype = sub_type.to_verification_type(loader);
    let sup_vtype = sup_type.to_verification_type(loader);


    let vf = VerifierContext { live_pool_getter: jvm.get_live_object_pool_getter(), bootstrap_loader: jvm.bootstrap_loader.clone() };
    let res = is_assignable(&vf, &sub_vtype, &sup_vtype).map(|_| true).unwrap_or(false);
    res as jboolean
}

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ::va_list::VaList) -> jobject {
    //todo dup
    let method_id: MethodId = transmute(jmethod_id );
    let jvm = get_state(env);
    let frame_temp = get_frame(env);
    let frame = frame_temp.deref();
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let classview = &class.view();
    let method = &classview.method_view_i(method_i as usize);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(jvm, frame, &class, None);
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
        jvm,
        &frame,
        &parsed,
        method_i as usize,
        class.clone(),
        &classview.method_view_i(method_i as usize),
    );
    to_object(obj.unwrap_object())
}

pub unsafe extern "C" fn new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    let method_id: MethodId = transmute(jmethod_id);
    let jvm = get_state(env);
    let frame_temp = get_frame(env);
    let frame = frame_temp.deref();
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let classview = &class.view();
    let method = &classview.method_view_i(method_i as usize);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(jvm, frame.clone(), &class, None);
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
        jvm,
        &frame,
        &parsed,
        method_i as usize,
        class.clone(),
        &classview.method_view_i(method_i as usize),
    );
    to_object(obj.unwrap_object())
}


pub unsafe extern "C" fn get_java_vm(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    //todo get rid of this transmute
    let state = get_state(env);
    let interface = get_invoke_interface(state);
    *vm = Box::into_raw(Box::new(transmute::<_, *mut JNIInvokeInterface_>(Box::leak(Box::new(interface)))));//todo do something about this leak
    0 as jint
}

pub(crate) unsafe extern "C" fn throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
    let jvm = get_state(env);
    *jvm.thread_state.get_current_thread().interpreter_state.throw.write().unwrap() = from_object(obj);
    0 as jint
}