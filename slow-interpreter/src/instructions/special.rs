use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ClassView;
use classfile_view::view::interface_view::InterfaceView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::{IInc, Wide};
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::{check_initing_or_inited_class, check_resolved_class};
use crate::java_values;
use crate::java_values::JavaValue;
use crate::java_values::Object::{Array, Object};
use crate::runtime_class::RuntimeClass;

pub fn arraylength(int_state: &mut InterpreterStateGuard) {
    let current_frame = int_state.current_frame_mut();
    let array_o = current_frame.pop().unwrap_object().unwrap();//todo handle npe
    let array = array_o.unwrap_array();
    current_frame.push(JavaValue::Int(array.mut_array().len() as i32));
}


pub fn invoke_checkcast(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let possibly_null = int_state.current_frame_mut().pop().unwrap_object();
    if possibly_null.is_none() {
        int_state.current_frame_mut().push(JavaValue::Object(possibly_null));
        return;
    }
    let object = possibly_null.unwrap();//todo handle npe
    match object.deref() {
        Object(o) => {
            let view = &int_state.current_frame_mut().class_pointer().view();
            let instance_of_class_name = view.constant_pool_view(cp as usize).unwrap_class().class_name().unwrap_name();
            let instanceof_class = check_initing_or_inited_class(jvm, int_state, instance_of_class_name.into()).unwrap();//todo pass the error up
            let object_class = o.class_pointer.clone();
            if inherits_from(jvm, int_state, &object_class, &instanceof_class) {
                int_state.push_current_operand_stack(JavaValue::Object(object.clone().into()));
            } else {
                int_state.debug_print_stack_trace();
                dbg!(object_class.view().name());
                dbg!(instanceof_class.view().name());
                unimplemented!()
            }
        }
        Array(a) => {
            let current_frame_class = &int_state.current_frame_mut().class_pointer().view();
            let instance_of_class = current_frame_class
                .constant_pool_view(cp as usize)
                .unwrap_class().class_name();
            let expected_type_wrapped = PTypeView::Ref(instance_of_class);

            let expected_type = expected_type_wrapped.unwrap_array_type();
            let cast_succeeds = match &a.elem_type {
                PTypeView::Ref(_) => {
                    //todo wrong for varying depth arrays?
                    let actual_runtime_class = check_initing_or_inited_class(jvm, int_state, a.elem_type.unwrap_class_type().into()).unwrap();//todo pass the error up
                    let expected_runtime_class = check_initing_or_inited_class(jvm, int_state, expected_type.unwrap_class_type().into()).unwrap();//todo pass the error up
                    inherits_from(jvm, int_state, &actual_runtime_class, &expected_runtime_class)
                }
                _ => {
                    a.elem_type == expected_type
                }
            };
            if cast_succeeds {
                int_state.push_current_operand_stack(JavaValue::Object(object.clone().into()));
            } else {
                unimplemented!()
            }
        }
    }
    //todo dup with instance off
}


pub fn invoke_instanceof(state: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let possibly_null = int_state.pop_current_operand_stack().unwrap_object();
    if possibly_null.is_none() {
        int_state.push_current_operand_stack(JavaValue::Int(0));
        return;
    }
    let unwrapped = possibly_null.unwrap();//todo pass the error up
    let view = &int_state.current_class_view();
    let instance_of_class_type = view.constant_pool_view(cp as usize).unwrap_class().class_name();
    instance_of_impl(state, int_state, unwrapped, instance_of_class_type);
}

pub fn instance_of_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, unwrapped: Arc<java_values::Object>, instance_of_class_type: ReferenceTypeView) {
    match unwrapped.deref() {
        Array(array) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    if instance_of_class_name == ClassName::serializable() ||
                        instance_of_class_name == ClassName::cloneable() {
                        unimplemented!()//todo need to handle serializable and the like
                    } else {
                        int_state.push_current_operand_stack(JavaValue::Int(0))
                    }
                }
                ReferenceTypeView::Array(a) => {
                    if a.deref() == &array.elem_type {
                        int_state.push_current_operand_stack(JavaValue::Int(1))
                    }
                }
            }
        }
        Object(object) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    let instanceof_class = check_resolved_class(jvm, int_state, instance_of_class_name.into()).unwrap();//todo check if this should be here//todo pass the error up
                    let object_class = object.class_pointer.clone();
                    if inherits_from(jvm, int_state, &object_class, &instanceof_class) {
                        int_state.push_current_operand_stack(JavaValue::Int(1))
                    } else {
                        int_state.push_current_operand_stack(JavaValue::Int(0))
                    }
                }
                ReferenceTypeView::Array(_) => int_state.push_current_operand_stack(JavaValue::Int(0)),
            }
        }
    };
}

fn runtime_super_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, inherits: &Arc<RuntimeClass>) -> Option<Arc<RuntimeClass>> {
    if inherits.view().super_name().is_some() {
        Some(check_initing_or_inited_class(jvm, int_state, inherits.view().super_name().unwrap().into()).unwrap())//todo pass the error up
    } else {
        None
    }
}

fn runtime_interface_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, i: InterfaceView) -> Arc<RuntimeClass> {
    let intf_name = i.interface_name();
    check_initing_or_inited_class(jvm, int_state, intf_name.into()).unwrap()//todo pass the error up
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from(state: &JVMState, int_state: &mut InterpreterStateGuard, inherits: &Arc<RuntimeClass>, parent: &Arc<RuntimeClass>) -> bool {
    if inherits.view().name() == parent.view().name() {
        return true;
    }
    let mut interfaces_match = false;

    for (_, i) in inherits.view().interfaces().enumerate() {
        let interface = runtime_interface_class(state, int_state, i);
        interfaces_match |= interface.view().name() == parent.view().name();
    }


    (match runtime_super_class(state, int_state, inherits) {
        None => false,
        Some(super_) => {
            //todo why is this not an impl function?
            super_.view().name() == parent.view().name() ||
                inherits_from(state, int_state, &super_, parent)
        }
    }) || interfaces_match
}

pub fn wide(current_frame: &mut StackEntry, w: Wide) {
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
        Wide::Dstore(_) => {
            unimplemented!()
        }
        Wide::IInc(iinc) => {
            let IInc { index, const_ } = iinc;
            let mut val = current_frame.local_vars()[index as usize].unwrap_int();
            val += const_ as i32;
            current_frame.local_vars_mut()[index as usize] = JavaValue::Int(val);
        }
    }
}