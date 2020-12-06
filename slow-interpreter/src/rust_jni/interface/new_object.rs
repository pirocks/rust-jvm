use std::mem::transmute;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jclass, jmethodID, JNIEnv, jobject};

use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter_util::push_new_object;
use crate::java_values::JavaValue;
use crate::method_table::from_jmethod_id;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ::va_list::VaList) -> jobject {
    //todo dup
    let method_id = from_jmethod_id(jmethod_id);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();//todo should return error instead of lookup
    let classview = &class.view();
    let method = &classview.method_view_i(method_i as usize);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(jvm, int_state, &class, None);
    let obj = int_state.pop_current_operand_stack();
    int_state.push_current_operand_stack(obj.clone());
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
                int_state.push_current_operand_stack(JavaValue::Object(o));
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
        int_state,
        &parsed,
        method_i as usize,
        class.clone(),
        &classview.method_view_i(method_i as usize),
    );
    new_local_ref_public(obj.unwrap_object(), int_state)
}

pub unsafe extern "C" fn new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    let method_id = from_jmethod_id(jmethod_id);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let classview = &class.view();
    let method = &classview.method_view_i(method_i as usize);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(jvm, int_state, &class, None);
    let obj = int_state.pop_current_operand_stack();
    int_state.push_current_operand_stack(obj.clone());
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => unimplemented!(),
            PTypeView::CharType => unimplemented!(),
            PTypeView::DoubleType => unimplemented!(),
            PTypeView::FloatType => unimplemented!(),
            PTypeView::IntType => {
                let int: i32 = l.arg();
                int_state.push_current_operand_stack(JavaValue::Int(int))
            },
            PTypeView::LongType => {
                let long: i64 = l.arg();
                int_state.push_current_operand_stack(JavaValue::Long(long))
            }
            PTypeView::Ref(_) => {
                let native_object: jobject = l.arg();
                let o = from_object(native_object);
                int_state.push_current_operand_stack(JavaValue::Object(o));
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
        int_state,
        &parsed,
        method_i as usize,
        class.clone(),
        &classview.method_view_i(method_i as usize),
    );
    new_local_ref_public(obj.unwrap_object(), int_state)
}

