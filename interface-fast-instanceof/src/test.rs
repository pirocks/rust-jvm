use class_ids::ClassID;

use crate::{ObjectFastInstanceOfTables};

pub struct ClassIdIncrementer{
    current_class_id: u32
}

impl ClassIdIncrementer{
    pub fn new() -> Self{
        Self{
            current_class_id: 0
        }
    }
    pub fn next_id(&mut self) -> ClassID{
        let res = self.current_class_id;
        self.current_class_id += 1;
        ClassID(res)
    }
}

#[test]
pub fn simple_test() {
    let mut class_ids = ClassIdIncrementer::new();
    let object_class_1 = class_ids.next_id();
    let object_class_1_interfaces = vec![class_ids.next_id(), class_ids.next_id(), class_ids.next_id(), class_ids.next_id(), class_ids.next_id(), class_ids.next_id()];
    let object_class_2 = class_ids.next_id();
    let object_class_2_interfaces = vec![ClassID(1), ClassID(2), ClassID(3), ClassID(4), ClassID(5), ClassID(6), class_ids.next_id(), class_ids.next_id()];
    let mut tables = ObjectFastInstanceOfTables::new();
    tables.add_class(object_class_1, object_class_1_interfaces.clone());
    for interface in object_class_1_interfaces.clone() {
        assert_eq!(tables.interface_instance_of_fast(object_class_1, interface), Some(true));
    }
    tables.add_class(object_class_2, object_class_2_interfaces.clone());
    for interface in object_class_1_interfaces.clone() {
        assert_eq!(tables.interface_instance_of_fast(object_class_1, interface), Some(true));
    }
    for interface in object_class_2_interfaces.clone() {
        assert_eq!(tables.interface_instance_of_fast(object_class_2, interface), Some(true));
    }
}


