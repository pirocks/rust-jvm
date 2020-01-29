use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;
use runtime_common::java_values::JavaValue;
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::{ClassName, class_name};
use runtime_common::runtime_class::RuntimeClass;
use std::sync::Arc;
use rust_jvm_common::classfile::Interface;

pub fn arraylength(current_frame: &Rc<StackEntry>) -> () {
    let array = current_frame.pop();
    match array {
        JavaValue::Array(a) => {
            current_frame.push(JavaValue::Int(a.unwrap().borrow().len() as i32));
        }
        _ => panic!()
    }
}

pub fn invoke_instanceof(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) {
    let object = current_frame.pop().unwrap_object();
    if object.is_none() {
        current_frame.push(JavaValue::Int(0));
        return;
    }
    let classfile = &current_frame.class_pointer.classfile;
    let instance_of_class_name = classfile.extract_class_from_constant_pool_name(cp);
    let instanceof_class = check_inited_class(state, &ClassName::Str(instance_of_class_name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
    let object_class = object.unwrap().class_pointer.clone();
//    dbg!(class_name(&object_class.classfile));
//    dbg!(class_name(&instanceof_class.classfile));
    if inherits_from(state, &object_class, &instanceof_class) {
        current_frame.push(JavaValue::Int(1))
    }else {
        current_frame.push(JavaValue::Int(0))
    }
}

fn runtime_super_class(state: &mut InterpreterState,inherits: &Arc<RuntimeClass>) -> Option<Arc<RuntimeClass>> {
    if inherits.classfile.has_super_class() {
        Some(check_inited_class(state, &inherits.classfile.super_class_name(), None, inherits.loader.clone()))
    } else {
        None
    }
}


fn runtime_interface_class(state: &mut InterpreterState,class_: &Arc<RuntimeClass>, i: Interface) -> Arc<RuntimeClass> {
    let intf_name = class_.classfile.extract_class_from_constant_pool_name(i);
    check_inited_class(state, &ClassName::Str(intf_name), None, class_.loader.clone())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
fn inherits_from(state: &mut InterpreterState,inherits: &Arc<RuntimeClass>, parent: &Arc<RuntimeClass>) -> bool {
    let interfaces_match = inherits.classfile.interfaces.iter().any(|x| {
        let interface = runtime_interface_class(state,inherits, *x);
        class_name(&interface.classfile) == class_name(&parent.classfile)
    });

    (match runtime_super_class(state,inherits) {
        None => false,
        Some(super_) => {
            //todo why is this not an impl function?
            class_name(&super_.classfile) == class_name(&parent.classfile) ||
                inherits_from(state,&super_, parent)
        }
    }) || interfaces_match
}