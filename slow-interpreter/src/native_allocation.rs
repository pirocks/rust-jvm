use std::collections::HashMap;
use std::ffi::CString;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::RwLock;

use jvmti_jni_bindings::jint;

#[derive(Clone)]
pub enum AllocationType {
    /*
    VecLeak,
    */
    BoxLeak,
    Malloc,
    CString,
}

pub struct NativeAllocator {
    pub(crate) allocations: RwLock<HashMap<usize, AllocationType>>//todo impl default or something
}

unsafe impl Send for NativeAllocator {}

impl NativeAllocator {
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
        let mut guard = self.allocations.write().unwrap();
        guard.insert(res as usize, AllocationType::Malloc);
        res
    }

    pub fn allocate_box<'life, ElemType>(&self, vec: ElemType) -> &'life mut ElemType {
        let res = Box::leak(box vec);
        let mut guard = self.allocations.write().unwrap();
        guard.insert(res as *mut ElemType as *mut c_void as usize , AllocationType::BoxLeak);
        res
    }

    pub unsafe fn allocate_cstring(&self, cstr: CString) -> *mut i8 {
        let res = cstr.into_raw();
        let mut guard = self.allocations.write().unwrap();
        guard.insert(res as *mut c_void as usize, AllocationType::CString);
        res
    }

    pub unsafe fn allocate_string(&self, cstr: String) -> *mut i8 {
        self.allocate_cstring(CString::new(cstr).unwrap())
    }

    pub unsafe fn free(&self, ptr: *mut c_void) {
        let allocation_type = self.allocations.read().unwrap().get(&(ptr as usize)).unwrap().clone();
        match allocation_type {
            AllocationType::BoxLeak => {
                unimplemented!()
            }
            AllocationType::Malloc => {
                libc::free(ptr);
                self.allocations.write().unwrap().remove(&(ptr as usize));
            }
            AllocationType::CString => {
                CString::from_raw(ptr as *mut i8);
            }
        }
    }
}