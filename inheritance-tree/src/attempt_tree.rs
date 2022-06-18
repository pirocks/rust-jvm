use std::borrow::Cow;
use cow_vec_item::CowVec;
use crate::{ClassNode, TreePath};
use crate::attempt_tree::Bit::Set;

pub struct InheritanceTree {
    top_node: InheritanceTreeNode,
}

impl InheritanceTree {
    pub fn new(object_class_id: ClassID) -> Self {
        Self {
            top_node: InheritanceTreeNode {
                class_id: object_class_id,
                sub_classes: ClassList::new(),
            }
        }
    }

    pub fn insert(&mut self, class_id_path: InheritanceClassIDPath) {

    }

    pub fn lookup(&self, path: InheritanceTreePath<'_>) -> ClassID {
        self.top_node.lookup_impl(path).unwrap()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ClassID(u32);

pub struct InheritanceClassIDPath<'a> {
    inner: &'a [ClassID]
}

pub struct InheritanceTreeNode {
    class_id: ClassID,
    sub_classes: ClassList,
}

impl InheritanceTreeNode {
    pub fn lookup_impl(&self, path: InheritanceTreePath<'_>) -> Option<ClassID> {
        if path.inner.is_empty() {
            return Some(self.class_id);
        }
        self.sub_classes.lookup_impl(path)
    }
}

pub struct ClassList {
    top_node: ClassListNode,
}

impl ClassList {
    pub const STAGE_1_TARGET_CAPACITY: usize = 1;
    pub const STAGE_2_TARGET_CAPACITY: usize = 4;
    pub const STAGE_3_TARGET_CAPACITY: usize = 8192;
    pub const STAGE_4_TARGET_CAPACITY: usize = 2 ^ 32;

    pub fn new() -> ClassList {
        ClassList {
            top_node: ClassListNode::GrownNode {
                set: Box::new(ClassListNode::GrownNode {
                    set: Box::new(ClassListNode::GrowthNode),
                    unset: Box::new(ClassListNode::GrowthNode),
                }),
                unset: Box::new(ClassListNode::GrownNode {
                    set: Box::new(ClassListNode::GrowthNode),
                    unset: Box::new(ClassListNode::GrowthNode),
                }),
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ClassListNode {
    GrownNode {
        set: Box<ClassListNode>,
        unset: Box<ClassListNode>,
    },
    LeafNode {
        sub_node: InheritanceTreeNode
    },
    GrowthNode,
}

impl ClassListNode {
    pub fn new_with_capacity(capacity_log: u8) -> Self {
        let mut current = Self::GrowthNode;
        for _ in 0..capacity_log {
            current = Self::GrownNode { set: Box::new(current.clone()), unset: Box::new(current) }
        }
        current
    }

    pub fn lookup_impl(&self, path: InheritanceTreePath) -> Option<ClassID> {
        match self {
            ClassListNode::GrownNode { set, unset } => {
                let (current_elem, rest) = path.split_1();
                match current_elem {
                    Bit::Set => {
                        set.lookup_impl(rest)
                    }
                    Bit::UnSet => {
                        unset.lookup_impl(rest)
                    }
                }
            }
            ClassListNode::LeafNode { sub_node } => {
                return Some(sub_node.class_id);
            }
            ClassListNode::GrowthNode => {
                None
            }
        }
    }
}

pub struct InheritanceTreePath<'a> {
    inner: &'a [Bit],
}

impl<'a> InheritanceTreePath<'a> {
    pub fn split_1(&self) -> (Bit, InheritanceTreePath<'a>) {
        (*self.inner[0], InheritanceTreePath { inner: &self.inner[1..] })
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Bit {
    Set,
    UnSet,
}

#[cfg(test)]
pub mod test {}