use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::interface_view::InterfaceView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::{IInc, Wide, WideAload, WideAstore, WideDload, WideDstore, WideFload, WideFstore, WideIload, WideIstore, WideLload, WideLstore, WideRet};
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::{check_initing_or_inited_class, check_resolved_class};
use crate::instructions::load::{aload, dload, fload, iload, lload};
use crate::instructions::store::{astore, dstore, fstore, istore, lstore};
use crate::interpreter::{ret, WasException};
use crate::java_values;
use crate::java_values::JavaValue;
use crate::java_values::Object::{Array, Object};
use crate::runtime_class::RuntimeClass;
use crate::utils::throw_npe;

pub fn arraylength(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame = int_state.current_frame_mut();
    let array_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array = array_o.unwrap_array();
    current_frame.push(JavaValue::Int(array.mut_array().len() as i32));
}


pub fn invoke_checkcast(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let possibly_null = int_state.current_frame_mut().pop().unwrap_object();
    if possibly_null.is_none() {
        int_state.current_frame_mut().push(JavaValue::Object(possibly_null));
        return;
    }
    let object = match possibly_null {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    match object.deref() {
        Object(o) => {
            let view = &int_state.current_frame_mut().class_pointer().view();
            let instance_of_class_name = view.constant_pool_view(cp as usize).unwrap_class().class_ref_type().unwrap_name();
            let instanceof_class = match check_initing_or_inited_class(jvm, int_state, instance_of_class_name.into()) {
                Ok(x) => x,
                Err(WasException {}) => return,
            };
            let object_class = o.objinfo.class_pointer.clone();
            if match inherits_from(jvm, int_state, &object_class, &instanceof_class) {
                Ok(x) => x,
                Err(WasException {}) => return
            } {
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
                .unwrap_class().class_ref_type();
            let expected_type_wrapped = PTypeView::Ref(instance_of_class);

            let expected_type = expected_type_wrapped.unwrap_array_type();
            let cast_succeeds = match &a.elem_type {
                PTypeView::Ref(_) => {
                    //todo wrong for varying depth arrays?
                    int_state.debug_print_stack_trace();
                    let actual_runtime_class = match check_initing_or_inited_class(jvm, int_state, a.elem_type.clone()) {
                        Ok(x) => x,
                        Err(WasException {}) => return,
                    };
                    let expected_runtime_class = match check_initing_or_inited_class(jvm, int_state, expected_type.clone()) {
                        Ok(x) => x,
                        Err(WasException {}) => return,
                    };
                    match inherits_from(jvm, int_state, &actual_runtime_class, &expected_runtime_class) {
                        Ok(res) => res,
                        Err(WasException {}) => return
                    }
                }
                _ => {
                    a.elem_type == expected_type
                }
            };
            if cast_succeeds {
                int_state.push_current_operand_stack(JavaValue::Object(object.clone().into()));
            } else {
                dbg!(&a.elem_type);
                dbg!(expected_type);
                unimplemented!()
            }
        }
    }
    //todo dup with instance off
}


pub fn invoke_instanceof(state: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let possibly_null = int_state.pop_current_operand_stack().unwrap_object();
    if let Some(unwrapped) = possibly_null {
        let view = &int_state.current_class_view();
        let instance_of_class_type = view.constant_pool_view(cp as usize).unwrap_class().class_ref_type();
        if let Err(WasException {}) = instance_of_impl(state, int_state, unwrapped, instance_of_class_type) {
            return;
        }
    } else {
        int_state.push_current_operand_stack(JavaValue::Int(0));
        return;
    }
}

pub fn instance_of_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, unwrapped: Arc<java_values::Object>, instance_of_class_type: ReferenceTypeView) -> Result<(), WasException> {
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
                    let instanceof_class = check_resolved_class(jvm, int_state, instance_of_class_name.into())?;//todo check if this should be here
                    let object_class = object.objinfo.class_pointer.clone();
                    if inherits_from(jvm, int_state, &object_class, &instanceof_class)? {
                        int_state.push_current_operand_stack(JavaValue::Int(1))
                    } else {
                        int_state.push_current_operand_stack(JavaValue::Int(0))
                    }
                }
                ReferenceTypeView::Array(_) => int_state.push_current_operand_stack(JavaValue::Int(0)),
            }
        }
    };
    Ok(())
}

fn runtime_super_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, inherits: &Arc<RuntimeClass>) -> Result<Option<Arc<RuntimeClass>>, WasException> {
    Ok(if inherits.view().super_name().is_some() {
        Some(check_initing_or_inited_class(jvm, int_state, inherits.view().super_name().unwrap().into())?)
    } else {
        None
    })
}

fn runtime_interface_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, i: InterfaceView) -> Result<Arc<RuntimeClass>, WasException> {
    let intf_name = i.interface_name();
    check_initing_or_inited_class(jvm, int_state, intf_name.into())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from(jvm: &JVMState, int_state: &mut InterpreterStateGuard, inherits: &Arc<RuntimeClass>, parent: &Arc<RuntimeClass>) -> Result<bool, WasException> {
    //todo it is questionable whether this logic should be here:
    if let RuntimeClass::Array(arr) = inherits.deref() {
        if parent.ptypeview() == ClassName::object().into() ||
            parent.ptypeview() == ClassName::cloneable().into() ||
            parent.ptypeview() == ClassName::serializable().into() {
            return Ok(true)
        }
        if let RuntimeClass::Array(parent_arr) = parent.deref() {
            return inherits_from(jvm, int_state, &arr.sub_class.clone(), &parent_arr.sub_class.clone())
        }
    }
    if inherits.ptypeview().is_primitive() {
        return Ok(false)
    }

    if inherits.view().name() == parent.view().name() {
        return Ok(true);
    }
    let mut interfaces_match = false;

    for (_, i) in inherits.view().interfaces().enumerate() {
        let interface = runtime_interface_class(jvm, int_state, i)?;
        interfaces_match |= interface.view().name() == parent.view().name();
    }


    Ok((match runtime_super_class(jvm, int_state, inherits)? {
        None => false,
        Some(super_) => {
            super_.view().name() == parent.view().name() ||
                inherits_from(jvm, int_state, &super_, parent)?
        }
    }) || interfaces_match)
}

pub fn wide(current_frame: &mut StackEntry, w: Wide) {
    match w {
        Wide::Iload(WideIload { index }) => {
            iload(current_frame, index as usize)
        }
        Wide::Fload(WideFload { index }) => {
            fload(current_frame, index as usize)
        }
        Wide::Aload(WideAload { index }) => {
            aload(current_frame, index as usize)
        }
        Wide::Lload(WideLload { index }) => {
            lload(current_frame, index as usize)
        }
        Wide::Dload(WideDload { index }) => {
            dload(current_frame, index as usize)
        }
        Wide::Istore(WideIstore { index }) => {
            istore(current_frame, index as usize)
        }
        Wide::Fstore(WideFstore { index }) => {
            fstore(current_frame, index as usize)
        }
        Wide::Astore(WideAstore { index }) => {
            astore(current_frame, index as usize)
        }
        Wide::Lstore(WideLstore { index }) => {
            lstore(current_frame, index as usize)
        }
        Wide::Ret(WideRet { index }) => {
            ret(current_frame, index as usize)
        }
        Wide::Dstore(WideDstore { index }) => {
            dstore(current_frame, index as usize)
        }
        Wide::IInc(iinc) => {
            let IInc { index, const_ } = iinc;
            let mut val = current_frame.local_vars()[index as usize].unwrap_int();
            val += const_ as i32;
            current_frame.local_vars_mut()[index as usize] = JavaValue::Int(val);
        }
    }
}