use std::rc::Rc;
use rust_jvm_common::classfile::{ConstantInfo, Class, String_, ConstantKind};
use rust_jvm_common::classnames::ClassName;
use crate::{StackEntry, JVMState};
use crate::interpreter_util::{check_inited_class, push_new_object, run_function};
use std::sync::Arc;
use crate::instructions::invoke::find_target_method;
use std::mem::transmute;
use std::cell::RefCell;
use crate::rust_jni::native_util::{to_object, from_object};
use crate::rust_jni::interface::string::intern_impl;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::{JavaValue, Object, ArrayObject};
use descriptor_parser::{MethodDescriptor, parse_field_descriptor};
use crate::class_objects::get_or_create_class_object;


fn load_class_constant(state: & JVMState, current_frame: &Rc<StackEntry>, constant_pool: &Vec<ConstantInfo>, c: &Class) {
    let res_class_name = constant_pool[c.name_index as usize].extract_string_from_utf8();
    let type_ = parse_field_descriptor(&res_class_name).unwrap().field_type;
    load_class_constant_by_type(state, current_frame, &PTypeView::from_ptype(&type_));
}

pub fn load_class_constant_by_type(state: & JVMState, current_frame: &Rc<StackEntry>, res_class_type: &PTypeView) {
    let object = get_or_create_class_object(state, res_class_type, current_frame.clone().into(), current_frame.class_pointer.loader.clone());
    current_frame.push(JavaValue::Object(object.into()));
}

fn load_string_constant(state: & JVMState, current_frame: &Rc<StackEntry>, constant_pool: &Vec<ConstantInfo>, s: &String_) {
    let res_string = constant_pool[s.string_index as usize].extract_string_from_utf8();
    create_string_on_stack(state, current_frame, res_string);
}

pub fn create_string_on_stack(state: & JVMState, current_frame: &Rc<StackEntry>, res_string: String) {
    let java_lang_string = ClassName::string();
    let current_loader = current_frame.class_pointer.loader.clone();
    let string_class = check_inited_class(state, &java_lang_string, current_frame.clone().into(), current_loader.clone());
    let str_as_vec = res_string.chars();
    let chars: Vec<JavaValue> = str_as_vec.map(|x| { JavaValue::Char(x) }).collect();
    push_new_object(state,current_frame.clone().into(), &string_class);
    let string_object = current_frame.pop();
    let mut args = vec![string_object.clone()];
    args.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(chars), elem_type: PTypeView::CharType })))));
    let char_array_type = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::CharType.into()));
    let expected_descriptor = MethodDescriptor { parameter_types: vec![char_array_type.to_ptype()], return_type: PTypeView::VoidType.to_ptype() };
    let (constructor_i, final_target_class) = find_target_method(state, current_loader.clone(), "<init>".to_string(), &expected_descriptor, string_class);
    let next_entry = StackEntry {
        last_call_stack: Some(current_frame.clone().into()),
        class_pointer: final_target_class,
        method_i: constructor_i as u16,
        local_vars: args.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    };
    run_function(state, Rc::new(next_entry));
    let interpreter_state = &state.get_current_thread().interpreter_state;
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


pub fn ldc_w(state: & JVMState, current_frame: Rc<StackEntry>, cp: u16) -> () {
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

pub fn from_constant_pool_entry(constant_pool: &Vec<ConstantInfo>, c: &ConstantInfo, state: & JVMState, stack: Rc<StackEntry>) -> JavaValue {
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
        ConstantKind::String(s) => {
            load_string_constant(state, &stack, constant_pool, s);
            stack.pop()
        }
        _ => panic!()
    }
}