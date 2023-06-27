use std::ffi::c_void;
use std::ptr::{NonNull, null_mut};

use libc::{MAP_ANONYMOUS, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use nix::errno::errno;
use nonnull_const::NonNullConst;

pub struct OwnedNativeStack {
    pub mmaped_top: NonNull<c_void>,
    pub(crate) mmaped_bottom: NonNull<c_void>,
    pub max_stack: usize,
}

#[derive(Debug)]
pub struct CannotAllocateStack;

impl OwnedNativeStack {
    #[allow(unreachable_code)]
    pub fn new() -> Result<Self, CannotAllocateStack> {
        pub const MAX_STACK: usize = 10 * 1024 * 1024 * 1024;
        let page_size = 4096;
        let mmaped_top = unsafe { libc::mmap(null_mut(), MAX_STACK + page_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
        if mmaped_top as i64 == -1 {
            dbg!(nix::errno::Errno::from_i32(errno()));
            panic!();
            return Err(CannotAllocateStack {});
        }
        unsafe {
            Ok(Self {
                mmaped_top: NonNull::new(mmaped_top.add(MAX_STACK)).unwrap(),
                mmaped_bottom: NonNull::new(mmaped_top).unwrap(),
                max_stack: MAX_STACK,
            })
        }
    }

    pub unsafe fn validate_frame_pointer(&self, frame_pointer: NonNullConst<c_void>) {
        if self.mmaped_top.as_ptr().offset_from(frame_pointer.as_ptr()) > self.max_stack as isize || frame_pointer.as_ptr() > self.mmaped_top.as_ptr() {
            dbg!(self.mmaped_top);
            dbg!(frame_pointer);
            panic!()
        }
    }
}