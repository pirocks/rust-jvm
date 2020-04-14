use crate::rust_jni::native_util::{to_object, get_state, get_frame, from_object};
use rust_jvm_common::classfile::ACC_STATIC;
use crate::rust_jni::MethodId;
use jni_bindings::{JNIEnv, jobject, jmethodID, jclass, JNINativeInterface_, jboolean, _jmethodID, _jobject};
use std::ffi::{VaList, VaListImpl, c_void};

use log::trace;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use classfile_view::view::ptype_view::PTypeView;
use crate::java_values::JavaValue;
use crate::StackEntry;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use std::ops::Deref;
use std::rc::Rc;

#[no_mangle]
pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));

    let res = frame.pop().unwrap_object();
    to_object(res)
}

unsafe fn call_nonstatic_method(env: *mut *const JNINativeInterface_, obj: jobject, method_id: jmethodID, mut l: VarargProvider) -> Rc<StackEntry> {
    let method_id = (method_id as *mut MethodId).as_ref().unwrap();
    let classfile = method_id.class.classfile.clone();
    let method = &classfile.methods[method_id.method_i];
    if method.access_flags & ACC_STATIC > 0 {
        unimplemented!()
    }
    let state = get_state(env);
    let frame = get_frame(env);
    let exp_descriptor_str = method.descriptor_str(&classfile);
    let parsed = parse_method_descriptor(exp_descriptor_str.as_str()).unwrap();
    frame.push(JavaValue::Object(from_object(obj)));
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => unimplemented!(),
            PTypeView::CharType => unimplemented!(),
            PTypeView::DoubleType => unimplemented!(),
            PTypeView::FloatType => unimplemented!(),
            PTypeView::IntType => unimplemented!(),
            PTypeView::LongType => unimplemented!(),
            PTypeView::Ref(_) => {
                let native_object: jobject = l.arg_ptr() as jobject;
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            PTypeView::ShortType => unimplemented!(),
            PTypeView::BooleanType => {
                frame.push(JavaValue::Boolean(l.arg_bool() != 0))//todo this erases byte values which a problem here and more generally the bool implementation
            },
            PTypeView::VoidType => unimplemented!(),
            PTypeView::TopType => unimplemented!(),
            PTypeView::NullType => unimplemented!(),
            PTypeView::Uninitialized(_) => unimplemented!(),
            PTypeView::UninitializedThis => unimplemented!(),
            PTypeView::UninitializedThisOrClass(_) => panic!(),
        }
    }
//todo add params into operand stack;
    trace!("----NATIVE EXIT ----");
    invoke_virtual_method_i(state, parsed, method_id.class.clone(), method_id.method_i, method, false);
    trace!("----NATIVE ENTER ----");
    frame
}

pub unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: VaList) -> jobject {
    let frame = call_static_method_impl(env, jmethod_id, VarargProvider::VaList(&mut l));
    let res = frame.pop().unwrap_object();
    to_object(res)
}

pub unsafe fn call_static_method_impl<'l>(env: *mut *const JNINativeInterface_, jmethod_id: jmethodID, mut l: VarargProvider) -> Rc<StackEntry> {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame_rc = get_frame(env);
    let frame = frame_rc.deref();
    let classfile = &method_id.class.classfile;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = method.descriptor_str(classfile);
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(method_descriptor_str.as_str()).unwrap();
//todo dup
    push_params_onto_frame(&mut l, &frame, &parsed);
    trace!("----NATIVE EXIT ----");
    invoke_static_impl(state, parsed, method_id.class.clone(), method_id.method_i, method);
    trace!("----NATIVE ENTER----");
    frame_rc
}

unsafe fn push_params_onto_frame(
    l: &mut VarargProvider,
    frame: &StackEntry,
    parsed: &MethodDescriptor,
) {
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => unimplemented!(),
            PTypeView::CharType => unimplemented!(),
            PTypeView::DoubleType => unimplemented!(),
            PTypeView::FloatType => unimplemented!(),
            PTypeView::IntType => unimplemented!(),
            PTypeView::LongType => unimplemented!(),
            PTypeView::Ref(_) => {
                //todo dup with other line
                let native_object: jobject = l.arg_ptr() as jobject;
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
}

pub unsafe extern "C" fn call_static_boolean_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jboolean {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jboolean
}

pub unsafe extern "C" fn call_static_object_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jobject {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    to_object(res.unwrap_object())
}

pub enum VarargProvider<'l, 'l2, 'l3> {
    Dots(&'l mut VaListImpl<'l2>),
    VaList(&'l mut VaList<'l2, 'l3>),
}

impl VarargProvider<'_, '_, '_> {
    pub unsafe fn arg_ptr(&mut self) -> *mut c_void {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
        }
    }
    pub unsafe fn arg_bool(&mut self) -> u8 {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
        }
    }
}

pub unsafe extern "C" fn call_void_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) {
    /*let frame =*/ call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));

// let res = frame.pop().unwrap_object();
// to_object(res)
}