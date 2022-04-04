use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::compressed_classfile::{CMethodDescriptor, CPDTypeOrderWrapper};
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

    pub fn lookup_method_shape(&self, method_shape_id: MethodShapeID) -> MethodShape {
        let guard = self.inner.read().unwrap();
        Self::consistency_check(&guard);
        guard.id_to_shape.get(&method_shape_id).unwrap().clone()
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ShapeOrderWrapper<'l>(pub &'l MethodShape);

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ShapeOrderWrapperOwned(pub MethodShape);

impl PartialOrd for ShapeOrderWrapperOwned{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ShapeOrderWrapperOwned{
    fn cmp(&self, other: &Self) -> Ordering {
        ShapeOrderWrapper(&self.0).cmp(&ShapeOrderWrapper(&other.0))
    }
}

impl PartialOrd for ShapeOrderWrapper<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ShapeOrderWrapper<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let MethodShape { name: this_name, desc: this_desc } = self.0;
        let MethodShape { name: other_name, desc: other_desc } = other.0;
        if this_name == other_name {
            if this_desc == other_desc {
                Ordering::Equal
            } else {
                if this_desc.arg_types.len() == other_desc.arg_types.len(){
                    this_desc.arg_types.iter().zip(other_desc.arg_types.iter()).map(|(this,other)|{
                        CPDTypeOrderWrapper(*this).partial_cmp(&CPDTypeOrderWrapper(*other))
                    }).flatten().find(|ordering|!matches!(ordering, Ordering::Equal)).unwrap_or(Ordering::Equal)
                }else {
                    this_desc.arg_types.len().cmp(&other_desc.arg_types.len())
                }
            }
        } else {
            this_name.0.cmp(&other_name.0)
        }
    }
}