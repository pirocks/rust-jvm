use crate::rust_jni::native_util::{from_object, get_state, get_frame, to_object};
use jni_bindings::{jobject, jboolean, jclass, JNIEnv, jmethodID, jint, JavaVM, JNIInvokeInterface_, jthrowable};
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
use descriptor_parser::parse_method_descriptor;

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
    let _inited_class = check_inited_class(state, &super_name, frame.clone().into(), frame.class_pointer.loader.clone());
    load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(super_name)));
    to_object(frame.pop().unwrap_object())
}


pub unsafe extern "C" fn is_assignable_from(_env: *mut JNIEnv, _sub: jclass, _sup: jclass) -> jboolean {
    //todo impl later
    true as jboolean
}

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ::va_list::VaList) -> jobject {
    //todo dup
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
    let classfile = &method_id.class.classfile;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = method.descriptor_str(classfile);
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(method_descriptor_str.as_str()).unwrap();
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
        &classfile.methods[method_id.method_i],
    );
    to_object(obj.unwrap_object())
}

pub unsafe extern "C" fn new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
    let classfile = &method_id.class.classfile;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = method.descriptor_str(classfile);
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(method_descriptor_str.as_str()).unwrap();
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
        &classfile.methods[method_id.method_i],
    );
    to_object(obj.unwrap_object())
}


pub unsafe extern "C" fn get_java_vm(_env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint {
    *vm = Box::into_raw(Box::new(Box::leak(Box::new(JNIInvokeInterface_ {
        reserved0: std::ptr::null_mut(),
        reserved1: std::ptr::null_mut(),
        reserved2: std::ptr::null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: None,
        AttachCurrentThreadAsDaemon: None,
    }))));
    0 as jint
}

pub(crate) unsafe extern "C" fn throw(env: *mut JNIEnv, obj: jthrowable) -> jint{
    let state = get_state(env);
    state.throw = from_object(obj);
    0 as jint
}