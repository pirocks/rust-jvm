use std::collections::HashMap;
use std::sync::{RwLock};

use crate::compressed_classfile::CMethodDescriptor;
use crate::compressed_classfile::names::MethodName;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct MethodShape {
    pub name: MethodName,
    pub desc: CMethodDescriptor,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MethodShapeID(pub u64);

struct MethodShapeIDsInner {
    id_to_shape: HashMap<MethodShapeID, MethodShape>,
    shape_to_id: HashMap<MethodShape, MethodShapeID>,
}

pub struct MethodShapeIDs {
    inner: RwLock<MethodShapeIDsInner>,
}

impl MethodShapeIDs {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(MethodShapeIDsInner {
                id_to_shape: Default::default(),
                shape_to_id: Default::default(),
            })
        }
    }

    pub fn lookup_method_shape_id(&self, method_shape: MethodShape) -> MethodShapeID {
        let mut guard = self.inner.write().unwrap();
        let right_len = Self::consistency_check(&guard);
        let new_id = MethodShapeID(right_len as u64);
        match guard.shape_to_id.get(&method_shape) {
            None => {
                guard.shape_to_id.insert(method_shape.clone(), new_id);
                guard.id_to_shape.insert(new_id, method_shape.clone());
                new_id
            }
            Some(res) => *res
        }
    }

    fn consistency_check(guard: &MethodShapeIDsInner) -> usize {
        let right_len = guard.shape_to_id.len();
        let left_len = guard.id_to_shape.len();
        assert_eq!(right_len, left_len);
        right_len
    }

    pub fn lookup_method_shape(&self, method_shape_id: MethodShapeID) -> MethodShape{
        let guard = self.inner.read().unwrap();
        Self::consistency_check(&guard);
        guard.id_to_shape.get(&method_shape_id).unwrap().clone()
    }
}
