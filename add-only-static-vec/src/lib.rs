#![feature(box_syntax)]

use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;
use std::mem;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::sync::RwLock;

use static_rc::StaticRc;

// pub const INITIAL_SIZE: usize = 100;

pub type AddOnlyVecIDType = u32;

pub struct AddOnlyVec<T> {
    //todo make this more type safe
    inner: RwLock<Vec<Box<T>>>,
}

impl<T> AddOnlyVec<T> {
    pub fn push(&self, elem: T) {
        self.inner.write().unwrap().push(box elem);
    }

    pub(crate) fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }

    pub fn new() -> Self {
        Self {
            inner: RwLock::new(vec![]),
        }
    }
}

impl<T> std::ops::Index<usize> for AddOnlyVec<T> {
    type Output = T;

    fn index<'l>(&'l self, index: usize) -> &'l Self::Output {
        let guard = self.inner.read().unwrap();
        let res: &T = guard[index].deref();
        unsafe { transmute::<&T, &'l T>(res) } //this is safe b/c we never free any boxes until self goes out of scope
    }
}

impl<T> std::ops::IndexMut<usize> for AddOnlyVec<T> {
    fn index_mut<'l>(&'l mut self, index: usize) -> &'l mut Self::Output {
        let mut guard = self.inner.write().unwrap();
        let res: &mut T = guard[index].deref_mut();
        unsafe { transmute::<&mut T, &'l mut T>(res) } //this is safe b/c we never free any boxes until self goes out of scope
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct AddOnlyId(pub AddOnlyVecIDType);

struct AddOnlyIdMapInner<T>
    where
        T: PartialEq + Eq + Hash,
{
    map: HashMap<StaticRc<T, 1, 2>, AddOnlyId>,
    owner: AddOnlyVec<Option<StaticRc<T, 1, 2>>>, //todo I could put an add only vec here but the static rcs are much cooler
}

pub struct AddOnlyIdMap<T>
    where
        T: PartialEq + Eq + Hash,
{
    inner: RwLock<AddOnlyIdMapInner<T>>,
}

impl<T> Drop for AddOnlyIdMap<T>
    where
        T: PartialEq + Eq + Hash,
{
    fn drop(&mut self) {
        let mut guard = self.inner.write().unwrap();
        let AddOnlyIdMapInner { map, owner } = guard.deref_mut();
        map.drain().for_each(|(key, value)| {
            mem::drop(StaticRc::<T, 2, 2>::join(
                owner[value.0 as usize].take().unwrap(),
                key,
            ));
        })
    }
}

impl<T> AddOnlyIdMap<T>
    where
        T: PartialEq + Eq + Hash,
{
    pub fn push(&self, elem: T) -> AddOnlyId {
        let mut inner = self.inner.write().unwrap();
        let next_id = inner.map.len();
        let elem_rc = StaticRc::new(elem);
        let (left, right) = StaticRc::split(elem_rc);
        return match inner.map.get(&left) {
            None => {
                assert_eq!(inner.owner.len(), next_id);
                inner.owner.push(Some(right));
                inner
                    .map
                    .insert(left, AddOnlyId(next_id.try_into().unwrap()));
                AddOnlyId(next_id.try_into().unwrap())
            }
            Some(res) => {
                StaticRc::<T, 2, 2>::join(left, right);
                *res
            }
        };
    }

    pub fn lookup<'l>(&'l self, id: AddOnlyId) -> &'l T {
        let inner = self.inner.read().unwrap();
        let res = inner.owner[id.0 as usize].as_ref().unwrap().deref();
        unsafe { transmute::<&T, &'l T>(res) } //this is safe b/c we never free any boxes until self goes out of scope
    }

    pub fn new() -> Self {
        Self {
            inner: RwLock::new(AddOnlyIdMapInner {
                map: Default::default(),
                owner: AddOnlyVec::new(),
            }),
        }
    }
}