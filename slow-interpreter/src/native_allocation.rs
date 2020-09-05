use std::collections::HashMap;
use std::ffi::CString;
use std::intrinsics::transmute;
use std::os::raw::c_void;
use std::sync::RwLock;

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
    pub(crate) allocations: RwLock<HashMap<usize, AllocationType>>//todo impl defualt or something
}

unsafe impl Send for NativeAllocator {}

impl NativeAllocator {
    pub unsafe fn allocate_malloc(&self, size: libc::size_t) -> *mut c_void {
        let res = libc::malloc(size);
        let mut guard = self.allocations.write().unwrap();
        guard.insert(transmute(res), AllocationType::Malloc);
        res
    }

    pub fn allocate_box<'life, ElemType>(&self, vec: ElemType) -> &'life mut ElemType {
        let res = Box::leak(box vec);
        let mut guard = self.allocations.write().unwrap();
        guard.insert(unsafe { transmute(res as *mut ElemType as *mut c_void) }, AllocationType::BoxLeak);
        res
    }

    pub unsafe fn allocate_cstring(&self, cstr: CString) -> *mut i8 {
        let res = cstr.into_raw();
        let mut guard = self.allocations.write().unwrap();
        guard.insert(transmute(res as *mut c_void), AllocationType::Malloc);
        res
    }

    pub unsafe fn free(&self, ptr: *mut c_void) {
        let allocation_type = self.allocations.read().unwrap().get(&transmute(ptr)).unwrap().clone();
        match allocation_type {
            AllocationType::BoxLeak => {
                unimplemented!()
            }
            AllocationType::Malloc => {
                libc::free(ptr);
                self.allocations.write().unwrap().remove(&transmute(ptr));
            }
            AllocationType::CString => {
                CString::from_raw(ptr as *mut i8);
            }
        }
    }
}