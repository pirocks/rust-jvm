use std::cell::UnsafeCell;

use jvmti_jni_bindings::JavaPrimitiveType;

//todo this could be abstracted into a stack backed bump allocator

pub const MAX_ARGS: usize = 256;

pub struct ArgBoxesToFree {
    arg_boxes: [UnsafeCell<u64>; MAX_ARGS],
    current_i: usize,
}

impl ArgBoxesToFree {
    pub fn new() -> Self {
        Self {
            arg_boxes: [0; MAX_ARGS].map(|_| UnsafeCell::new(0u64)),
            current_i: 0,
        }
    }

    pub fn new_generic<T: JavaPrimitiveType>(&mut self, val: T) -> ArgBox<T> {
        let to_write_index = self.current_i;
        self.current_i += 1;
        unsafe {
            self.arg_boxes[to_write_index].get().cast::<T>().write(val);
            ArgBox(self.arg_boxes[to_write_index].get().cast::<T>().as_mut().unwrap())
        }
    }
}


pub struct ArgBox<'a, T: ?Sized>(&'a mut T);

impl<'a, T> ArgBox<'a, T> {
    pub fn as_ref(&self) -> &T {
        self.0
    }
}