use std::collections::HashMap;
use std::hash::Hash;

pub struct StrictBiHashMap<Left, Right> {
    left_to_right: HashMap<Left, Right>,
    right_to_left: HashMap<Right, Left>,
}

impl<Left: Clone + Hash + Eq, Right: Clone + Hash + Eq> StrictBiHashMap<Left, Right> {
    pub fn new() -> Self {
        Self {
            left_to_right: HashMap::new(),
            right_to_left: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool{
        assert_eq!(self.right_to_left.len(), self.left_to_right.len());
        self.left_to_right.is_empty()
    }

    pub fn len(&self) -> usize {
        assert_eq!(self.right_to_left.len(), self.left_to_right.len());
        self.left_to_right.len()
    }

    pub fn insert(&mut self, left: Left, right: Right) {
        assert!(self.left_to_right.insert(left.clone(), right.clone()).is_none());
        assert!(self.right_to_left.insert(right, left).is_none());
    }
}

