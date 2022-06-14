#![feature(box_patterns)]

use std::borrow::Cow;
use std::ops::DerefMut;
use std::sync::Arc;
use runtime_class_stuff::RuntimeClass;


pub struct InheritanceTree<'gc> {
    object_node: InheritanceTreeNode<'gc>,
}

impl<'gc> InheritanceTree<'gc> {
    pub fn node_at_path(&self, path: TreePath) -> &InheritanceTreeNode<'gc> {
        let mut current_node = &self.object_node;
        let path = path.to_left_or_right_path();
        for left_or_right in path.as_slice().into_iter().cloned() {
            current_node = current_node.left_or_right(left_or_right).unwrap()
        }
        return current_node;
    }
}

pub struct ClassNode<'gc> {
    left: Box<InheritanceTreeNode<'gc>>,
    right: Box<InheritanceTreeNode<'gc>>,
    class: Arc<RuntimeClass<'gc>>,
}

pub trait HasLeftAndRight<'gc> {
    fn left(&self) -> &InheritanceTreeNode<'gc>;
    fn right(&self) -> &InheritanceTreeNode<'gc>;
    fn left_right_mut(&mut self) -> (&mut InheritanceTreeNode<'gc>, &mut InheritanceTreeNode<'gc>);
    fn left_mut(&mut self) -> &mut InheritanceTreeNode<'gc> {
        self.left_right_mut().0
    }
    fn right_mut(&mut self) -> &mut InheritanceTreeNode<'gc> {
        self.left_right_mut().1
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

    fn find_free_growth_nodes_impl<'a>(&'a mut self, res: &mut Vec<&'a mut InheritanceTreeNode<'gc>>) {
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

impl<'gc> HasLeftAndRight<'gc> for ClassNode<'gc> {
    fn left(&self) -> &InheritanceTreeNode<'gc> {
        &self.left
    }

    fn right(&self) -> &InheritanceTreeNode<'gc> {
        &self.right
    }

    fn left_right_mut(&mut self) -> (&mut InheritanceTreeNode<'gc>, &mut InheritanceTreeNode<'gc>) {
        (&mut self.left, &mut self.right)
    }
}

impl<'gc> ClassNode<'gc> {
    pub fn num_direct_children(&self) -> u64 {
        self.left.num_direct_children_impl() + self.right.num_direct_children_impl()
    }

    fn find_free_growth_nodes<'a>(&'a mut self) -> Vec<&'a mut InheritanceTreeNode<'gc>> {
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

    pub fn insert_subclass(&mut self, to_insert: Arc<RuntimeClass<'gc>>) {
        todo!()
    }
}

pub struct GrownNode<'gc> {
    left: Box<InheritanceTreeNode<'gc>>,
    right: Box<InheritanceTreeNode<'gc>>,
}

impl<'gc> HasLeftAndRight<'gc> for GrownNode<'gc> {
    fn left(&self) -> &InheritanceTreeNode<'gc> {
        &self.left
    }

    fn right(&self) -> &InheritanceTreeNode<'gc> {
        &self.right
    }

    fn left_right_mut(&mut self) -> (&mut InheritanceTreeNode<'gc>, &mut InheritanceTreeNode<'gc>) {
        (&mut self.left, &mut self.right)
    }
}

impl<'gc> GrownNode<'gc> {}

pub enum InheritanceTreeNode<'gc> {
    Class(ClassNode<'gc>),
    GrowthNode,
    GrownNode(GrownNode<'gc>),
}

impl<'gc> InheritanceTreeNode<'gc> {
    pub fn left_or_right(&self, left_or_right: LeftOrRight) -> Option<&InheritanceTreeNode<'gc>> {
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
}

#[cfg(test)]
pub mod test {}