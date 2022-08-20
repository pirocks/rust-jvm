use std::cmp::max;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use crate::{Bit, ClassID, InheritanceTreeNode};
use crate::paths::InheritanceTreePath;

pub const STAGE_1_TARGET_CAPACITY: u32 = 1;
pub const STAGE_2_TARGET_CAPACITY: u32 = 1 << 2;
pub const STAGE_3_TARGET_CAPACITY: u32 = 1 << 13;
pub const STAGE_4_TARGET_CAPACITY: u32 = 1 << 31;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Stage {
    Stage1,
    Stage2,
    Stage3,
    Stage4,
}

impl Stage {
    pub fn stage_depth(&self) -> u32 {
        match self {
            Stage::Stage1 => STAGE_1_TARGET_CAPACITY.ilog2(),
            Stage::Stage2 => STAGE_2_TARGET_CAPACITY.ilog2(),
            Stage::Stage3 => STAGE_3_TARGET_CAPACITY.ilog2(),
            Stage::Stage4 => STAGE_4_TARGET_CAPACITY.ilog2(),
        }
    }

    pub fn stage_path(&self) -> InheritanceTreePath {
        InheritanceTreePath::Owned {
            inner: match self {
                Stage::Stage1 => vec![Bit::Set, Bit::Set],
                Stage::Stage2 => vec![Bit::Set, Bit::UnSet],
                Stage::Stage3 => vec![Bit::UnSet, Bit::Set],
                Stage::Stage4 => vec![Bit::UnSet, Bit::UnSet],
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassList {
    stage_1_utilization: u32,
    stage_2_utilization: u32,
    stage_3_utilization: u32,
    stage_4_utilization: u32,
    top_node: ClassListNode,
}


impl ClassList {
    pub fn new_4_stage() -> ClassList {
        ClassList {
            stage_1_utilization: 0,
            stage_2_utilization: 0,
            stage_3_utilization: 0,
            stage_4_utilization: 0,
            top_node: ClassListNode::GrownNode {
                set: Box::new(ClassListNode::GrownNode {
                    set: Box::new(ClassListNode::GrowthNode),
                    unset: Box::new(ClassListNode::GrowthNode),
                }),
                unset: Box::new(ClassListNode::GrownNode {
                    set: Box::new(ClassListNode::GrowthNode),
                    unset: Box::new(ClassListNode::GrowthNode),
                }),
            },
        }
    }


    fn stage_impl(&mut self, stage: Stage) -> &mut ClassListNode {
        self.top_node.node_at_path_mut(&stage.stage_path())
    }

    fn stage_1(&mut self) -> &mut ClassListNode {
        self.stage_impl(Stage::Stage1)
    }

    fn stage_2(&mut self) -> &mut ClassListNode {
        self.stage_impl(Stage::Stage2)
    }

    fn stage_3(&mut self) -> &mut ClassListNode {
        self.stage_impl(Stage::Stage3)
    }

    fn stage_4(&mut self) -> &mut ClassListNode {
        self.stage_impl(Stage::Stage4)
    }

    fn stage_utilization_mut(&mut self, stage: Stage) -> &mut u32 {
        match stage {
            Stage::Stage1 => &mut self.stage_1_utilization,
            Stage::Stage2 => &mut self.stage_2_utilization,
            Stage::Stage3 => &mut self.stage_3_utilization,
            Stage::Stage4 => &mut self.stage_4_utilization
        }
    }

    fn current_stage_to_insert(&mut self) -> Stage {
        //todo this seems rather inefficient in that its doing a lot of counting
        if *self.stage_utilization_mut(Stage::Stage1) >= STAGE_1_TARGET_CAPACITY {
            if *self.stage_utilization_mut(Stage::Stage2) >= STAGE_2_TARGET_CAPACITY {
                if *self.stage_utilization_mut(Stage::Stage3) >= STAGE_3_TARGET_CAPACITY {
                    if *self.stage_utilization_mut(Stage::Stage4) >= STAGE_4_TARGET_CAPACITY {
                        panic!()
                    } else {
                        Stage::Stage4
                    }
                } else {
                    Stage::Stage3
                }
            } else {
                Stage::Stage2
            }
        } else {
            Stage::Stage1
        }
    }

    pub fn stage_mut(&mut self, stage: Stage) -> &mut ClassListNode {
        match stage {
            Stage::Stage1 => self.stage_1(),
            Stage::Stage2 => self.stage_2(),
            Stage::Stage3 => self.stage_3(),
            Stage::Stage4 => self.stage_4()
        }
    }

    pub fn insert<'any>(&mut self, class_id: ClassID) -> InheritanceTreePath<'any> {
        let stage_to_insert = self.current_stage_to_insert();
        *self.stage_utilization_mut(stage_to_insert) += 1;
        let stage_depth = stage_to_insert.stage_depth();
        let stage_path = stage_to_insert.stage_path();
        let stage_to_insert = self.stage_mut(stage_to_insert);
        let sub_path = stage_to_insert.insert_at_depth(stage_depth, class_id);
        assert_eq!(sub_path.as_slice().len(), stage_depth as usize);
        stage_path.concat(&sub_path)
    }


    pub fn inheritance_tree_node_at_path_ref(&self, path: &InheritanceTreePath<'_>) -> &InheritanceTreeNode {
        if let ClassListNode::LeafNode { sub_node } = self.node_at_path_ref(path) {
            sub_node.deref()
        } else {
            panic!()
        }
    }

    pub fn inheritance_tree_node_at_path_mut(&mut self, path: &InheritanceTreePath<'_>) -> &mut InheritanceTreeNode {
        if let ClassListNode::LeafNode { sub_node } = self.node_at_path_mut(path) {
            sub_node.deref_mut()
        } else {
            panic!()
        }
    }

    pub fn node_at_path_ref(&self, path: &InheritanceTreePath<'_>) -> &ClassListNode {
        self.top_node.node_at_path_ref(path)
    }

    pub fn node_at_path_mut(&mut self, path: &InheritanceTreePath<'_>) -> &mut ClassListNode {
        self.top_node.node_at_path_mut(path)
    }

    pub fn lookup_impl(&self, path: &InheritanceTreePath<'_>) -> Option<ClassID> {
        match self.node_at_path_ref(path) {
            ClassListNode::LeafNode { sub_node } => {
                Some(sub_node.class_id)
            }
            ClassListNode::GrownNode { .. } |
            ClassListNode::GrowthNode => None
        }
    }

    pub fn max_bit_depth(&self) -> usize {
        self.top_node.max_bit_depth()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClassListNode {
    GrownNode {
        set: Box<ClassListNode>,
        unset: Box<ClassListNode>,
    },
    LeafNode {
        sub_node: Box<InheritanceTreeNode>
    },
    GrowthNode,
}

#[derive(Copy, Clone, Debug)]
pub struct NoSpaceLeft;

impl ClassListNode {
    pub fn insert_at_depth(&mut self, depth: u32, to_insert: ClassID) -> InheritanceTreePath {
        self.try_insert_at_depth(depth, to_insert).unwrap()
    }

    pub fn try_insert_at_depth(&mut self, depth: u32, to_insert: ClassID) -> Result<InheritanceTreePath, NoSpaceLeft> {
        let mut path = Vec::with_capacity(depth as usize);
        self.try_insert_at_depth_impl(depth, to_insert, &mut path)?;
        Ok(InheritanceTreePath::Owned { inner: path })
    }

    pub fn try_insert_at_depth_impl(&mut self, depth: u32, to_insert: ClassID, path_so_far: &mut Vec<Bit>) -> Result<(), NoSpaceLeft> {
        //todo this seems rather inefficient in that its doing linear search
        if depth == 0 {
            match self {
                ClassListNode::GrowthNode => {
                    *self = ClassListNode::LeafNode {
                        sub_node: Box::new(InheritanceTreeNode::new(to_insert))
                    };
                    return Ok(());
                }
                ClassListNode::LeafNode { .. } => {
                    return Err(NoSpaceLeft);
                }
                _ => {
                    panic!()
                }
            }
        }
        match self {
            ClassListNode::LeafNode { .. } => {
                panic!()
            }
            ClassListNode::GrownNode { set, unset } => {
                let save = path_so_far.len();
                path_so_far.push(Bit::Set);
                if let Ok(()) = set.try_insert_at_depth_impl(depth - 1, to_insert, path_so_far) {
                    return Ok(());
                }
                path_so_far.resize(save, Bit::Set);
                path_so_far.push(Bit::UnSet);
                return unset.try_insert_at_depth_impl(depth - 1, to_insert, path_so_far);
            }
            ClassListNode::GrowthNode => {
                *self = ClassListNode::GrownNode {
                    set: Box::new(ClassListNode::GrowthNode),
                    unset: Box::new(ClassListNode::GrowthNode),
                };
                return self.try_insert_at_depth_impl(depth, to_insert, path_so_far);
            }
        }
    }

    fn tree_size_inclusive_is_above_threshold_impl(&self, threshold: &mut u32) -> bool {
        *threshold -= 1;
        if *threshold == 0 {
            return true;
        }
        if let ClassListNode::GrownNode { set, unset } = self {
            if set.tree_size_inclusive_is_above_threshold_impl(threshold) {
                return true;
            }
            if unset.tree_size_inclusive_is_above_threshold_impl(threshold) {
                return true;
            }
        }
        false
    }

    pub fn tree_size_inclusive_is_above_threshold(&self, mut threshold: u32) -> bool {
        self.tree_size_inclusive_is_above_threshold_impl(&mut threshold)
    }

    pub fn new_with_capacity(capacity_log: u8) -> Self {
        let mut current = Self::GrowthNode;
        for _ in 0..capacity_log {
            current = Self::GrownNode { set: Box::new(current.clone()), unset: Box::new(current) }
        }
        current
    }

    pub fn lookup_impl(&self, path: &InheritanceTreePath) -> Option<ClassID> {
        match self {
            ClassListNode::GrownNode { set, unset } => {
                let (current_elem, rest) = path.split_1();
                match current_elem {
                    Bit::Set => {
                        set.lookup_impl(&rest)
                    }
                    Bit::UnSet => {
                        unset.lookup_impl(&rest)
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

    pub fn node_at_path_ref(&self, path: &InheritanceTreePath<'_>) -> &ClassListNode {
        match self {
            ClassListNode::GrownNode { set, unset } => {
                let (current_elem, rest) = path.split_1();
                match current_elem {
                    Bit::Set => {
                        set.node_at_path_ref(&rest)
                    }
                    Bit::UnSet => {
                        unset.node_at_path_ref(&rest)
                    }
                }
            }
            ClassListNode::LeafNode { .. } |
            ClassListNode::GrowthNode => {
                if path.is_empty() {
                    return self;
                }
                panic!()
            }
        }
    }

    pub fn node_at_path_mut(&mut self, path: &InheritanceTreePath<'_>) -> &mut ClassListNode {
        if path.is_empty() {
            return self;
        }
        match self {
            ClassListNode::GrownNode { set, unset } => {
                let (current_elem, rest) = path.split_1();
                match current_elem {
                    Bit::Set => {
                        set.node_at_path_mut(&rest)
                    }
                    Bit::UnSet => {
                        unset.node_at_path_mut(&rest)
                    }
                }
            }
            ClassListNode::LeafNode { .. } |
            ClassListNode::GrowthNode => {
                dbg!(self);
                dbg!(path);
                panic!()
            }
        }
    }

    pub fn max_bit_depth(&self) -> usize {
        match self {
            ClassListNode::GrownNode { set, unset } => {
                max(set.max_bit_depth(), unset.max_bit_depth()) + 1
            }
            ClassListNode::LeafNode { sub_node } => {
                sub_node.max_bit_depth()
            }
            ClassListNode::GrowthNode => {
                0
            }
        }
    }
}

