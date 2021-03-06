use std::sync::Arc;

use classfile_view::view::constant_info_view::{ClassPoolElemView, ConstantInfoView, StringView};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::MethodDescriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::assert_inited_or_initing_class;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::find_target_method;
use crate::interpreter::{run_function, WasException};
use crate::interpreter_util::push_new_object;
use crate::java::lang::string::JString;
use crate::java_values::{ArrayObject, JavaValue, Object};
use crate::rust_jni::interface::string::intern_safe;

fn load_class_constant(state: &JVMState, int_state: &mut InterpreterStateGuard, c: &ClassPoolElemView) -> Result<(), WasException> {
    let res_class_name = c.class_ref_type();
    let type_ = PTypeView::Ref(res_class_name);
    load_class_constant_by_type(state, int_state, type_)?;
    Ok(())
}

pub fn load_class_constant_by_type(jvm: &JVMState, int_state: &mut InterpreterStateGuard, res_class_type: PTypeView) -> Result<(), WasException> {
    let object = get_or_create_class_object(jvm, res_class_type, int_state)?;
    int_state.current_frame_mut().push(JavaValue::Object(object.into()));
    Ok(())
}

fn load_string_constant(jvm: &JVMState, int_state: &mut InterpreterStateGuard, s: &StringView) {
    let res_string = s.string();
    assert!(int_state.throw().is_none());
    let before_intern = JString::from_rust(jvm, int_state, res_string).expect("todo");
    let string = intern_safe(jvm, before_intern.object().into());
    int_state.push_current_operand_stack(string.java_value());
}

pub fn create_string_on_stack(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard, res_string: String) -> Result<(), WasException> {
    let java_lang_string = ClassName::string();
    let string_class = assert_inited_or_initing_class(
        jvm,
        interpreter_state,
        java_lang_string.into(),
    );
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue> = str_as_vec.map(|x| { JavaValue::Char(x as u16) }).collect();
    push_new_object(jvm, interpreter_state, &string_class);
    let string_object = interpreter_state.pop_current_operand_stack();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
        jvm,
        interpreter_state,
        chars,
        PTypeView::CharType,
        jvm.thread_state.new_monitor("monitor for a string".to_string()),
    )?)))));
    let char_array_type = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::CharType.into()));
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type.to_ptype()], return_type: PTypeView::VoidType.to_ptype() };
    let (constructor_i, final_target_class) = find_target_method(jvm, interpreter_state, "<init>".to_string(), &expected_descriptor, string_class);
    let next_entry = StackEntry::new_java_frame(jvm, final_target_class, constructor_i as u16, args);
    let function_call_frame = interpreter_state.push_frame(next_entry);
    match run_function(jvm, interpreter_state) {
        Ok(_) => {}
        Err(_) => todo!()
    }
    let was_exception = interpreter_state.throw().is_some();
    interpreter_state.pop_frame(jvm, function_call_frame, was_exception);
    if interpreter_state.throw().is_some() {
        unimplemented!()
    }
    let function_return = interpreter_state.function_return_mut();
    if *function_return {
        *function_return = false;
    }
    interpreter_state.push_current_operand_stack(JavaValue::Object(string_object.unwrap_object()));
    Ok(())
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
            let string_value = intern_safe(jvm, JString::from_rust(jvm, int_state, s.string()).expect("todo").object().into()).java_value();
            int_state.push_current_operand_stack(string_value)
        }
        ConstantInfoView::Class(c) => match load_class_constant(jvm, int_state, &c) {
            Err(WasException {}) => {
                return;
            }
            Ok(()) => {}
        },
        ConstantInfoView::Float(f) => {
            let float: f32 = f.float;
            int_state.push_current_operand_stack(JavaValue::Float(float));
        }
        ConstantInfoView::Integer(i) => {
            let int: i32 = i.int;
            int_state.push_current_operand_stack(JavaValue::Int(int));
        }
        _ => {
            dbg!(cp);
            int_state.debug_print_stack_trace();
            dbg!(&pool_entry);
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
            let string_value = int_state.pop_current_operand_stack();
            intern_safe(jvm, string_value.cast_string().unwrap().object().into()).java_value()
        }
        _ => panic!()
    }
}