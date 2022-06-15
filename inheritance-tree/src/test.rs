use itertools::Itertools;
use crate::{ClassNode, GrownNode, InheritanceTree, InheritanceTreeElement, InheritanceTreeNode, LeftOrRight, TreePath};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TestTreeElement {
    elems: Vec<u64>,
}

impl InheritanceTreeElement<TestTreeElement> for TestTreeElement {
    fn path(&self) -> Vec<TestTreeElement> {
        (0..self.elems.len()).map(|len| {
            TestTreeElement {
                elems: self.elems[0..len].iter().cloned().collect_vec()
            }
        }).collect_vec()
    }
}

#[test]
pub fn test_insert() {
    let a = TestTreeElement { elems: vec![1] };
    let b = TestTreeElement { elems: vec![4] };
    let c = TestTreeElement { elems: vec![5] };
    let c1 = TestTreeElement { elems: vec![5, 6] };
    let c2 = TestTreeElement { elems: vec![5, 7] };
    let c3 = TestTreeElement { elems: vec![5, 8] };
    // let mut _test_tree = InheritanceTree::new();
    // test_tree.insert(a);
    // test_tree.insert(b);
    // test_tree.insert(c);
    // test_tree.insert(c1);
    // test_tree.insert(c2);
    // test_tree.insert(c3);
    todo!()
}


#[test]
pub fn test_lookup() {
    let top_class = TestTreeElement {
        elems: vec![]
    };
    let class_1 = TestTreeElement {
        elems: vec![1]
    };
    let class_2 = TestTreeElement {
        elems: vec![2]
    };
    let class_3 = TestTreeElement {
        elems: vec![3]
    };
    let class_1_1 = TestTreeElement {
        elems: vec![1, 1]
    };
    let left = Box::new(InheritanceTreeNode::GrownNode(GrownNode {
        left: Box::new(InheritanceTreeNode::Class(ClassNode {
            left: Box::new(InheritanceTreeNode::GrowthNode),
            right: Box::new(InheritanceTreeNode::GrowthNode),
            class: class_2.clone(),
        })),
        right: Box::new(InheritanceTreeNode::Class(ClassNode {
            left: Box::new(InheritanceTreeNode::GrowthNode),
            right: Box::new(InheritanceTreeNode::GrowthNode),
            class: class_3.clone(),
        })),
    }));
    let inheritance_tree = InheritanceTree {
        object_node: InheritanceTreeNode::Class(ClassNode {
            left,
            right: Box::new(InheritanceTreeNode::Class(ClassNode {
                left: Box::new(InheritanceTreeNode::Class(ClassNode {
                    left: Box::new(InheritanceTreeNode::GrowthNode),
                    right: Box::new(InheritanceTreeNode::GrowthNode),
                    class: class_1_1.clone(),
                })),
                right: Box::new(InheritanceTreeNode::GrowthNode),
                class: class_1.clone(),
            })),
            class: top_class,
        })
    };
    match inheritance_tree.node_at_path(TreePath::Path { path: vec![LeftOrRight::Left, LeftOrRight::Right] }) {
        InheritanceTreeNode::Class(class) => {
            assert_eq!(&class.class, &class_3);
        }
        _ => panic!()
    };
    match inheritance_tree.node_at_path(TreePath::Path { path: vec![LeftOrRight::Left, LeftOrRight::Left] }) {
        InheritanceTreeNode::Class(class) => {
            assert_eq!(&class.class, &class_2);
        }
        _ => panic!()
    };
    match inheritance_tree.node_at_path(TreePath::Path { path: vec![LeftOrRight::Right, LeftOrRight::Left] }) {
        InheritanceTreeNode::Class(class) => {
            assert_eq!(&class.class, &class_1_1);
        }
        _ => panic!()
    };
    match inheritance_tree.node_at_path(TreePath::Path { path: vec![LeftOrRight::Right] }) {
        InheritanceTreeNode::Class(class) => {
            assert_eq!(&class.class, &class_1);
        }
        _ => panic!()
    }
}