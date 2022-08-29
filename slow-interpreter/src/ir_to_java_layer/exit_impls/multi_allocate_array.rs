use std::ffi::c_void;
use std::num::NonZeroU8;
use std::ptr::NonNull;

use itertools::Itertools;

use another_jit_vm_ir::IRVMExitAction;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::NativeJavaValue;

use crate::{check_initing_or_inited_class, InterpreterStateGuard, JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, UnAllocatedObject, UnAllocatedObjectArray};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java_values::default_value;

#[inline(never)]
pub fn multi_allocate_array<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, elem_type: CPDTypeID, num_arrays: u8, len_start: *const i64, return_to_ptr: *const c_void, res_address: *mut NonNull<c_void>) -> IRVMExitAction {
    let elem_type = *jvm.cpdtype_table.read().unwrap().get_cpdtype(elem_type);
    let elem_type = elem_type.unwrap_non_array();
    let array_type = CPDType::Array { base_type: elem_type, num_nested_arrs: NonZeroU8::new(num_arrays).unwrap() };
    let mut lens = vec![];
    unsafe {
        for len_index in 0..num_arrays {
            let offsetted_ptr = len_start.sub(len_index as usize);
            lens.push(offsetted_ptr.cast::<i32>().read());
        }
    }
    assert_inited_or_initing_class(jvm, elem_type.to_cpdtype());
    let default = default_value(elem_type.to_cpdtype());
    let rc = check_initing_or_inited_class(jvm, /*int_state*/todo!(), array_type).unwrap();
    let res = multi_new_array_impl(jvm, rc.cpdtype(),lens.as_slice() ,default.as_njv());
    unsafe { res_address.cast::<NativeJavaValue<'gc>>().write(res.to_native()) }
    std::mem::forget(res);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}


pub fn multi_new_array_impl<'gc>(jvm: &'gc JVMState<'gc>, cpdtype: CPDType, dimensions: &[i32], default: NewJavaValue<'gc, '_>) -> NewJavaValueHandle<'gc> {
    if dimensions.is_empty() {
        assert!(!cpdtype.is_array());
        return default.to_handle_discouraged();
    } else {
        assert!(cpdtype.is_array());
        let first_dimension = dimensions[0];
        let mut elems = vec![];
        for _ in 0..first_dimension {
            elems.push(multi_new_array_impl(jvm, cpdtype.unwrap_array_type(), &dimensions[1..], default.clone()));
        }
        NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class: assert_inited_or_initing_class(jvm, cpdtype), elems: elems.iter().map(|elem| elem.as_njv()).collect_vec() })))
    }
}