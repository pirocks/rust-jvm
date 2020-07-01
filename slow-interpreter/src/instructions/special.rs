use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::{ClassName};
use std::sync::Arc;
use std::ops::Deref;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::JavaValue;
use crate::{JVMState, StackEntry};
use crate::runtime_class::RuntimeClass;
use crate::java_values::Object::{Array, Object};
use crate::java_values;
use classfile_view::view::interface_view::InterfaceView;
use rust_jvm_common::classfile::{Wide, IInc};


pub fn arraylength(current_frame: &StackEntry) -> () {
    let array_o = current_frame.pop().unwrap_object().unwrap();
    let array = array_o.unwrap_array();
    current_frame.push(JavaValue::Int(array.elems.borrow().len() as i32));
}


pub fn invoke_checkcast(jvm: &'static JVMState, current_frame: &StackEntry, cp: u16) {
    let possibly_null = current_frame.pop().unwrap_object();
    if possibly_null.is_none() {
        current_frame.push(JavaValue::Object(possibly_null));
        return;
    }
    let object = possibly_null.unwrap();
    match object.deref() {
        Object(o) => {
            let view = &current_frame.class_pointer.view();
            let instance_of_class_name = view.constant_pool_view(cp as usize).unwrap_class().class_name().unwrap_name();
            let instanceof_class = check_inited_class(jvm, &instance_of_class_name.into(), current_frame.class_pointer.loader(jvm).clone());
            let object_class = o.class_pointer.clone();
            if inherits_from(jvm, &object_class, &instanceof_class) {
                current_frame.push(JavaValue::Object(object.clone().into()));
                return;
            } else {
                // current_frame.print_stack_trace();
                unimplemented!()
            }
        }
        Array(a) => {
            let current_frame_class = &current_frame.class_pointer.view();
            let instance_of_class = current_frame_class
                .constant_pool_view(cp as usize)
                .unwrap_class().class_name();
            let expected_type_wrapped = PTypeView::Ref(instance_of_class);

            let expected_type = expected_type_wrapped.unwrap_array_type();
            let cast_succeeds = match &a.elem_type {
                PTypeView::Ref(_) => {
                    //todo wrong for varying depth arrays?
                    let actual_runtime_class = check_inited_class(jvm, &a.elem_type.unwrap_class_type().clone().into(), current_frame.class_pointer.loader(jvm).clone());
                    let expected_runtime_class = check_inited_class(jvm, &expected_type.unwrap_class_type().clone().into(), current_frame.class_pointer.loader(jvm).clone());
                    inherits_from(jvm, &actual_runtime_class, &expected_runtime_class)
                }
                _ => {
                    a.elem_type == expected_type
                }
            };
            if cast_succeeds {
                current_frame.push(JavaValue::Object(object.clone().into()));
                return;
            } else {
                unimplemented!()
            }
        }
    }
    //todo dup with instance off
}


pub fn invoke_instanceof(state: &'static JVMState, current_frame: &StackEntry, cp: u16) {
    let possibly_null = current_frame.pop().unwrap_object();
    if possibly_null.is_none() {
        current_frame.push(JavaValue::Int(0));
        return;
    }
    let unwrapped = possibly_null.unwrap();
    let view = &current_frame.class_pointer.view();
    let instance_of_class_type = view.constant_pool_view(cp as usize).unwrap_class().class_name();
    // assert!(instance_of_class_type.try_unwrap_name().is_none());
    instance_of_impl(state, current_frame, unwrapped, instance_of_class_type);
}

pub fn instance_of_impl(jvm: &'static JVMState, current_frame: &StackEntry, unwrapped: Arc<java_values::Object>, instance_of_class_type: ReferenceTypeView) {
    match unwrapped.deref() {
        Array(array) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    if instance_of_class_name == ClassName::serializable() ||
                        instance_of_class_name == ClassName::cloneable() {
                        unimplemented!()//todo need to handle serializable and the like
                    } else {
                        current_frame.push(JavaValue::Int(0))
                    }
                }
                ReferenceTypeView::Array(a) => {
                    if a.deref() == &array.elem_type {
                        current_frame.push(JavaValue::Int(1))
                    }
                }
            }
        }
        Object(object) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    let instanceof_class = check_inited_class(jvm, &instance_of_class_name.into(), current_frame.class_pointer.loader(jvm).clone());
                    let object_class = object.class_pointer.clone();
                    if inherits_from(jvm, &object_class, &instanceof_class) {
                        current_frame.push(JavaValue::Int(1))
                    } else {
                        current_frame.push(JavaValue::Int(0))
                    }
                }
                ReferenceTypeView::Array(_) => current_frame.push(JavaValue::Int(0)),
            }
        }
    };
}

fn runtime_super_class(jvm: &'static JVMState, inherits: &Arc<RuntimeClass>) -> Option<Arc<RuntimeClass>> {
    if inherits.view().super_name().is_some() {
        Some(check_inited_class(jvm, &inherits.view().super_name().unwrap().into(), inherits.loader(jvm).clone()))
    } else {
        None
    }
}

fn runtime_interface_class(jvm: &'static JVMState, class_: &Arc<RuntimeClass>, i: InterfaceView) -> Arc<RuntimeClass> {
    let intf_name = i.interface_name();
    check_inited_class(jvm, &ClassName::Str(intf_name).into(), class_.loader(jvm).clone())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from(state: &'static JVMState, inherits: &Arc<RuntimeClass>, parent: &Arc<RuntimeClass>) -> bool {
    if &inherits.view().name() == &parent.view().name() {
        return true;
    }
    let interfaces_match = inherits.view().interfaces().enumerate().any(|(_, i)| {
        let interface = runtime_interface_class(state, inherits, i);
        &interface.view().name() == &parent.view().name()
    });

    (match runtime_super_class(state, inherits) {
        None => false,
        Some(super_) => {
            //todo why is this not an impl function?
            &super_.view().name() == &parent.view().name() ||
                inherits_from(state, &super_, parent)
        }
    }) || interfaces_match
}

pub fn wide(current_frame: &StackEntry, w: Wide) {
    match w {
        Wide::Iload(_) => {
            unimplemented!()
        }
        Wide::Fload(_) => {
            unimplemented!()
        }
        Wide::Aload(_) => {
            unimplemented!()
        }
        Wide::Lload(_) => {
            unimplemented!()
        }
        Wide::Dload(_) => {
            unimplemented!()
        }
        Wide::Istore(_) => {
            unimplemented!()
        }
        Wide::Fstore(_) => {
            unimplemented!()
        }
        Wide::Astore(_) => {
            unimplemented!()
        }
        Wide::Lstore(_) => {
            unimplemented!()
        }
        Wide::Ret(_) => {
            unimplemented!()
        }
        Wide::IInc(iinc) => {
            let IInc{ index, const_ } = iinc;
            let mut  val = current_frame.local_vars.borrow()[index as usize].unwrap_int();
            val += const_ as i32;
            current_frame.local_vars.borrow_mut()[index as usize] = JavaValue::Int(val);
        }
    }
}