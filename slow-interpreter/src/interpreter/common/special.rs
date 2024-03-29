use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::interface_view::InterfaceView;
use gc_memory_layout_common::memory_regions::MemoryRegions;
use jvmti_jni_bindings::jint;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedRefType, CPDType, CPRefType};


use crate::{AllocatedHandle, JVMState, WasException};
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::class_loading::{assert_inited_or_initing_class, assert_loaded_class, check_resolved_class, try_assert_loaded_class};
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue, RealInterpreterStateGuard};
use crate::java_values::GcManagedObject;
use crate::java_values::Object::{Array, Object};

pub fn instance_of_exit_impl<'gc, 'any>(jvm: &'gc JVMState<'gc>, cpdtype: CPDType, obj: Option<&'any AllocatedHandle<'gc>>) -> jint {
    match obj {
        None => {
            0
        }
        Some(obj) => {
            instance_of_exit_impl_impl(jvm, cpdtype.unwrap_ref_type(), obj)
        }
    }
}

pub fn instance_of_exit_impl_impl<'gc>(jvm: &'gc JVMState<'gc>, instance_of_class_type: CompressedParsedRefType, obj: &'_ AllocatedHandle<'gc>) -> jint {
    let rc = obj.runtime_class(jvm);
    instance_of_exit_impl_impl_impl(&jvm, instance_of_class_type, rc, obj)
}

pub fn instance_of_exit_impl_impl_impl<'gc>(jvm: &'gc JVMState<'gc>, instance_of_class_type: CompressedParsedRefType, runtime_object_class: Arc<RuntimeClass<'gc>>, obj: &'_ AllocatedHandle<'gc>) -> jint {
    let actual_cpdtype = runtime_object_class.cpdtype();
    match actual_cpdtype.unwrap_ref_type() {
        CompressedParsedRefType::Array { base_type: actual_base_type, num_nested_arrs: actual_num_nested_arrs } => {
            match instance_of_class_type {
                CompressedParsedRefType::Class(instance_of_class_name) => {
                    if instance_of_class_name == CClassName::serializable() || instance_of_class_name == CClassName::cloneable() {
                        unimplemented!() //todo need to handle serializable and the like, check subtype is castable as per spec
                    } else if instance_of_class_name == CClassName::object() {
                        1
                    } else {
                        0
                    }
                }
                CompressedParsedRefType::Array { base_type: expected_class_type, num_nested_arrs: expected_num_nested_arrs } => {
                    if actual_base_type == expected_class_type && actual_num_nested_arrs == expected_num_nested_arrs {
                        1
                    } else {
                        if actual_num_nested_arrs == expected_num_nested_arrs {
                            if inherits_from_cpdtype(jvm, &assert_loaded_class(jvm, actual_base_type.to_cpdtype()), expected_class_type.to_cpdtype()) {
                                return 1;
                            }
                            return 0;
                        }
                        dbg!(actual_num_nested_arrs);
                        dbg!(expected_num_nested_arrs);
                        dbg!(actual_base_type.to_cpdtype().jvm_representation(&jvm.string_pool));
                        dbg!(expected_class_type.to_cpdtype().jvm_representation(&jvm.string_pool));

                        todo!()
                    }
                }
            }
        }
        CompressedParsedRefType::Class(object) => {
            match instance_of_class_type {
                CompressedParsedRefType::Class(instance_of_class_name) => {
                    if let Some(instance_of_class) = try_assert_loaded_class(jvm, instance_of_class_name.into()) {
                        if instance_of_class.unwrap_class_class().class_view.is_interface() {
                            let interface_class_id = jvm.class_ids.get_id_or_add(instance_of_class.cpdtype());
                            let guard = jvm.gc.memory_region.lock().unwrap();
                            let region_header = MemoryRegions::find_object_region_header(obj.ptr());
                            for i in 0..region_header.interface_ids_list_len {
                                let current_class_id = unsafe { region_header.interface_ids_list.offset(i as isize).read() };
                                if current_class_id == interface_class_id {
                                    assert!(inherits_from_cpdtype(jvm, &runtime_object_class, instance_of_class_name.into()));
                                    return 1;
                                }
                            }
                            assert!(!inherits_from_cpdtype(jvm, &runtime_object_class, instance_of_class_name.into()));
                            return 0;
                        } else {
                            if let Some(sub_inheritance_tree_vec) = runtime_object_class.unwrap_class_class().inheritance_tree_vec.as_ref() {
                                let instance_of_class = match try_assert_loaded_class(jvm, instance_of_class_type.to_cpdtype()) {
                                    None => {
                                        return if inherits_from_cpdtype(jvm, &runtime_object_class, instance_of_class_name.into()) {
                                            panic!()
                                        } else {
                                            0
                                        };
                                    }
                                    Some(instance_of_class) => {
                                        if let Some(super_inheritance_tree_vec) = instance_of_class.unwrap_class_class().inheritance_tree_vec.as_ref() {
                                            unsafe {
                                                return if sub_inheritance_tree_vec.as_ref().is_subpath_of(super_inheritance_tree_vec.as_ref()) {
                                                    assert!(inherits_from_cpdtype(jvm, &runtime_object_class, instance_of_class_name.into()));
                                                    1
                                                } else {
                                                    assert!(!inherits_from_cpdtype(jvm, &runtime_object_class, instance_of_class_name.into()));
                                                    0
                                                };
                                            }
                                        }
                                    }
                                };
                            }
                        }
                    }
                    if inherits_from_cpdtype(jvm, &runtime_object_class, instance_of_class_name.into()) {
                        1
                    } else {
                        0
                    }
                }
                CompressedParsedRefType::Array { .. } => {
                    0
                }
            }
        }
    }
}

