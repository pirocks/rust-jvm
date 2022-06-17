#![feature(box_patterns)]

use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::DerefMut;

#[cfg(test)]
pub mod test;

pub mod attempt_tree;

pub trait InheritanceTreeElement<T: InheritanceTreeElement<T>>: Clone + PartialEq + Eq {
    fn is_same(&self, other: &T) -> bool {
        self.path() == other.path()
    }
    fn is_child_transitive(&self, child: &T) -> bool {
        let current_len = self.path().len();
        let child_len = child.path().len();
        if child_len <= current_len {
            return false;
        }
        &child.path()[0..current_len] == self.path().as_slice()
    }
    fn is_child_direct(&self, child: &T) -> bool {
        let current_len = self.path().len();
        let child_len = child.path().len();
        if child_len == current_len + 1 {
            return false;
        }
        &child.path()[0..current_len] == self.path().as_slice()
    }
    fn path(&self) -> Vec<T>;
}

pub struct InheritanceTree<T: Debug + InheritanceTreeElement<T>> {
    object_node: InheritanceTreeNode<T>,
}

impl<T: Debug + InheritanceTreeElement<T>> InheritanceTree<T> {
    pub fn node_at_path(&self, path: TreePath) -> &InheritanceTreeNode<T> {
        let mut current_node = &self.object_node;
        let path = path.to_left_or_right_path();
        for left_or_right in path.as_slice().into_iter().cloned() {
            current_node = current_node.left_or_right(left_or_right).unwrap()
        }
        return current_node;
    }

    pub fn find_path(&mut self, elem: T) -> TreePath {
        todo!()
    }

    pub fn insert_at_path(&mut self, path: TreePath, elem: T) {
        todo!()
    }

    pub fn new() -> Self {
        Self {
            object_node: InheritanceTreeNode::GrowthNode
        }
    }
}

pub struct ClassNode<T: Debug + InheritanceTreeElement<T>> {
    left: Box<InheritanceTreeNode<T>>,
    right: Box<InheritanceTreeNode<T>>,
    class: T,
}

pub trait HasLeftAndRight<T: Debug + InheritanceTreeElement<T>> {
    fn left(&self) -> &InheritanceTreeNode<T>;
    fn right(&self) -> &InheritanceTreeNode<T>;
    fn left_right_mut(&mut self) -> (&mut InheritanceTreeNode<T>, &mut InheritanceTreeNode<T>);
    fn left_mut(&mut self) -> &mut InheritanceTreeNode<T> {
        self.left_right_mut().0
    }
    fn right_mut(&mut self) -> &mut InheritanceTreeNode<T> {
        self.left_right_mut().1
    }

    fn find_path_impl(&self, current_tree_path: TreePath, target_elem: &T) -> Option<TreePath> {
        match &self.left() {
            InheritanceTreeNode::Class(elem) => {
                if elem.class.is_same(target_elem) {
                    return Some(current_tree_path.push(LeftOrRight::Left));
                }
                if elem.class.is_child_transitive(&elem.class){
                    return elem.find_path_impl(current_tree_path, target_elem);
                }
            }
            InheritanceTreeNode::GrowthNode => {}
            InheritanceTreeNode::GrownNode(grown) => {
                if let Some(res) = grown.find_path_impl(current_tree_path.clone(),target_elem){
                    return Some(res)
                }
            }
        };
        match self.right() {
            InheritanceTreeNode::Class(elem) => {
                if elem.class.is_same(&target_elem) {
                    return Some(current_tree_path.push(LeftOrRight::Left));
                }
                if elem.class.is_child_transitive(&elem.class){
                    return elem.find_path_impl(current_tree_path, target_elem);
                }
                None
            }
            InheritanceTreeNode::GrowthNode => {
                None
            }
            InheritanceTreeNode::GrownNode(grown) => {
                grown.find_path_impl(current_tree_path,target_elem)
            }
        }
    }

    fn num_growth_points(&self) -> u64 {
        (match &self.left() {
            InheritanceTreeNode::Class(class) => class.num_growth_points(),
            InheritanceTreeNode::GrowthNode => 1,
            InheritanceTreeNode::GrownNode(node) => node.num_growth_points()
        }) + (
            match &self.right() {
                InheritanceTreeNode::Class(class) => class.num_growth_points(),
                InheritanceTreeNode::GrowthNode => 1,
                InheritanceTreeNode::GrownNode(node) => node.num_growth_points()
            })
    }

    fn find_free_growth_nodes_impl<'a>(&'a mut self, res: &mut Vec<&'a mut InheritanceTreeNode<T>>) {
        let (left, right) = self.left_right_mut();
        match left {
            InheritanceTreeNode::Class(_) => {}
            InheritanceTreeNode::GrowthNode => {
                res.push(left);
            }
            InheritanceTreeNode::GrownNode(grown_node) => {
                grown_node.find_free_growth_nodes_impl(res);
            }
        }
        match right {
            InheritanceTreeNode::Class(_) => {}
            InheritanceTreeNode::GrowthNode => {
                res.push(right);
            }
            InheritanceTreeNode::GrownNode(grown_node) => {
                grown_node.find_free_growth_nodes_impl(res);
            }
        }
    }
}

