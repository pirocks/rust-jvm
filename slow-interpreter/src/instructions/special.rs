use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::interface_view::InterfaceView;
use rust_jvm_common::classfile::{
    IInc, Wide, WideAload, WideAstore, WideDload, WideDstore, WideFload, WideFstore, WideIload,
    WideIstore, WideLload, WideLstore, WideRet,
};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{check_initing_or_inited_class, check_resolved_class};
use crate::instructions::load::{aload, dload, fload, iload, lload};
use crate::instructions::store::{astore, dstore, fstore, istore, lstore};
use crate::interpreter::{ret, WasException};
use crate::java_values::{GcManagedObject, JavaValue};
use crate::java_values::Object::{Array, Object};
use crate::runtime_class::RuntimeClass;
use crate::stack_entry::StackEntryMut;
use crate::utils::throw_npe;

pub fn arraylength(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
) {
    let mut current_frame = int_state.current_frame_mut();
    let array_o = match current_frame
        .pop(Some(RuntimeType::object()))
        .unwrap_object()
    {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array = array_o.unwrap_array();
    current_frame.push(JavaValue::Int(array.len() as i32));
}

pub fn invoke_checkcast(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    cpdtype: &CPDType,
) {
    let possibly_null = int_state
        .current_frame_mut()
        .pop(Some(RuntimeType::object()))
        .unwrap_object();
    if possibly_null.is_none() {
        int_state
            .current_frame_mut()
            .push(JavaValue::Object(possibly_null));
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
            let instance_of_class_name = cpdtype.unwrap_class_type();
            let instanceof_class = match check_initing_or_inited_class(
                jvm,
                int_state,
                instance_of_class_name.into(),
            ) {
                Ok(x) => x,
                Err(WasException {}) => return,
            };
            let object_class = o.objinfo.class_pointer.clone();
            if match inherits_from(jvm, int_state, &object_class, &instanceof_class) {
                Ok(x) => x,
                Err(WasException {}) => return,
            } {
                int_state.push_current_operand_stack(JavaValue::Object(object.clone().into()));
            } else {
                int_state.debug_print_stack_trace(jvm);
                dbg!(object_class
                    .view()
                    .name()
                    .unwrap_object_name()
                    .0
                    .to_str(&jvm.string_pool));
                dbg!(instanceof_class
                    .view()
                    .name()
                    .unwrap_object_name()
                    .0
                    .to_str(&jvm.string_pool));
                unimplemented!()
            }
        }
        Array(a) => {
            let expected_type = cpdtype.unwrap_array_type();
            let cast_succeeds = match &a.elem_type {
                CPDType::Ref(_) => {
                    //todo wrong for varying depth arrays?
                    let actual_runtime_class =
                        match check_initing_or_inited_class(jvm, int_state, a.elem_type.clone()) {
                            Ok(x) => x,
                            Err(WasException {}) => return,
                        };
                    let expected_runtime_class = match check_initing_or_inited_class(
                        jvm,
                        int_state,
                        expected_type.clone(),
                    ) {
                        Ok(x) => x,
                        Err(WasException {}) => return,
                    };
                    match inherits_from(
                        jvm,
                        int_state,
                        &actual_runtime_class,
                        &expected_runtime_class,
                    ) {
                        Ok(res) => res,
                        Err(WasException {}) => return,
                    }
                }
                _ => &a.elem_type == expected_type,
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

pub fn invoke_instanceof(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    cpdtype: &CPDType,
) {
    let possibly_null = int_state
        .pop_current_operand_stack(Some(CClassName::object().into()))
        .unwrap_object();
    if let Some(unwrapped) = possibly_null {
        let instance_of_class_type = cpdtype.unwrap_ref_type().clone();
        if let Err(WasException {}) =
        instance_of_impl(jvm, int_state, unwrapped, instance_of_class_type)
        {
            return;
        }
    } else {
        int_state.push_current_operand_stack(JavaValue::Int(0));
        return;
    }
}

pub fn instance_of_impl(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    unwrapped: GcManagedObject<'gc_life>,
    instance_of_class_type: CPRefType,
) -> Result<(), WasException> {
    match unwrapped.deref() {
        Array(array) => {
            match instance_of_class_type {
                CPRefType::Class(instance_of_class_name) => {
                    if instance_of_class_name == CClassName::serializable()
                        || instance_of_class_name == CClassName::cloneable()
                    {
                        unimplemented!() //todo need to handle serializable and the like
                    } else {
                        int_state.push_current_operand_stack(JavaValue::Int(0))
                    }
                }
                CPRefType::Array(a) => {
                    if a.deref() == &array.elem_type {
                        int_state.push_current_operand_stack(JavaValue::Int(1))
                    }
                }
            }
        }
        Object(object) => {
            match instance_of_class_type {
                CPRefType::Class(instance_of_class_name) => {
                    let instanceof_class =
                        check_resolved_class(jvm, int_state, instance_of_class_name.into())?; //todo check if this should be here
                    let object_class = object.objinfo.class_pointer.clone();
                    if inherits_from(jvm, int_state, &object_class, &instanceof_class)? {
                        int_state.push_current_operand_stack(JavaValue::Int(1))
                    } else {
                        int_state.push_current_operand_stack(JavaValue::Int(0))
                    }
                }
                CPRefType::Array(_) => int_state.push_current_operand_stack(JavaValue::Int(0)),
            }
        }
    };
    Ok(())
}

fn runtime_super_class(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    inherits: &Arc<RuntimeClass<'gc_life>>,
) -> Result<Option<Arc<RuntimeClass<'gc_life>>>, WasException> {
    Ok(if inherits.view().super_name().is_some() {
        Some(check_initing_or_inited_class(
            jvm,
            int_state,
            inherits.view().super_name().unwrap().into(),
        )?)
    } else {
        None
    })
}

fn runtime_interface_class(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    i: InterfaceView,
) -> Result<Arc<RuntimeClass<'gc_life>>, WasException> {
    let intf_name = i.interface_name();
    check_initing_or_inited_class(jvm, int_state, intf_name.into())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    inherits: &Arc<RuntimeClass<'gc_life>>,
    parent: &Arc<RuntimeClass<'gc_life>>,
) -> Result<bool, WasException> {
    //todo it is questionable whether this logic should be here:
    if let RuntimeClass::Array(arr) = inherits.deref() {
        if parent.cpdtype() == CClassName::object().into()
            || parent.cpdtype() == CClassName::cloneable().into()
            || parent.cpdtype() == CClassName::serializable().into()
        {
            return Ok(true);
        }
        if let RuntimeClass::Array(parent_arr) = parent.deref() {
            return inherits_from(
                jvm,
                int_state,
                &arr.sub_class.clone(),
                &parent_arr.sub_class.clone(),
            );
        }
    }
    if inherits.cpdtype().is_primitive() {
        return Ok(false);
    }

    if inherits.view().name() == parent.view().name() {
        return Ok(true);
    }
    let mut interfaces_match = false;

    for (_, i) in inherits.view().interfaces().enumerate() {
        let interface = runtime_interface_class(jvm, int_state, i)?;
        interfaces_match |= inherits_from(jvm, int_state, &interface, &parent)?;
    }

    Ok((match runtime_super_class(jvm, int_state, inherits)? {
        None => false,
        Some(super_) => {
            super_.view().name() == parent.view().name()
                || inherits_from(jvm, int_state, &super_, parent)?
        }
    }) || interfaces_match)
}

pub fn wide(
    jvm: &'gc_life JVMState<'gc_life>,
    mut current_frame: StackEntryMut<'gc_life, 'l>,
    w: &Wide,
) {
    match w {
        Wide::Iload(WideIload { index }) => iload(jvm, current_frame, *index),
        Wide::Fload(WideFload { index }) => fload(jvm, current_frame, *index),
        Wide::Aload(WideAload { index }) => aload(current_frame, *index),
        Wide::Lload(WideLload { index }) => lload(jvm, current_frame, *index),
        Wide::Dload(WideDload { index }) => dload(jvm, current_frame, *index),
        Wide::Istore(WideIstore { index }) => istore(jvm, current_frame, *index),
        Wide::Fstore(WideFstore { index }) => fstore(jvm, current_frame, *index),
        Wide::Astore(WideAstore { index }) => astore(current_frame, *index),
        Wide::Lstore(WideLstore { index }) => lstore(jvm, current_frame, *index),
        Wide::Ret(WideRet { index }) => ret(jvm, current_frame, *index),
        Wide::Dstore(WideDstore { index }) => dstore(jvm, current_frame, *index),
        Wide::IInc(iinc) => {
            let IInc { index, const_ } = iinc;
            let mut val = current_frame
                .local_vars()
                .get(*index, RuntimeType::IntType)
                .unwrap_int();
            val += *const_ as i32;
            current_frame
                .local_vars_mut()
                .set(*index, JavaValue::Int(val));
        }
    }
}