pub fn invoke_instanceof<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, cpdtype: CPDType) -> PostInstructionAction<'gc> {
    let interpreter_jv = current_frame.pop(CClassName::object().into());
    let possibly_null = interpreter_jv.unwrap_object();
    let instance_of_class_type = cpdtype.unwrap_ref_type().clone();
    let res_int = instance_of_exit_impl(jvm, cpdtype, interpreter_jv.to_new_java_handle(jvm).unwrap_object().as_ref());
    current_frame.push(InterpreterJavaValue::Int(res_int));
    PostInstructionAction::Next {}
}

pub fn instance_of_impl<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, unwrapped: GcManagedObject<'gc>, instance_of_class_type: CPRefType) -> Result<(), WasException<'gc>> {
    match unwrapped.deref() {
        Array(array) => {
            match instance_of_class_type {
                CPRefType::Class(instance_of_class_name) => {
                    if instance_of_class_name == CClassName::serializable() || instance_of_class_name == CClassName::cloneable() {
                        unimplemented!() //todo need to handle serializable and the like
                    } else {
                        int_state.current_frame_mut().push(InterpreterJavaValue::Int(0))
                    }
                }
                CPRefType::Array { base_type, num_nested_arrs } => {
                    if todo!()/*a.deref() == &array.elem_type*/ {
                        int_state.current_frame_mut().push(InterpreterJavaValue::Int(1))
                    }
                }
            }
        }
        Object(object) => {
            match instance_of_class_type {
                CPRefType::Class(instance_of_class_name) => {
                    let mut temp: OpaqueFrame<'gc, 'l> = todo!();
                    let instanceof_class = check_resolved_class(jvm, &mut temp/*int_state.inner()*/, instance_of_class_name.into())?; //todo check if this should be here
                    let object_class = object.objinfo.class_pointer.clone();
                    if todo!()/*inherits_from(jvm, int_state, &object_class, &instanceof_class)?*/ {
                        int_state.current_frame_mut().push(InterpreterJavaValue::Int(1))
                    } else {
                        int_state.current_frame_mut().push(InterpreterJavaValue::Int(0))
                    }
                }
                CPRefType::Array { .. } => int_state.current_frame_mut().push(InterpreterJavaValue::Int(0)),
            }
        }
    };
    Ok(())
}

fn runtime_super_class<'gc>(jvm: &'gc JVMState<'gc>, inherits: &Arc<RuntimeClass<'gc>>) -> Option<Arc<RuntimeClass<'gc>>> {
    if inherits.view().super_name().is_some() { Some(assert_inited_or_initing_class(jvm, inherits.view().super_name().unwrap().into())) } else { None }
}

fn runtime_interface_class<'gc>(jvm: &'gc JVMState<'gc>, i: InterfaceView) -> Arc<RuntimeClass<'gc>> {
    let intf_name = i.interface_name();
    assert_inited_or_initing_class(jvm, intf_name.into())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from_cpdtype<'gc>(jvm: &'gc JVMState<'gc>, inherits: &Arc<RuntimeClass<'gc>>, parent: CPDType) -> bool {
    //todo it is questionable whether this logic should be here:
    if let RuntimeClass::Array(arr) = inherits.deref() {
        if parent == CClassName::object().into() || parent == CClassName::cloneable().into() || parent == CClassName::serializable().into() {
            return true;
        }
        if let Some(parent_arr) = parent.try_unwrap_array_type() {
            return inherits_from_cpdtype(jvm, &arr.sub_class, parent_arr);
        }
    }
    if inherits.cpdtype().is_primitive() {
        return false;
    }

    if inherits.view().name().to_cpdtype() == parent {
        return true;
    }
    let mut interfaces_match = false;

    for (_, i) in inherits.view().interfaces().enumerate() {
        let interface = runtime_interface_class(jvm, i);
        interfaces_match |= inherits_from_cpdtype(jvm, &interface, parent);
    }

    (match runtime_super_class(jvm, inherits) {
        None => false,
        Some(super_) => super_.view().name().to_cpdtype() == parent || inherits_from_cpdtype(jvm, &super_, parent),
    }) || interfaces_match
}

//todo dup
//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from<'gc>(jvm: &'gc JVMState<'gc>, inherits: &Arc<RuntimeClass<'gc>>, parent: &Arc<RuntimeClass<'gc>>) -> bool {
    //todo it is questionable whether this logic should be here:
    if let RuntimeClass::Array(arr) = inherits.deref() {
        if parent.cpdtype() == CClassName::object().into() || parent.cpdtype() == CClassName::cloneable().into() || parent.cpdtype() == CClassName::serializable().into() {
            return true;
        }
        if let RuntimeClass::Array(parent_arr) = parent.deref() {
            return inherits_from(jvm, &arr.sub_class.clone(), &parent_arr.sub_class.clone());
        }
    }
    if inherits.cpdtype().is_primitive() {
        return false;
    }

    if inherits.view().name() == parent.view().name() {
        return true;
    }
    let mut interfaces_match = false;

    for (_, i) in inherits.view().interfaces().enumerate() {
        let interface = runtime_interface_class(jvm, i);
        interfaces_match |= inherits_from(jvm, &interface, &parent);
    }

    (match runtime_super_class(jvm, inherits) {
        None => false,
        Some(super_) => super_.view().name() == parent.view().name() || inherits_from(jvm, &super_, parent),
    }) || interfaces_match
}
