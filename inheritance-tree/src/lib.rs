use std::borrow::Cow;
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

pub struct ClassNode<'gc>{
    left: InheritanceTreeNode<'gc>,
    right: InheritanceTreeNode<'gc>,
    class: Arc<RuntimeClass<'gc>>,
}

pub struct GrownNode<'gc> {
    left: InheritanceTreeNode<'gc>,
    right: InheritanceTreeNode<'gc>,
}

pub enum InheritanceTreeNode<'gc> {
    Class(ClassNode<'gc>),
    GrowthNode,
    GrownNode(GrownNode<'gc>),
}

impl<'gc> InheritanceTreeNode<'gc> {
    pub fn left_or_right(&self, left_or_right: LeftOrRight) -> Option<&InheritanceTreeNode<'gc>> {
        Some(match self {
            InheritanceTreeNode::Class(ClassNode{ left, right, class }) => {
                match left_or_right {
                    LeftOrRight::Left => {
                        left
                    }
                    LeftOrRight::Right => {
                        right
                    }
                }
            }
            InheritanceTreeNode::GrownNode(GrownNode{ left, right }) => {
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
    BitPath256 {
        bit_path: u256
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
            TreePath::BitPath256 { bit_path } => { todo!() }
            TreePath::Path { path } => {
                Cow::Borrowed(path)
            }
        }
    }
}

#[cfg(test)]
pub mod test{

}