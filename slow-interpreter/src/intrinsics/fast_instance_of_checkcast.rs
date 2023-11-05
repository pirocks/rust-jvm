use std::ffi::c_void;
use std::ptr::NonNull;
use gc_memory_layout_common::memory_regions::{MemoryRegions};
use inheritance_tree::ClassID;
use inheritance_tree::paths::BitPath256;
use jvmti_jni_bindings::{jclass};
use crate::hidden_field_constants::CLASS_BIT_PATH_POINTER_OFFSET;

#[repr(C)]
enum InstanceOfUnsafeResult {
    False = 0,
    True = 1,
    Unknown = 2,
}


unsafe extern "C" fn array_store_exception_check(array_ref: *mut c_void, value: *mut c_void) -> InstanceOfUnsafeResult{
    let array_region = MemoryRegions::find_object_region_header_raw(NonNull::new(array_ref).unwrap()).as_ref().unwrap();
    let array_elem_class = array_region.class_pointer_cache;
    instance_of_class_object(value, array_elem_class)
}

unsafe extern "C" fn instance_of_class_object<'gc>(ptr_in: *mut c_void, class_object: jclass) -> InstanceOfUnsafeResult {
    let inheritance_bit_path = (class_object.offset(CLASS_BIT_PATH_POINTER_OFFSET as isize) as *const BitPath256).read();
    instance_of_class(ptr_in, &inheritance_bit_path as *const BitPath256)
}

unsafe extern "C" fn instance_of_class(ptr_in: *mut c_void, inheritance_bit_path: *const BitPath256) -> InstanceOfUnsafeResult {
    //todo use class id to traverse an inheritance tree if other options fail.
    let object_header = MemoryRegions::find_object_region_header_raw(NonNull::new(ptr_in).unwrap()).as_ref().unwrap();
    let object_bit_path = match object_header.inheritance_bit_path_ptr.as_ref() {
        Some(x) => x,
        None => {
            return InstanceOfUnsafeResult::Unknown
        },
    };
    let class_bit_path = match inheritance_bit_path.as_ref() {
        Some(x) => x,
        None => {
            return InstanceOfUnsafeResult::Unknown
        },
    };
    if class_bit_path.is_subpath_of(object_bit_path){
        InstanceOfUnsafeResult::True
    }else {
        InstanceOfUnsafeResult::False
    }
}

unsafe extern "C" fn instance_of_interface(ptr_in: *mut c_void, class_id: ClassID) -> InstanceOfUnsafeResult {
    let object_header = MemoryRegions::find_object_region_header_raw(NonNull::new(ptr_in).unwrap()).as_ref().unwrap();
    for i in 0..object_header.interface_ids_list_len{
        if object_header.interface_ids_list.offset(i as isize).read() == class_id{
            return InstanceOfUnsafeResult::True
        }
    }
    return InstanceOfUnsafeResult::False
}
