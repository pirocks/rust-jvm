use crate::class_list::{ClassListNode, STAGE_1_TARGET_CAPACITY, STAGE_2_TARGET_CAPACITY, STAGE_3_TARGET_CAPACITY};
use crate::{ClassID, ClassList, InheritanceClassIDPath, InheritanceTreeInner};

#[test]
pub fn class_list_insert_many() {
    let mut class_list = ClassList::new_4_stage();
    for i in 0..(STAGE_3_TARGET_CAPACITY + STAGE_2_TARGET_CAPACITY + STAGE_1_TARGET_CAPACITY + 10){
        class_list.insert(ClassID(i));
    }
}



#[test]
pub fn class_list_insert_at_depth_works_1() {
    let mut class_list_node = ClassListNode::GrowthNode {};
    class_list_node.try_insert_at_depth_impl(2, ClassID(0), &mut vec![]).unwrap();
    class_list_node.try_insert_at_depth_impl(2, ClassID(1), &mut vec![]).unwrap();
    class_list_node.try_insert_at_depth_impl(2, ClassID(2), &mut vec![]).unwrap();
    class_list_node.try_insert_at_depth_impl(2, ClassID(3), &mut vec![]).unwrap();
    let should_be_err = class_list_node.try_insert_at_depth_impl(2, ClassID(4), &mut vec![]);
    should_be_err.err().unwrap();
}

#[test]
pub fn class_list_insert_at_depth_works_2() {
    let mut class_list_node = ClassListNode::GrowthNode {};
    class_list_node.try_insert_at_depth_impl(1, ClassID(0), &mut vec![]).unwrap();
    class_list_node.try_insert_at_depth_impl(1, ClassID(1), &mut vec![]).unwrap();
    let should_be_err = class_list_node.try_insert_at_depth_impl(1, ClassID(4), &mut vec![]);
    should_be_err.err().unwrap();
}

#[test]
pub fn class_list_insert_at_depth_works_3() {
    let mut class_list_node = ClassListNode::GrowthNode {};
    class_list_node.try_insert_at_depth_impl(0, ClassID(0), &mut vec![]).unwrap();
    let should_be_err = class_list_node.try_insert_at_depth_impl(0, ClassID(4), &mut vec![]);
    should_be_err.err().unwrap();
}


#[test]
pub fn inheritance_tree_build_up() {
    let object_class = ClassID(0);
    let mut inheritance_tree = InheritanceTreeInner::new(object_class);
    let class_a = ClassID(1);
    let class_a_a = ClassID(2);
    let class_a_b = ClassID(3);
    let class_a_c = ClassID(4);
    let class_b = ClassID(5);
    let class_b_a = ClassID(6);
    let a_a_path = InheritanceClassIDPath::Owned { inner: vec![object_class, class_a, class_a_a] };
    let a_path = InheritanceClassIDPath::Owned { inner: vec![object_class, class_a] };
    let a_b_path = InheritanceClassIDPath::Owned { inner: vec![object_class, class_a, class_a_b] };
    let a_c_path = InheritanceClassIDPath::Owned { inner: vec![object_class, class_a, class_a_c] };
    let b_path = InheritanceClassIDPath::Owned { inner: vec![object_class, class_b] };
    let b_a_path = InheritanceClassIDPath::Owned { inner: vec![object_class, class_b, class_b_a] };
    inheritance_tree.insert(&a_a_path);
    inheritance_tree.insert(&a_path);
    inheritance_tree.insert(&a_b_path);
    inheritance_tree.insert(&a_c_path);
    inheritance_tree.insert(&b_path);
    inheritance_tree.insert(&b_a_path);
    // three class list levels + max of 2 stage 2 elems.
    assert!(inheritance_tree.max_bit_depth() <= 2*3 + 2 + 2);
    inheritance_tree.top_node.lookup_class_id_path(&a_a_path);

}


#[test]
pub fn inheritance_tree_bug_0_maybe() {
    let object_class = ClassID(0);
    let a_class = ClassID(5);
    let b_class = ClassID(8);
    let c_class = ClassID(10);
    let d_class = ClassID(11);
    let mut inheritance_tree = InheritanceTreeInner::new(object_class);
    inheritance_tree.insert(&vec![object_class].into());
    inheritance_tree.insert(&vec![object_class, a_class].into());
    inheritance_tree.insert(&vec![object_class].into());
    inheritance_tree.insert(&vec![object_class,b_class].into());
    inheritance_tree.insert(&vec![object_class,c_class].into());
    inheritance_tree.insert(&vec![object_class,d_class].into());
}