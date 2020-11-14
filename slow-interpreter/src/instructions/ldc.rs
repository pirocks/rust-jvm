use std::sync::Arc;

use classfile_view::view::constant_info_view::{ClassPoolElemView, ConstantInfoView, StringView};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::find_target_method;
use crate::interpreter::run_function;
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::java::lang::string::JString;
use crate::java_values::{ArrayObject, JavaValue, Object};

fn load_class_constant(state: &JVMState, int_state: &mut InterpreterStateGuard, c: &ClassPoolElemView) {
    let res_class_name = c.class_name();
    let type_ = PTypeView::Ref(res_class_name);
    load_class_constant_by_type(state, int_state, &type_);
}

pub fn load_class_constant_by_type(jvm: &JVMState, int_state: &mut InterpreterStateGuard, res_class_type: &PTypeView) {
    let object = get_or_create_class_object(jvm, res_class_type, int_state, jvm.bootstrap_loader.clone());
    // dbg!(object.clone().lookup_field("name"));
    // dbg!(object.clone());
    // dbg!(object.unwrap_normal_object().fields.borrow());
    int_state.current_frame_mut().push(JavaValue::Object(object.into()));
}

fn load_string_constant(jvm: &JVMState, int_state: &mut InterpreterStateGuard, s: &StringView) {
    let res_string = s.string();
    let string = JString::from_rust(jvm, int_state, res_string).intern(jvm, int_state).java_value();
    int_state.push_current_operand_stack(string);
}

pub fn create_string_on_stack(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard, res_string: String) {
    let java_lang_string = ClassName::string();
    let current_loader = jvm.bootstrap_loader.clone();
    let string_class = check_inited_class(
        jvm,
        interpreter_state,
        &java_lang_string.into(),
        current_loader.clone(),
    );
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue> = str_as_vec.map(|x| { JavaValue::Char(x as u16) }).collect();
    push_new_object(jvm, interpreter_state, &string_class, None);//todo what if stack overflows here?
    let string_object = interpreter_state.pop_current_operand_stack();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
        jvm,
        interpreter_state,
        chars,
        PTypeView::CharType,
        jvm.thread_state.new_monitor("monitor for a string".to_string()),
    ))))));
    let char_array_type = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::CharType.into()));
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type.to_ptype()], return_type: PTypeView::VoidType.to_ptype() };
    let (constructor_i, final_target_class) = find_target_method(jvm, current_loader.clone(), "<init>".to_string(), &expected_descriptor, string_class);
    let next_entry = StackEntry::new_java_frame(final_target_class, constructor_i as u16, args);
    let function_call_frame = interpreter_state.push_frame(next_entry);
    run_function(jvm, interpreter_state);
    interpreter_state.pop_frame(function_call_frame);
    if interpreter_state.throw().is_some() || *interpreter_state.terminate() {
        unimplemented!()
    }
    let function_return = interpreter_state.function_return_mut();
    if *function_return {
        *function_return = false;
    }
    // let interned = unsafe {
    //     from_object(intern_impl(new_local_ref_public(, interpreter_state)))
    // };
    interpreter_state.push_current_operand_stack(JavaValue::Object(string_object.unwrap_object()));
}

pub fn ldc2_w(current_frame: &mut StackEntry, cp: u16) {
    let view = current_frame.class_pointer().view();
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


pub fn ldc_w(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let view = int_state.current_class_view().clone();
    let pool_entry = &view.constant_pool_view(cp as usize);
    match &pool_entry {
        ConstantInfoView::String(s) => {
            let string_value = JString::from_rust(jvm, int_state, s.string()).java_value();
            int_state.push_current_operand_stack(string_value)
        }
        ConstantInfoView::Class(c) => load_class_constant(jvm, int_state, &c),
        ConstantInfoView::Float(f) => {
            let float: f32 = f.float;
            int_state.push_current_operand_stack(JavaValue::Float(float));
        }
        ConstantInfoView::Integer(i) => {
            let int: i32 = i.int;
            int_state.push_current_operand_stack(JavaValue::Int(int));
        }
        _ => {
            // dbg!(&pool_entry.kind);
            unimplemented!()
        }
    }
}

pub fn from_constant_pool_entry(c: &ConstantInfoView, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> JavaValue {
    match &c {
        ConstantInfoView::Integer(i) => JavaValue::Int(i.int),
        ConstantInfoView::Float(f) => JavaValue::Float(f.float),
        ConstantInfoView::Long(l) => JavaValue::Long(l.long),
        ConstantInfoView::Double(d) => JavaValue::Double(d.double),
        ConstantInfoView::String(s) => {
            load_string_constant(jvm, int_state, s);
            int_state.pop_current_operand_stack()
        }
        _ => panic!()
    }
}