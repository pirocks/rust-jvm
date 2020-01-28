use runtime_common::{InterpreterState, CallStackEntry};
use std::rc::Rc;
use rust_jvm_common::classfile::{ConstantInfo, Class, String_, ConstantKind};
use runtime_common::java_values::{JavaValue, ObjectPointer, VecPointer};
use rust_jvm_common::classnames::ClassName;
use crate::get_or_create_class_object;
use rust_jvm_common::utils::extract_string_from_utf8;
use crate::interpreter_util::{check_inited_class, push_new_object, run_function};
use rust_jvm_common::unified_types::{ParsedType, ArrayType};
use std::sync::Arc;
use crate::instructions::invoke::find_target_method;
use classfile_parser::types::MethodDescriptor;
use std::mem::transmute;

fn load_class_constant(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, constant_pool: &Vec<ConstantInfo>, c: &Class) {
    let res_class_name = extract_string_from_utf8(&constant_pool[c.name_index as usize]);
    load_class_constant_by_name(state, current_frame, res_class_name);
}

pub fn load_class_constant_by_name(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, res_class_name: String) {
    let object = get_or_create_class_object(state, &ClassName::Str(res_class_name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
    current_frame.operand_stack.borrow_mut().push(JavaValue::Object(ObjectPointer {
        object
    }.into()));
}

fn load_string_constant(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, constant_pool: &Vec<ConstantInfo>, s: &String_) {
    let res_string = extract_string_from_utf8(&constant_pool[s.string_index as usize]);
    create_string_on_stack(state, current_frame, res_string);
}

pub fn create_string_on_stack(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, res_string: String) {
    let java_lang_string = ClassName::string();
    let current_loader = current_frame.class_pointer.loader.clone();
    let string_class = check_inited_class(state, &java_lang_string, current_frame.clone().into(), current_loader.clone());
    let str_as_vec = res_string.into_bytes().clone();
    let chars: Vec<JavaValue> = str_as_vec.iter().map(|x| { JavaValue::Char(*x as char) }).collect();
    push_new_object(current_frame.clone().into(), &string_class);
    let string_object = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Array(Some(VecPointer { object: Arc::new(chars.into()) })));
    let char_array_type = ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::CharType) });
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type], return_type: ParsedType::VoidType };
    let (constructor_i, _constructor) = find_target_method(current_loader.clone(), "<init>".to_string(), &expected_descriptor, &string_class);
    let next_entry = CallStackEntry {
        last_call_stack: Some(current_frame.clone().into()),
        class_pointer: string_class,
        method_i: constructor_i as u16,
        local_vars: args.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    };
    run_function(state, Rc::new(next_entry));
    if state.terminate || state.throw {
        unimplemented!()
    }
    if state.function_return {
        state.function_return = false;
    }
    current_frame.operand_stack.borrow_mut().push(string_object);
}

pub fn ldc2_w(current_frame: Rc<CallStackEntry>, cp: u16) -> () {
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let pool_entry = &constant_pool[cp as usize];
    match &pool_entry.kind {
        ConstantKind::Long(l) => {
            let high = l.high_bytes as u64;
            let low = l.low_bytes as u64;
            current_frame.operand_stack.borrow_mut().push(JavaValue::Long((high << 32 | low) as i64));
        }
        ConstantKind::Double(d) => {
            let high = d.high_bytes as u64;
            let low = d.low_bytes as u64;
            current_frame.operand_stack.borrow_mut().push(JavaValue::Double(unsafe { transmute(high << 32 | low) }));
        }
        _ => {}
    }
}


pub fn ldc(state: &mut InterpreterState, current_frame: Rc<CallStackEntry>, cp: u8) -> () {
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let pool_entry = &constant_pool[cp as usize];
    match &pool_entry.kind {
        ConstantKind::String(s) => load_string_constant(state, &current_frame, &constant_pool, &s),
        ConstantKind::Class(c) => load_class_constant(state, &current_frame, constant_pool, &c),
        ConstantKind::Float(f) => {
            let float: f32 = unsafe { transmute(f.bytes) };
            current_frame.operand_stack.borrow_mut().push(JavaValue::Float(float));
        }
        ConstantKind::Integer(i) => {
            let int: i32 = unsafe { transmute(i.bytes) };
            current_frame.operand_stack.borrow_mut().push(JavaValue::Int(int));
        }
        _ => {
            dbg!(&pool_entry.kind);
            unimplemented!()
        }
    }
}