use crate::instructions::invoke::{invoke_virtual_method_i, invoke_static_impl};
use crate::rust_jni::native_util::{to_object, get_state, get_frame, from_object};
use runtime_common::java_values::JavaValue;
use rust_jvm_common::classfile::ACC_STATIC;
use crate::rust_jni::MethodId;
use jni_bindings::{JNIEnv, jobject, jmethodID, jclass, JNINativeInterface_, jboolean};
use std::ffi::VaList;
use std::rc::Rc;
use runtime_common::StackEntry;
use log::trace;
use descriptor_parser::{parse_method_descriptor, MethodDescriptor};
use rust_jvm_common::view::ptype_view::PTypeView;

#[no_mangle]
pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
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
        match type_ {
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
            PTypeView::UninitializedThisOrClass(_) => panic!(),
        }
    }
    //todo add params into operand stack;
    trace!("----NATIVE EXIT ----");
    invoke_virtual_method_i(state, frame.clone(), parsed, method_id.class.clone(), method_id.method_i, method);
    trace!("----NATIVE ENTER ----");
    let res = frame.pop().unwrap_object();
    to_object(res)
}

pub unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: VaList) -> jobject {
    let frame = call_static_method_v(env, jmethod_id, &mut l);
    let res = frame.pop().unwrap_object();
    to_object(res)
}

pub unsafe fn call_static_method_v(env: *mut *const JNINativeInterface_, jmethod_id: jmethodID, l: &mut VaList) -> Rc<StackEntry> {
    let method_id = (jmethod_id as *mut MethodId).as_ref().unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
    let classfile = &method_id.class.classfile;
    let method = &classfile.methods[method_id.method_i];
    let method_descriptor_str = method.descriptor_str(classfile);
    let _name = method.method_name(classfile);
    let parsed = parse_method_descriptor(method_descriptor_str.as_str()).unwrap();
//todo dup
    push_params_onto_frame(l, &frame, &parsed);
    trace!("----NATIVE EXIT ----");
    invoke_static_impl(state, frame.clone(), parsed, method_id.class.clone(), method_id.method_i, method);
    trace!("----NATIVE ENTER----");
    frame
}

unsafe fn push_params_onto_frame(l: &mut VaList, frame: &Rc<StackEntry>, parsed: &MethodDescriptor) {
    for type_ in &parsed.parameter_types {
        match type_ {
            PTypeView::ByteType => unimplemented!(),
            PTypeView::CharType => unimplemented!(),
            PTypeView::DoubleType => unimplemented!(),
            PTypeView::FloatType => unimplemented!(),
            PTypeView::IntType => unimplemented!(),
            PTypeView::LongType => unimplemented!(),
            PTypeView::Ref(_) => {
                //todo dup with other line
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
}

pub unsafe extern "C" fn call_static_boolean_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jboolean {
    call_static_method_v(env, method_id, &mut l);
    let res = get_frame(env).pop();
    res.unwrap_int() as jboolean
}
