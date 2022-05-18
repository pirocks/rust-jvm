use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ChangeableConstID(u64);

pub struct ChangeableConst {
    inner: *mut AtomicU64,
}

impl ChangeableConst {
    pub fn new(default: u64) -> Self {
        Self {
            inner: Box::into_raw(box AtomicU64::new(default))
        }
    }
}

impl Drop for ChangeableConst {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.inner)) }
    }
}

pub struct ChangeableConsts {
    inner: HashMap<ChangeableConstID, ChangeableConst>,
}

impl ChangeableConsts {
    pub fn new() -> Self {
        Self {
            inner: Default::default()
        }
    }

    pub fn change_const64(&self, id: ChangeableConstID, new_val: u64) {
        unsafe {
            self.inner.get(&id).unwrap().inner.as_ref().unwrap().store(new_val, Ordering::SeqCst);
            assert_eq!(self.inner.get(&id).unwrap().inner.as_ref().unwrap().load(Ordering::SeqCst), new_val);
        }
    }

    pub fn add_const64(&mut self, new_val: u64) -> ChangeableConstID {
        let next_id = ChangeableConstID(self.inner.len() as u64);
        self.inner.insert(next_id, ChangeableConst::new(new_val));
        next_id
    }

    pub fn raw_ptr(&self, id: ChangeableConstID) -> *mut AtomicU64 {
        self.inner.get(&id).unwrap().inner
    }
}