impl<T: Debug + InheritanceTreeElement<T>> HasLeftAndRight<T> for ClassNode<T> {
    fn left(&self) -> &InheritanceTreeNode<T> {
        &self.left
    }

    fn right(&self) -> &InheritanceTreeNode<T> {
        &self.right
    }

    fn left_right_mut(&mut self) -> (&mut InheritanceTreeNode<T>, &mut InheritanceTreeNode<T>) {
        (&mut self.left, &mut self.right)
    }
}

impl<T: Debug + InheritanceTreeElement<T>> ClassNode<T> {
    pub fn num_direct_children(&self) -> u64 {
        self.left.num_direct_children_impl() + self.right.num_direct_children_impl()
    }

    fn find_free_growth_nodes<'a>(&'a mut self) -> Vec<&'a mut InheritanceTreeNode<T>> {
        let mut res = vec![];
        let left = self.left.deref_mut();
        match left {
            InheritanceTreeNode::Class(_) => {}
            InheritanceTreeNode::GrowthNode => {
                res.push(left);
            }
            InheritanceTreeNode::GrownNode(grown) => {
                grown.find_free_growth_nodes_impl(&mut res);
            }
        }

        let right = self.right.deref_mut();
        match right {
            InheritanceTreeNode::Class(_) => {}
            InheritanceTreeNode::GrowthNode => {
                res.push(right);
            }
            InheritanceTreeNode::GrownNode(grown) => {
                grown.find_free_growth_nodes_impl(&mut res);
            }
        }
        res
    }

    pub fn insert_subclass(&mut self, to_insert: T) {
        todo!()
    }
}

pub struct GrownNode<T: Debug + InheritanceTreeElement<T>> {
    left: Box<InheritanceTreeNode<T>>,
    right: Box<InheritanceTreeNode<T>>,
}

impl<T: Debug + InheritanceTreeElement<T>> HasLeftAndRight<T> for GrownNode<T> {
    fn left(&self) -> &InheritanceTreeNode<T> {
        &self.left
    }

    fn right(&self) -> &InheritanceTreeNode<T> {
        &self.right
    }

    fn left_right_mut(&mut self) -> (&mut InheritanceTreeNode<T>, &mut InheritanceTreeNode<T>) {
        (&mut self.left, &mut self.right)
    }
}

impl<T: Debug + InheritanceTreeElement<T>> GrownNode<T> {}

pub enum InheritanceTreeNode<T: Debug + InheritanceTreeElement<T>> {
    Class(ClassNode<T>),
    GrowthNode,
    GrownNode(GrownNode<T>),
}

impl<T: Debug + InheritanceTreeElement<T>> InheritanceTreeNode<T> {
    pub fn left_or_right(&self, left_or_right: LeftOrRight) -> Option<&InheritanceTreeNode<T>> {
        Some(match self {
            InheritanceTreeNode::Class(ClassNode { left, right, class }) => {
                match left_or_right {
                    LeftOrRight::Left => {
                        left
                    }
                    LeftOrRight::Right => {
                        right
                    }
                }
            }
            InheritanceTreeNode::GrownNode(GrownNode { left, right }) => {
                match left_or_right {
                    LeftOrRight::Left => {
                        left
                    }
                    LeftOrRight::Right => {
                        right
                    }
                }
            }
            InheritanceTreeNode::GrowthNode => {
                return None;
            }
        })
    }

    fn num_direct_children_impl(&self) -> u64 {
        match self {
            InheritanceTreeNode::Class(ClassNode { left, right, class }) => {
                1
            }
            InheritanceTreeNode::GrowthNode => {
                0
            }
            InheritanceTreeNode::GrownNode(GrownNode { left, right }) => {
                left.num_direct_children_impl() + right.num_direct_children_impl()
            }
        }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum LeftOrRight {
    Left,
    Right,
}

impl LeftOrRight {
    pub fn from_bool(from: bool) -> Self {
        match from {
            true => Self::Left,
            false => Self::Right
        }
    }
}

#[derive(Clone)]
pub enum TreePath {
    BitPath64 {
        bit_path: u64
    },
    BitPath128 {
        bit_path: u128
    },
    Path {
        path: Vec<LeftOrRight>
    },
}

impl TreePath {
    pub fn to_left_or_right_path(&self) -> Cow<Vec<LeftOrRight>> {
        match self {
            TreePath::BitPath64 { bit_path } => {
                let mut res = vec![];
                for bit_i in 0..64 {
                    let bit = (*bit_path >> bit_i) & 0x1;
                    res.push(LeftOrRight::from_bool(bit != 0))
                }
                Cow::Owned(res)
            }
            TreePath::BitPath128 { bit_path } => todo!(),
            TreePath::Path { path } => {
                Cow::Borrowed(path)
            }
        }
    }

    pub fn push(self, left_or_right: LeftOrRight) -> TreePath{
        match self {
            TreePath::BitPath64 { .. } => {
                todo!()
            }
            TreePath::BitPath128 { .. } => {
                todo!()
            }
            TreePath::Path { mut path,.. } => {
                path.push(left_or_right);
                TreePath::Path { path }
            }
        }
    }
}

