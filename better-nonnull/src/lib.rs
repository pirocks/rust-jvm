use std::ptr::NonNull;

#[repr(transparent)]
pub struct BetterNonNull<T: ?Sized>(pub NonNull<T>);

impl <T> BetterNonNull<T>{
    pub fn new(ptr: *mut T) -> Option<Self>{
        Some(BetterNonNull(NonNull::new(ptr)?))
    }

    pub unsafe fn offset(&self, amount: isize) -> Option<BetterNonNull<T>>{
        Some(BetterNonNull(NonNull::new(self.0.as_ptr().offset(amount))?))
    }
}


impl <T> From<NonNull<T>> for BetterNonNull<T>{
    fn from(value: NonNull<T>) -> Self {
        BetterNonNull(value)
    }
}