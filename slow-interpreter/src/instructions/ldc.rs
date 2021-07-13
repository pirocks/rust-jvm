use classfile_view::view::constant_info_view::{ClassPoolElemView, ConstantInfoView, StringView};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::assert_inited_or_initing_class;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::find_target_method;
use crate::interpreter::{run_function, WasException};
use crate::interpreter_util::push_new_object;
use crate::java::lang::string::JString;
use crate::java_values::{ArrayObject, JavaValue, Object};
use crate::rust_jni::interface::string::intern_safe;
use crate::stack_entry::StackEntryMut;

fn load_class_constant(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, c: &ClassPoolElemView) -> Result<(), WasException> {
    let res_class_name = c.class_ref_type();
    let type_ = CPDType::Ref(res_class_name);
    load_class_constant_by_type(jvm, int_state, type_)?;
    Ok(())
}

pub fn load_class_constant_by_type(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, res_class_type: CPDType) -> Result<(), WasException> {
    let object = get_or_create_class_object(jvm, res_class_type, int_state)?;
    int_state.current_frame_mut().push(JavaValue::Object(object.into()));
    Ok(())
}

fn load_string_constant(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, s: &StringView) {
    let res_string = s.string();
    assert!(int_state.throw().is_none());
    let before_intern = JString::from_rust(jvm, int_state, res_string).expect("todo");
    let string = intern_safe(jvm, before_intern.object().into());
    int_state.push_current_operand_stack(string.java_value());
}

pub fn create_string_on_stack(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, res_string: String) -> Result<(), WasException> {
    let java_lang_string = CClassName::string();
    let string_class = assert_inited_or_initing_class(
        jvm,
        java_lang_string.into(),
    );
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue<'gc_life>> = str_as_vec.map(|x| { JavaValue::Char(x as u16) }).collect();
    push_new_object(jvm, interpreter_state, &string_class);
    let string_object = interpreter_state.pop_current_operand_stack(Some(CClassName::string().into()));
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(
        jvm,
        interpreter_state,
        chars,
        CPDType::CharType,
        jvm.thread_state.new_monitor("monitor for a string".to_string()),
    )?)))));
    let char_array_type = CPDType::Ref(CPRefType::Array(CPDType::CharType.into()));
    let expected_descriptor = CMethodDescriptor { arg_types: vec![char_array_type], return_type: CPDType::VoidType };
    let (constructor_i, final_target_class) = find_target_method(jvm, interpreter_state, MethodName::constructor_init(), &expected_descriptor, string_class);
    let next_entry = StackEntry::new_java_frame(jvm, final_target_class, constructor_i as u16, args);
    let function_call_frame = interpreter_state.push_frame(next_entry, jvm);
    match run_function(jvm, interpreter_state) {
        Ok(_) => {}
        Err(_) => todo!()
    }
    let was_exception = interpreter_state.throw().is_some();
    interpreter_state.pop_frame(jvm, function_call_frame, was_exception);
    if interpreter_state.throw().is_some() {
        unimplemented!()
    }
    if interpreter_state.function_return() {
        interpreter_state.set_function_return(false);
    }
    interpreter_state.push_current_operand_stack(JavaValue::Object(string_object.unwrap_object()));
    Ok(())
}

pub fn ldc2_w(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, cp: u16) {
    let view = current_frame.class_pointer(jvm).view();
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


pub fn ldc_w(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, cp: u16) {
    let view = int_state.current_class_view(jvm).clone();
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
            // dbg!(cp);
            int_state.debug_print_stack_trace(jvm);
            // dbg!(&pool_entry);
            unimplemented!()
        }
    }
}

pub fn from_constant_pool_entry(c: &ConstantInfoView, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> JavaValue<'gc_life> {
    match &c {
        ConstantInfoView::Integer(i) => JavaValue::Int(i.int),
        ConstantInfoView::Float(f) => JavaValue::Float(f.float),
        ConstantInfoView::Long(l) => JavaValue::Long(l.long),
        ConstantInfoView::Double(d) => JavaValue::Double(d.double),
        ConstantInfoView::String(s) => {
            load_string_constant(jvm, int_state, s);
            let string_value = int_state.pop_current_operand_stack(Some(CClassName::string().into()));
            intern_safe(jvm, string_value.cast_string().unwrap().object().into()).java_value()
        }
        _ => panic!()
    }
}