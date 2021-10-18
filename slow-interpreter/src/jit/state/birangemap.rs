use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;
use std::os::raw::c_void;

use rangemap::RangeMap;
use rangemap::map::IntoIter;

#[derive(Debug)]
pub struct BiRangeMap<K: Clone + Ord + Eq, V: Clone + Eq + Hash> {
    left_to_right: RangeMap<K, V>,
    right_to_left: HashMap<V, Range<K>>,
}

impl<K: Clone + Ord + Eq + SingleElementRangeable<K>, V: Clone + Eq + Hash> BiRangeMap<K, V> {
    pub fn new() -> Self {
        Self {
            left_to_right: Default::default(),
            right_to_left: Default::default(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.left_to_right.insert(key.to_single_element_range(), value.clone());
        let (new_range, _) = self.left_to_right.get_key_value(&key).unwrap();
        self.right_to_left.insert(value, new_range.clone());
    }

    pub fn insert_range(&mut self, key: Range<K>, value: V) {
        self.left_to_right.insert(key.clone(), value.clone());
        let (new_range, _) = self.left_to_right.get_key_value(&key.start).unwrap();
        self.right_to_left.insert(value, new_range.clone());
    }

    pub fn get<'l>(&self, key: &'l K) -> Option<&V> {
        self.left_to_right.get(key)
    }

    pub fn get_reverse(&self, key: &V) -> Option<&Range<K>> {
        self.right_to_left.get(key)
    }
}

impl<K: Clone + Ord + Eq, V: Clone + Ord + Eq + Hash> IntoIterator for BiRangeMap<K, V> {
    type Item = (Range<K>, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.left_to_right.into_iter()
    }
}

pub trait SingleElementRangeable<T: SingleElementRangeable<T>> {
    fn to_single_element_range(&self) -> Range<T>;
}


impl SingleElementRangeable<*mut c_void> for *mut c_void {
    fn to_single_element_range(&self) -> Range<*mut c_void> {
        unsafe { *self..self.offset(1) }
    }
}
