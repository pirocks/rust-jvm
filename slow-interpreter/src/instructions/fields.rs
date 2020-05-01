use crate::interpreter_util::check_inited_class;
use verification::verifier::instructions::special::extract_field_descriptor;
use rust_jvm_common::classnames::ClassName;
use classfile_view::loading::LoaderArc;
use crate::java_values::JavaValue;
use crate::{JVMState, StackEntry};


pub fn putstatic(jvm: & JVMState, current_frame: & StackEntry, cp: u16) -> () {
    let view = &current_frame.class_pointer.view();
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let target_classfile = check_inited_class(jvm, &field_class_name, loader_arc.clone());
    let mut stack = current_frame.operand_stack.borrow_mut();
    let field_value = stack.pop().unwrap();
    target_classfile.static_vars.write().unwrap().insert(field_name, field_value);
}

pub fn putfield(jvm: & JVMState, current_frame: & StackEntry, cp: u16) -> () {
    let view = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let _target_classfile = check_inited_class(jvm, &field_class_name, loader_arc.clone());
    let mut stack = current_frame.operand_stack.borrow_mut();
    let val = stack.pop().unwrap();
    let object_ref = stack.pop().unwrap();
    match object_ref {
        JavaValue::Object(o) => {
            {
                o.unwrap().unwrap_normal_object().fields.borrow_mut().insert(field_name, val);
            }
        }
        _ => {
            dbg!(object_ref);
            panic!()
        }
    }
}

pub fn get_static(jvm: & JVMState, current_frame: & StackEntry, cp: u16) -> () {
    //todo make sure class pointer is updated correctly

    let view = &current_frame.class_pointer.view();
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let (field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    /*if field_name == "reflectionFactory" {
        dbg!(cp);
        panic!()
    }*/
    get_static_impl(jvm, current_frame, cp, loader_arc, &field_class_name, &field_name);
}

fn get_static_impl(state: & JVMState, current_frame: & StackEntry, cp: u16, loader_arc: &LoaderArc, field_class_name: &ClassName, field_name: &String) {
    let target_classfile = check_inited_class(state, &field_class_name,  loader_arc.clone());
//    current_frame.print_stack_trace();
    let temp = target_classfile.static_vars.read().unwrap();
    let attempted_get = temp.get(field_name);
    let field_value = match attempted_get {
        None => {
            return get_static_impl(state,current_frame,cp,loader_arc,&target_classfile.view().super_name().unwrap(),field_name)
        },
        Some(val) => {
            val.clone()
        },
    };
    let mut stack = current_frame.operand_stack.borrow_mut();
    stack.push(field_value);
}

pub fn get_field(current_frame: & StackEntry, cp: u16, _debug: bool) -> () {
    let view = &current_frame.class_pointer.view();
    let (_field_class_name, field_name, _field_descriptor) = extract_field_descriptor(cp, view);
    let object_ref = current_frame.pop();
    match object_ref {
        JavaValue::Object(o) => {
            let fields = o.as_ref().unwrap().unwrap_normal_object().fields.borrow();
            if fields.get(field_name.as_str()).is_none() {
                dbg!(&o);
                dbg!(&fields.keys());
            }
            let res = fields.get(field_name.as_str()).unwrap().clone();
            current_frame.push(res);
        }
        _ => panic!(),
    }
}

