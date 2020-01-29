use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use rust_jvm_common::classfile::{ConstantInfo, Class, String_, ConstantKind};
use runtime_common::java_values::JavaValue;
use rust_jvm_common::classnames::ClassName;
use crate::get_or_create_class_object;
use crate::interpreter_util::{check_inited_class, push_new_object, run_function};
use rust_jvm_common::unified_types::{ParsedType, ArrayType};
use std::sync::Arc;
use crate::instructions::invoke::find_target_method;
use classfile_parser::types::MethodDescriptor;
use std::mem::transmute;
use std::cell::RefCell;

fn load_class_constant(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, constant_pool: &Vec<ConstantInfo>, c: &Class) {
    let res_class_name = constant_pool[c.name_index as usize].extract_string_from_utf8();
    load_class_constant_by_name(state, current_frame, res_class_name);
}

pub fn load_class_constant_by_name(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, res_class_name: String) {
    let object = get_or_create_class_object(state, &ClassName::Str(res_class_name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
    current_frame.push(JavaValue::Object(object.into()));
}

fn load_string_constant(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, constant_pool: &Vec<ConstantInfo>, s: &String_) {
    let res_string = constant_pool[s.string_index as usize].extract_string_from_utf8();
    create_string_on_stack(state, current_frame, res_string);
}

pub fn create_string_on_stack(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, res_string: String) {
    let java_lang_string = ClassName::string();
    let current_loader = current_frame.class_pointer.loader.clone();
    let string_class = check_inited_class(state, &java_lang_string, current_frame.clone().into(), current_loader.clone());
    let str_as_vec = res_string.clone().into_bytes().clone();
    let chars: Vec<JavaValue> = str_as_vec.iter().map(|x| { JavaValue::Char(*x as char) }).collect();
    push_new_object(current_frame.clone().into(), &string_class);
    let string_object = current_frame.pop();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Array(Arc::new(RefCell::new(chars)).into()));
    let char_array_type = ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::CharType) });
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type], return_type: ParsedType::VoidType };
    let (constructor_i, _constructor) = find_target_method(current_loader.clone(), "<init>".to_string(), &expected_descriptor, &string_class);
    let next_entry = StackEntry {
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
    dbg!(res_string);
    current_frame.push(string_object);
}

pub fn ldc2_w(current_frame: Rc<StackEntry>, cp: u16) -> () {
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let pool_entry = &constant_pool[cp as usize];
    match &pool_entry.kind {
        ConstantKind::Long(l) => {
            let high = l.high_bytes as u64;
            let low = l.low_bytes as u64;
            current_frame.push(JavaValue::Long((high << 32 | low) as i64));
        }
        ConstantKind::Double(d) => {
            let high = d.high_bytes as u64;
            let low = d.low_bytes as u64;
            current_frame.push(JavaValue::Double(unsafe { transmute(high << 32 | low) }));
        }
        _ => {}
    }
}


pub fn ldc(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u8) -> () {
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let pool_entry = &constant_pool[cp as usize];
    match &pool_entry.kind {
        ConstantKind::String(s) => load_string_constant(state, &current_frame, &constant_pool, &s),
        ConstantKind::Class(c) => load_class_constant(state, &current_frame, constant_pool, &c),
        ConstantKind::Float(f) => {
            let float: f32 = unsafe { transmute(f.bytes) };
            current_frame.push(JavaValue::Float(float));
        }
        ConstantKind::Integer(i) => {
            let int: i32 = unsafe { transmute(i.bytes) };
            current_frame.push(JavaValue::Int(int));
        }
        _ => {
            dbg!(&pool_entry.kind);
            unimplemented!()
        }
    }
}

pub fn from_constant_pool_entry(constant_pool: &Vec<ConstantInfo>,c: &ConstantInfo, state: &mut InterpreterState, stack: Option<Rc<StackEntry>>) -> JavaValue {
    match &c.kind {
        ConstantKind::Integer(i) => JavaValue::Int(unsafe { transmute(i.bytes) }),
        ConstantKind::Float(f) => JavaValue::Float(unsafe { transmute(f.bytes) }),
        ConstantKind::Long(l) => JavaValue::Long(unsafe {
            let high = (l.high_bytes as u64) << 32;
            let low = l.low_bytes as u64;
            transmute(high | low)
        }),
        ConstantKind::Double(d) => JavaValue::Double(unsafe {
            let high = (d.high_bytes as u64) << 32;
            let low = d.low_bytes as u64;
            transmute(high | low)
        }),
        ConstantKind::String(s) =>{
            load_string_constant(state,&stack.clone().unwrap(),constant_pool,s);
            stack.unwrap().pop()
        },
        _ => panic!()
    }
}