use std::ffi::c_void;

use gc_memory_layout_common::memory_regions::MemoryRegions;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{AllocatedHandle, JavaValueCommon, JVMState};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::frames::HasFrame;
use crate::java_values::ByAddressAllocatedObject;

pub fn dump_frame_contents<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'l>) {
    unsafe {
        if !IN_TO_STRING {
            dump_frame_contents_impl(jvm, int_state)
        }
    }
}

pub fn dump_frame_contents_impl<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, '_>) {
    if !int_state.full_frame_available(jvm) {
        let local_vars = int_state.local_var_simplified_types(jvm);
        eprint!("Local Vars:");
        unsafe {
            for (i, local_var_type) in local_vars.into_iter().enumerate() {
                let jv = int_state.raw_local_var_get(i as u16);
                eprint!("#{}: {:?}\t", i, jv as *const c_void)
            }
        }
        eprintln!();
        eprint!("Operand Stack:");
        let operand_stack_types = int_state.operand_stack_simplified_types(jvm);
        unsafe {
            for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
                let jv = int_state.raw_operand_stack_get(i as u16);
                eprint!("#{}: {:?}\t", i, jv as *const c_void)
            }
        }
        eprintln!();
        return;
    }
    let local_var_types = int_state.local_var_types(jvm);
    eprint!("Local Vars:");
    unsafe {
        for (i, local_var_type) in local_var_types.into_iter().enumerate() {
            match local_var_type.to_runtime_type() {
                RuntimeType::TopType => {
                    let jv = int_state.raw_local_var_get(i as u16);
                    eprint!("#{}: Top: {:?}\t", i, jv as *const c_void)
                }
                _ => {
                    let jv = int_state.local_get_handle(i as u16, local_var_type.to_runtime_type());
                    if let Some(Some(obj)) = jv.try_unwrap_object_alloc() {
                        display_obj(jvm, int_state, i, obj);
                    } else {
                        let jv = int_state.local_get_handle(i as u16, local_var_type.to_runtime_type());
                        eprint!("#{}: {:?}\t", i, jv.as_njv())
                    }
                }
            }
        }
    }
    eprintln!();
    let operand_stack_types = int_state.operand_stack_types(jvm);
    // current_frame.ir_stack_entry_debug_print();
    eprint!("Operand Stack:");
    for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
        let jv = int_state.os_get_from_start(i as u16, operand_stack_type.to_runtime_type());
        if let Some(Some(obj)) = jv.try_unwrap_object_alloc() {
            display_obj(jvm, int_state, i, obj)
        } else {
            let jv = int_state.os_get_from_start(i as u16, operand_stack_type.to_runtime_type());
            eprint!("#{}: {:?}\t", i, jv.as_njv())
        }
    }
    eprintln!()
}

static mut IN_TO_STRING: bool = false;

#[allow(unused)]
fn display_obj<'gc>(jvm: &'gc JVMState<'gc>, _int_state: &mut JavaExitFrame<'gc, '_>, i: usize, obj: AllocatedHandle<'gc>) {
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
        } /*else if obj_type == CClassName::concurrent_hash_map().into() {
            obj.cast_concurrent_hash_map().debug_print_table(jvm);
            //todo display hashtable entrys
        } else if obj_type == CClassName::hashtable_entry().into() {
            let ptr = obj.ptr();
            let entry = obj.cast_entry();
            let next = entry.next(jvm);
            eprint!("#{}: {:?}(hashtable entry:{:?})\t", i, ptr, next.unwrap_object().map(|obj|obj.ptr()))
        }*/ /*else if obj_type == CClassName::big_integer().into(){
            let ptr = obj.ptr();
            let big_integer = obj.cast_big_integer();
            dbg!(big_integer.signum(jvm));
            dbg!(big_integer.mag(jvm).unwrap_object_nonnull().unwrap_array().array_iterator().collect_vec());
            let as_string = big_integer.to_string(jvm, _int_state).unwrap().unwrap().to_rust_string(jvm);
            eprint!("#{}: {:?}(biginteger:{})\t", i, ptr, as_string);
        }*/ else {
            if obj_type.short_representation(&jvm.string_pool).as_str() == "StringBuilder" {
                let region_header = MemoryRegions::find_object_region_header(obj.ptr());
                // dbg!("");
                // dbg!(region_header as *mut RegionHeader);
                // dbg!(region_header.vtable_ptr);
                // dbg!(region_header.itable_ptr);
            }
            let ptr = obj.ptr();
            eprint!("#{}: {:?}({})({})\t", i, ptr, obj_type.short_representation(&jvm.string_pool), "");
        }
    }
}
