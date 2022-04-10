use std::ffi::c_void;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;
use crate::{AllocatedHandle, InterpreterStateGuard, JavaValueCommon, JVMState};
use crate::java_values::ByAddressAllocatedObject;

pub fn dump_frame_contents<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, 'l>) {
    unsafe {
        if !IN_TO_STRING {
            dump_frame_contents_impl(jvm, int_state)
        }
    }
}

pub fn dump_frame_contents_impl<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>) {
    if !int_state.current_frame().full_frame_available(jvm) {
        let current_frame = int_state.current_frame();
        let local_vars = current_frame.local_var_simplified_types(jvm);
        eprint!("Local Vars:");
        unsafe {
            for (i, local_var_type) in local_vars.into_iter().enumerate() {
                let jv = current_frame.local_vars(jvm).raw_get(i as u16);
                eprint!("#{}: {:?}\t", i, jv as *const c_void)
            }
        }
        eprintln!();
        eprint!("Operand Stack:");
        let operand_stack_ref = current_frame.operand_stack(jvm);
        let operand_stack_types = operand_stack_ref.simplified_types();
        unsafe {
            for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
                let jv = operand_stack_ref.raw_get(i as u16);
                eprint!("#{}: {:?}\t", i, jv.object)
            }
        }
        eprintln!();
        return;
    }
    let local_var_types = int_state.current_frame().local_var_types(jvm);
    eprint!("Local Vars:");
    unsafe {
        for (i, local_var_type) in local_var_types.into_iter().enumerate() {
            match local_var_type.to_runtime_type() {
                RuntimeType::TopType => {
                    let jv = int_state.current_frame().local_vars(jvm).raw_get(i as u16);
                    eprint!("#{}: Top: {:?}\t", i, jv as *const c_void)
                }
                _ => {
                    let jv = int_state.current_frame().local_vars(jvm).get(i as u16, local_var_type.to_runtime_type());
                    if let Some(Some(obj)) = jv.try_unwrap_object_alloc() {
                        display_obj(jvm, int_state, i, obj);
                    } else {
                        let jv = int_state.current_frame().local_vars(jvm).get(i as u16, local_var_type.to_runtime_type());
                        eprint!("#{}: {:?}\t", i, jv.as_njv())
                    }
                }
            }
        }
    }
    eprintln!();
    let operand_stack_types = int_state.current_frame().operand_stack(jvm).types();
    // current_frame.ir_stack_entry_debug_print();
    eprint!("Operand Stack:");
    for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
        if let RuntimeType::TopType = operand_stack_type {
            panic!()
            /*let jv = operand_stack.raw_get(i as u16);
        eprint!("#{}: Top: {:?}\t", i, jv.object)*/
        } else {
            let jv = int_state.current_frame().operand_stack(jvm).get(i as u16, operand_stack_type.clone());
            if let Some(Some(obj)) = jv.try_unwrap_object_alloc() {
                display_obj(jvm, int_state, i, obj)
            } else {
                let jv = int_state.current_frame().operand_stack(jvm).get(i as u16, operand_stack_type);
                eprint!("#{}: {:?}\t", i, jv.as_njv())
            }
        }
    }
    eprintln!()
}

static mut IN_TO_STRING: bool = false;

fn display_obj<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, i: usize, obj: AllocatedHandle<'gc>) {
    let obj_type = obj.runtime_class(jvm).cpdtype();
    unsafe {
        if obj_type == CClassName::string().into() {
            let ptr = obj.ptr();
            let string = obj.cast_string();
            eprint!("#{}: {:?}(String:{:?})\t", i, ptr, string.to_rust_string_better(jvm).unwrap_or("malformed_string".to_string()))
        } else if obj_type == CClassName::class().into() {
            let class_short_name = match jvm.classes.read().unwrap().class_object_pool.get_by_left(&ByAddressAllocatedObject::LookupOnly(obj.as_allocated_obj().raw_ptr_usize())) {
                Some(class) => {
                    Some(class.cpdtype().jvm_representation(&jvm.string_pool))
                }
                None => None,
            };
            let ptr = obj.ptr();
            let ref_data = obj.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_reflectionData());
            eprint!("#{}: {:?}(Class:{:?} {:?})\t", i, ptr, class_short_name, ref_data.as_njv().to_native().object)
        } else {
            let ptr = obj.ptr();
            let save = IN_TO_STRING;
            IN_TO_STRING = true;
            if !save {
                eprint!("#{}: {:?}({})({})\t", i, ptr, obj_type.short_representation(&jvm.string_pool), ""/*obj.cast_object().to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm)*/);
            }
            IN_TO_STRING = save;
        }
    }
}
