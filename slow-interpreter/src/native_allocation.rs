use std::collections::HashMap;
use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Mutex, RwLock};

use jvmti_jni_bindings::jint;
use sketch_jvm_version_of_utf8::JVMString;

#[derive(Clone)]
pub enum AllocationType {
    Malloc,
    CString,
}

pub struct NativeAllocator {
    allocations: Mutex<HashMap<usize, AllocationType>>,
}

impl NativeAllocator {
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(HashMap::new())
        }
    }

    pub unsafe fn allocate_and_write_vec<T>(&self, data: Vec<T>, len_ptr: *mut jint, data_ptr: *mut *mut T) {
        let len = data.len();
        let size = size_of::<T>() * len;
        data_ptr.write(self.allocate_malloc(size) as *mut T);
        assert!(len < i32::MAX as usize);
        if len_ptr != null_mut() {
            len_ptr.write(len as i32);
        }
        for (i, elem) in data.into_iter().enumerate() {
            data_ptr.read().add(i).write(elem)
        }
    }

    pub unsafe fn allocate_malloc(&self, size: libc::size_t) -> *mut c_void {
        let res = libc::malloc(size);
        let mut guard = self.allocations.lock().unwrap();
        guard.insert(res as usize, AllocationType::Malloc);
        res
    }

    pub unsafe fn allocate_cstring(&self, cstr: CString) -> *mut i8 {
        let res = cstr.into_raw();
        let mut guard = self.allocations.lock().unwrap();
        guard.insert(res as *mut c_void as usize, AllocationType::CString);
        res
    }

    pub unsafe fn allocate_string(&self, cstr: String) -> *mut i8 {
        self.allocate_cstring(CString::new(cstr).unwrap())
    }

    pub unsafe fn allocate_modified_string(&self, cstr: String) -> *mut i8 {
        let buf = JVMString::from_regular_string(cstr.as_str()).buf.clone();
        let mut len = 0;
        let mut ptr: *mut u8 = null_mut();
        self.allocate_and_write_vec(buf, &mut len as *mut jint, &mut ptr as *mut *mut u8);
        ptr as *mut i8
    }

    pub unsafe fn free(&self, ptr: *mut c_void) {
        if ptr.is_null() {
            return; // this is needed to be correct w/ malloc of zero size
        }
        let allocation_type = match self.allocations.lock().unwrap().get(&(ptr as usize)) {
            Some(x) => x,
            None => return,
        }
            .clone();
        match allocation_type {
            AllocationType::Malloc => {
                libc::free(ptr);
            }
            AllocationType::CString => {
                let _ = CString::from_raw(ptr as *mut i8);
            }
        }
        self.allocations.lock().unwrap().remove(&(ptr as usize));
    }
}


#[cfg(test)]
pub mod tests {
    use std::ffi::{c_void, CString};
    use crate::native_allocation::NativeAllocator;

    #[test]
    pub fn allocation_round_trip() {
        unsafe {
            let allocator = NativeAllocator::new();
            let res = allocator.allocate_string("test string".to_string());
            allocator.free(res as *mut c_void);
            let res = allocator.allocate_cstring(CString::new("test string").unwrap());
            allocator.free(res as *mut c_void);
            let res = allocator.allocate_malloc(100);
            allocator.free(res);
            assert!(allocator.allocations.lock().unwrap().is_empty());
        }
    }
}
