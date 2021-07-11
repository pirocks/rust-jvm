use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType;

use crate::view::{ClassBackedView, ClassView};

pub enum InterfaceView<'l> {
    ClassBacked {
        view: &'l ClassBackedView,
        i: usize,
    },
    Cloneable,
    Serializable,
}


pub enum InterfaceIterator<'l> {
    ClassBacked {
        view: &'l ClassBackedView,
        i: usize,
    },
    Empty,
    CloneableAndSerializable {
        i: usize
    },
}


impl<'l> InterfaceView<'l> {
    fn from(c: &ClassBackedView, i: usize) -> InterfaceView {
        InterfaceView::ClassBacked { view: c, i }
    }
    pub fn interface_name(&self) -> ClassName {
        match self {
            InterfaceView::ClassBacked { view, i } => {
                PType::Ref(view.underlying_class.extract_class_from_constant_pool_name(view.underlying_class.interfaces[*i])).unwrap_class_type()
            }
            InterfaceView::Cloneable => {
                ClassName::cloneable()
            }
            InterfaceView::Serializable => {
                ClassName::serializable()
            }
        }
    }
}


impl<'l> Iterator for InterfaceIterator<'l> {
    type Item = InterfaceView<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            InterfaceIterator::ClassBacked { view, i } => {
                if *i >= view.num_interfaces() {
                    return None;
                }
                let res = InterfaceView::from(view, *i);
                *i += 1;
                Some(res)
            }
            InterfaceIterator::Empty => {
                None
            }
            InterfaceIterator::CloneableAndSerializable { i } => {
                let res = match *i {
                    0 => InterfaceView::Cloneable.into(),
                    1 => InterfaceView::Serializable.into(),
                    _ => None
                };
                *i += 1;
                res
            }
        }
    }
}
