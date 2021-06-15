#![feature(box_syntax)]

use std::ops::{Deref, DerefMut, Index};
use std::pin::Pin;

// pub const INITIAL_SIZE: usize = 100;

pub struct AddOnlyStaticVec<T> {
    inner: Vec<Pin<Box<T>>>,
}

impl<T> AddOnlyStaticVec<T> {
    pub fn push(&'static mut self, elem: T) {
        self.inner.push(Box::pin(elem));
    }

    pub fn get(&'static self, i: usize) -> &T {
        self.inner.as_slice()[i].deref()
    }
}

#[derive(Debug)]
pub struct Elem {}

#[test]
pub fn test() {
    // let add_only: &'static mut AddOnlyStaticVec<Elem> = Box::leak(box AddOnlyStaticVec { inner: vec![] });
    // add_only.push(Elem {});
    // let val_ref: &'static Elem = add_only.get(0);
    // add_only.push(Elem{})
}