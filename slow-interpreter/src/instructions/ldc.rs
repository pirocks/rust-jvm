use rust_jvm_common::classnames::ClassName;
use crate::{StackEntry, JVMState};
use crate::interpreter_util::{check_inited_class, push_new_object};
use std::sync::Arc;
use crate::instructions::invoke::find_target_method;
use std::cell::RefCell;
use crate::rust_jni::native_util::{to_object, from_object};
use crate::rust_jni::interface::string::intern_impl;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::{JavaValue, Object, ArrayObject};
use descriptor_parser::{MethodDescriptor, parse_field_descriptor};
use crate::class_objects::get_or_create_class_object;
use std::ops::Deref;
use crate::interpreter::run_function;
use classfile_view::view::constant_info_view::{ConstantInfoView, StringView, ClassPoolElemView};
use classfile_view::view::ClassView;


fn load_class_constant(state: &JVMState, current_frame: &StackEntry, c: &ClassPoolElemView) {
    let res_class_name = c.class_name().unwrap_name();
    let type_ = parse_field_descriptor(res_class_name.get_referred_name().as_str()).unwrap().field_type;
    load_class_constant_by_type(state, current_frame, &PTypeView::from_ptype(&type_));
}

pub fn load_class_constant_by_type(jvm: &JVMState, current_frame: &StackEntry, res_class_type: &PTypeView) {
    let object = get_or_create_class_object(jvm, res_class_type, current_frame.clone().into(), current_frame.class_pointer.loader(jvm).clone());
    current_frame.push(JavaValue::Object(object.into()));
}

fn load_string_constant(jvm: &JVMState, s: &StringView) {
    let res_string = s.string();
    create_string_on_stack(jvm, res_string);
}

pub fn create_string_on_stack(jvm: &JVMState, res_string: String) {
    let java_lang_string = ClassName::string();
    let frame_temp = jvm.get_current_frame();
    let current_frame = frame_temp.deref();
    let current_loader = current_frame.class_pointer.loader(jvm).clone();
    let string_class = check_inited_class(
        jvm,
        &java_lang_string,
        current_loader.clone(),
    );
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue> = str_as_vec.map(|x| { JavaValue::Char(x) }).collect();
    push_new_object(jvm, current_frame, &string_class);//todo what if stack overflows here?
    let string_object = current_frame.pop();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject {
        elems: RefCell::new(chars),
        elem_type: PTypeView::CharType,
        monitor: jvm.new_monitor("monitor for a string".to_string()),
    })))));
    let char_array_type = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::CharType.into()));
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type.to_ptype()], return_type: PTypeView::VoidType.to_ptype() };
    let (constructor_i, final_target_class) = find_target_method(jvm, current_loader.clone(), "<init>".to_string(), &expected_descriptor, string_class);
    let next_entry = StackEntry {
        class_pointer: final_target_class,
        method_i: constructor_i as u16,
        local_vars: args.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    }.into();
    jvm.get_current_thread().call_stack.borrow_mut().push(next_entry);
    run_function(jvm);
    jvm.get_current_thread().call_stack.borrow_mut().pop();
    let interpreter_state = &jvm.get_current_thread().interpreter_state;
    if interpreter_state.throw.borrow().is_some() || *interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    if *interpreter_state.function_return.borrow() {
        interpreter_state.function_return.replace(false);
    }
    let interned = unsafe {
        from_object(intern_impl(to_object(string_object.unwrap_object())))
    };
    current_frame.push(JavaValue::Object(interned));
}

pub fn ldc2_w(current_frame: &StackEntry, cp: u16) -> () {
    let view = current_frame.class_pointer.view();
    let pool_entry = &view.constant_pool_view(cp as usize);
    match &pool_entry {
        ConstantInfoView::Long(l) => {
            current_frame.push(JavaValue::Long(l.long));
        }
        ConstantInfoView::Double(d) => {
            current_frame.push(JavaValue::Double(d.double));
        }
        _ => {}
    }
}


pub fn ldc_w(state: &JVMState, current_frame: &StackEntry, cp: u16) -> () {
    let view = &current_frame.class_pointer.view();
    let pool_entry = &view.constant_pool_view(cp as usize);
    match &pool_entry {
        ConstantInfoView::String(s) => load_string_constant(state, &s),
        ConstantInfoView::Class(c) => load_class_constant(state, &current_frame, &c),
        ConstantInfoView::Float(f) => {
            let float: f32 = f.float;
            current_frame.push(JavaValue::Float(float));
        }
        ConstantInfoView::Integer(i) => {
            let int: i32 = i.int;
            current_frame.push(JavaValue::Int(int));
        }
        _ => {
            // dbg!(&pool_entry.kind);
            unimplemented!()
        }
    }
}

pub fn from_constant_pool_entry(class: &ClassView, c: &ConstantInfoView, jvm: &JVMState) -> JavaValue {
    match &c {
        ConstantInfoView::Integer(i) => JavaValue::Int(i.int),
        ConstantInfoView::Float(f) => JavaValue::Float(f.float),
        ConstantInfoView::Long(l) => JavaValue::Long(l.long),
        ConstantInfoView::Double(d) => JavaValue::Double(d.double),
        ConstantInfoView::String(s) => {
            load_string_constant(jvm, s);
            let frame = jvm.get_current_frame();
            frame.pop()
        }
        _ => panic!()
    }
}