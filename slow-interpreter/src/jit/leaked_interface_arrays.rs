use std::collections::{BTreeSet, HashMap};
use itertools::Itertools;
use inheritance_tree::ClassID;

pub struct InterfaceArrays{
    inner: HashMap<BTreeSet<ClassID>, (*const ClassID, usize)>
}

impl InterfaceArrays {
    pub fn new() -> Self{
        Self{
            inner: HashMap::new()
        }
    }

    pub fn add_interfaces(&mut self, interfaces: BTreeSet<ClassID>) -> (*const ClassID, usize){
        return match self.inner.get(&interfaces) {
            None => {
                self.inner.insert(interfaces.clone(), Self::leak_interface_array(interfaces.clone()));
                self.add_interfaces(interfaces)
            }
            Some(res) => {
                *res
            }
        }
    }

    fn leak_interface_array(interfaces: BTreeSet<ClassID>) -> (*const ClassID, usize) {
        let mut interfaces = interfaces.into_iter().collect_vec();
        interfaces.shrink_to_fit();
        let (ptr, len, capacity) = interfaces.into_raw_parts();
        assert_eq!(len, capacity);
        (ptr, len)
    }


    pub fn lookup_interfaces(&self, interfaces: &BTreeSet<ClassID>) -> (*const ClassID, usize){
        *self.inner.get(interfaces).unwrap()
    }
}

