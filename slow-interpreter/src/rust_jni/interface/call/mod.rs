use std::ffi::{VaList, VaListImpl};
use std::mem::transmute;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use jvmti_jni_bindings::{jboolean, jint, jlong, jmethodID, JNINativeInterface_, jobject, jshort, jvalue};
use rust_jvm_common::ptype::PType;

// use log::trace;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::java_values::JavaValue;
use crate::method_table::MethodId;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state};
use crate::StackEntry;

pub mod call_nonstatic;

unsafe fn call_nonstatic_method<'l>(env: *mut *const JNINativeInterface_, obj: jobject, method_id: jmethodID, mut l: VarargProvider) -> Option<JavaValue> {
    let method_id: MethodId = transmute(method_id);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let classview = class.view().clone();
    let method = &classview.method_view_i(method_i as usize);
    if method.is_static() {
        unimplemented!()
    }
    let parsed = method.desc();
    int_state.push_current_operand_stack(JavaValue::Object(from_object(obj)));
    //todo ducplication with push_params_onto_frame
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => {
                int_state.push_current_operand_stack(JavaValue::Byte(l.arg_byte()))
            }
            PTypeView::CharType => {
                int_state.push_current_operand_stack(JavaValue::Char(l.arg_char()))
            }
            PTypeView::DoubleType => {
                int_state.push_current_operand_stack(JavaValue::Double(l.arg_double()))
            }
            PTypeView::FloatType => {
                int_state.push_current_operand_stack(JavaValue::Float(l.arg_float()))
            }
            PTypeView::IntType => {
                int_state.push_current_operand_stack(JavaValue::Int(l.arg_int()))
            }
            PTypeView::LongType => {
                int_state.push_current_operand_stack(JavaValue::Long(l.arg_long()))
            }
            PTypeView::Ref(_) => {
                let native_object: jobject = l.arg_ptr() as jobject;
                let o = from_object(native_object);
                int_state.push_current_operand_stack(JavaValue::Object(o));
            }
            PTypeView::ShortType => {
                int_state.push_current_operand_stack(JavaValue::Short(l.arg_short()))
            }
            PTypeView::BooleanType => {
                int_state.push_current_operand_stack(JavaValue::Boolean(l.arg_bool()))
            }
            PTypeView::VoidType => unimplemented!(),
            PTypeView::TopType => unimplemented!(),
            PTypeView::NullType => unimplemented!(),
            PTypeView::Uninitialized(_) => unimplemented!(),
            PTypeView::UninitializedThis => unimplemented!(),
            PTypeView::UninitializedThisOrClass(_) => panic!(),
        }
    }
    invoke_virtual_method_i(jvm, int_state, parsed, class.clone(), method_i as usize, &method, false);
    assert!(int_state.throw().is_none());//todo
    if method.desc().return_type == PType::VoidType {
        None
    } else {
        int_state.pop_current_operand_stack().into()
    }
}

pub unsafe fn call_static_method_impl(env: *mut *const JNINativeInterface_, jmethod_id: jmethodID, mut l: VarargProvider) -> Option<JavaValue> {
    let method_id = *(jmethod_id as *mut MethodId);
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let classfile = &class.view();
    let method = &classfile.method_view_i(method_i as usize);
    let method_descriptor_str = method.desc_str();
    let _name = method.name();
    let parsed = parse_method_descriptor(method_descriptor_str.as_str()).unwrap();
//todo dup
    push_params_onto_frame(&mut l, int_state.current_frame_mut(), &parsed);
    // trace!("----NATIVE EXIT ----");
    invoke_static_impl(jvm, int_state, parsed, class.clone(), method_i as usize, method.method_info());
    // trace!("----NATIVE ENTER----");
    if method.desc().return_type == PType::VoidType {
        None
    } else {
        int_state.pop_current_operand_stack().into()
    }
}

unsafe fn push_params_onto_frame(
    l: &mut VarargProvider,
    frame: &mut StackEntry,
    parsed: &MethodDescriptor,
) {
    for type_ in &parsed.parameter_types {
        match PTypeView::from_ptype(type_) {
            PTypeView::ByteType => {
                frame.push(JavaValue::Byte(l.arg_byte()));
            }
            PTypeView::CharType => {
                frame.push(JavaValue::Char(l.arg_char()));
            }
            PTypeView::DoubleType => {
                frame.push(JavaValue::Double(l.arg_double()));
            }
            PTypeView::FloatType => {
                frame.push(JavaValue::Float(l.arg_float()));
            }
            PTypeView::IntType => {
                frame.push(JavaValue::Int(l.arg_int()));
            }
            PTypeView::LongType => {
                frame.push(JavaValue::Long(l.arg_long()));
            }
            PTypeView::Ref(_) => {
                //todo dup with other line
                let native_object: jobject = l.arg_ptr() as jobject;
                let o = from_object(native_object);
                frame.push(JavaValue::Object(o));
            }
            PTypeView::ShortType => {
                frame.push(JavaValue::Short(l.arg_short()));
            }
            PTypeView::BooleanType => {
                frame.push(JavaValue::Boolean(l.arg_bool()));
            }
            PTypeView::VoidType => unimplemented!(),
            PTypeView::TopType |
            PTypeView::NullType |
            PTypeView::Uninitialized(_) |
            PTypeView::UninitializedThis |
            PTypeView::UninitializedThisOrClass(_) => panic!()
        }
    }
}

pub mod call_static;

pub enum VarargProvider<'l, 'l2, 'l3> {
    Dots(&'l mut VaListImpl<'l2>),
    VaList(&'l mut VaList<'l2, 'l3>),
    Array(*const jvalue),
}

impl VarargProvider<'_, '_, '_> {
    pub unsafe fn arg_ptr(&mut self) -> jobject {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                let res = (**a_ptr).l;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }
    pub unsafe fn arg_bool(&mut self) -> jboolean {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).z;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }
    pub unsafe fn arg_short(&mut self) -> jshort {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).s;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_long(&mut self) -> jlong {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).j;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_int(&mut self) -> jint {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).i;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }


    pub unsafe fn arg_float(&mut self) -> f32 {
        match self {
            VarargProvider::Dots(l) => transmute(l.arg::<u32>()),
            VarargProvider::VaList(l) => transmute(l.arg::<u32>()),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).f;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_double(&mut self) -> f64 {
        match self {
            VarargProvider::Dots(l) => transmute(l.arg::<u64>()),
            VarargProvider::VaList(l) => transmute(l.arg::<u64>()),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).d;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_byte(&mut self) -> i8 {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).b;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }

    pub unsafe fn arg_char(&mut self) -> u16 {
        match self {
            VarargProvider::Dots(l) => l.arg(),
            VarargProvider::VaList(l) => l.arg(),
            VarargProvider::Array(a_ptr) => {
                //todo duplication
                let res = (**a_ptr).c;
                *a_ptr = a_ptr.offset(1);
                res
            }
        }
    }
}