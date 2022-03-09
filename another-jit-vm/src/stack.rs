use std::cell::RefCell;
use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::null_mut;
use libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use nix::errno::errno;

thread_local! {
    static ONE_PER_THREAD: RefCell<usize> = RefCell::new(0);
}

pub struct OwnedNativeStack{
    pub mmaped_top: *mut c_void,
    pub(crate) mmaped_bottom: *mut c_void,
    pub max_stack: usize,
}

impl OwnedNativeStack{
    pub fn new() -> Self {
        ONE_PER_THREAD.with(|refcell|{
            *refcell.borrow_mut() += 1;
            if refcell.borrow().deref() != &1 {
                // panic!()
            } else {}
        });
        pub const MAX_STACK: usize = 10 * 1024 * 1024 * 1024;
        let page_size = 4096;
        let mmaped_top = unsafe { libc::mmap(null_mut(), MAX_STACK + page_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS , -1, 0) };
        if mmaped_top as i64 == -1{
            dbg!(nix::errno::Errno::from_i32(errno()));
            panic!()
        }
        unsafe {
            Self {
                mmaped_top: mmaped_top.add(MAX_STACK),
                mmaped_bottom: mmaped_top,
                max_stack: MAX_STACK,
            }
        }
    }

    pub unsafe fn validate_frame_pointer(&self, frame_pointer: *const c_void) {
        if self.mmaped_top.offset_from(frame_pointer) > self.max_stack as isize || frame_pointer > self.mmaped_top {
            dbg!(self.mmaped_top);
            dbg!(frame_pointer);
            panic!()
        }
    }
}