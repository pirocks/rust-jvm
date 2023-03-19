use std::collections::HashMap;

pub struct StrictBiHashMap<Left, Right>{
    left_to_right: HashMap<Left, Right>,
    right_to_left: HashMap<Right, Left>
}

impl <Left, Right> StrictBiHashMap<Left, Right>{
    pub fn new() -> Self{
        Self{
            left_to_right: HashMap::new(),
            right_to_left: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize{
        assert_eq!(self.right_to_left.len(), self.left_to_right.len());
        self.left_to_right.len()
    }
}

