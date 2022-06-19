#![feature(int_log)]

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;
use crate::class_list::ClassList;
use crate::paths::{InheritanceClassIDPath, InheritanceTreePath};

pub mod class_list;
pub mod paths;
#[cfg(test)]
pub mod test;
pub mod class_ids;

pub struct InheritanceTree{
    inner: RwLock<InheritanceTreeInner>
}

impl InheritanceTree {
    pub fn new(object_class_id: ClassID) -> Self {
        Self {
            inner: RwLock::new(InheritanceTreeInner::new(object_class_id))
        }
    }

    pub fn insert(&self, class_id_path: &InheritanceClassIDPath) {
        self.inner.write().unwrap().insert(class_id_path);
        unsafe {
            if libc::rand() < 100000000 {
                dbg!(self.inner.read().unwrap().max_bit_depth());
            }
        }
    }

    pub fn max_bit_depth(&self) -> usize{
        self.inner.read().unwrap().max_bit_depth()
    }
}

#[derive(Debug)]
pub struct InheritanceTreeInner {
    top_node: InheritanceTreeNode,
}

impl InheritanceTreeInner {
    pub fn new(object_class_id: ClassID) -> Self {
        Self {
            top_node: InheritanceTreeNode {
                class_id: object_class_id,
                sub_classes: ClassList::new_4_stage(),
                subclass_locations: SubClassLocations::new()
            }
        }
    }

    pub fn insert(&mut self, class_id_path: &InheritanceClassIDPath) {
        self.top_node.insert(class_id_path)
    }

    pub fn max_bit_depth(&self) -> usize{
        self.top_node.max_bit_depth()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubClassLocations{
    inner: HashMap<ClassID, Vec<Bit>>,
}

impl SubClassLocations{
    pub fn new() -> Self{
        Self{
            inner: HashMap::new()
        }
    }

    pub fn lookup_class_id_non_recursive(&self, class_id: ClassID) -> Option<InheritanceTreePath> {
        self.inner.get(&class_id)
            .map(|path|
                InheritanceTreePath::Borrowed {
                    inner: path.as_slice()
                }
            )
    }

    pub fn insert(&mut self, class_id: ClassID, path: InheritanceTreePath) {
        self.inner.insert(class_id, path.to_owned());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritanceTreeNode {
    class_id: ClassID,
    sub_classes: ClassList,
    subclass_locations: SubClassLocations
}

impl InheritanceTreeNode{
    pub fn new(class_id: ClassID) -> Self{
        Self{
            class_id,
            sub_classes: ClassList::new_4_stage(),
            subclass_locations: SubClassLocations::new()
        }
    }

    pub fn lookup_impl(&self, path: &InheritanceTreePath<'_>) -> Option<ClassID> {
        if path.is_empty() {
            return Some(self.class_id);
        }
        self.sub_classes.lookup_impl(path)
    }

    pub fn lookup_class_id_path(&self, class_id_path: &InheritanceClassIDPath) -> Option<InheritanceTreePath> {
        let (class_id, rest) = class_id_path.split_1();
        if let Some(already_present_path) = self.subclass_locations.lookup_class_id_non_recursive(class_id) {
            assert_eq!(self.sub_classes.lookup_impl(&already_present_path).unwrap(), class_id);
            let next_node = self.sub_classes.inheritance_tree_node_at_path_ref(&already_present_path);
            if rest.is_empty(){
                return Some(already_present_path)
            }else {
                return next_node.lookup_class_id_path(&rest)
            }
        }
        None
    }


    pub fn insert(&mut self, class_id_path: &InheritanceClassIDPath) {
        let (class_id, rest) = class_id_path.split_1();
        let already_present_path = if let Some(already_present_path) = self.subclass_locations.lookup_class_id_non_recursive(class_id) {
            already_present_path
        }else {
            let path = self.sub_classes.insert(class_id);
            self.sub_classes.inheritance_tree_node_at_path_mut(&path);
            self.subclass_locations.insert(class_id,path);
            self.subclass_locations.lookup_class_id_non_recursive(class_id).unwrap()
        };

        if rest.is_empty(){
            return
        }else {
            let next_node = self.sub_classes.inheritance_tree_node_at_path_mut(&already_present_path);
            next_node.insert(&rest)
        }
    }

    pub fn max_bit_depth(&self) -> usize{
        self.sub_classes.max_bit_depth()
    }

}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Bit {
    Set,
    UnSet,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClassID(u32);