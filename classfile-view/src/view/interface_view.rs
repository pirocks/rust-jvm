use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType;

use crate::view::{ClassBackedView, ClassView};

pub struct InterfaceView<'l> {
    view: &'l ClassBackedView,
    i: usize,
}


pub struct InterfaceIterator<'l> {
    pub(crate) view: &'l ClassBackedView,
    pub(crate) i: usize,
}


impl<'l> InterfaceView<'l> {
    fn from(c: &ClassBackedView, i: usize) -> InterfaceView {
        InterfaceView { view: c, i }
    }
    pub fn interface_name(&self) -> ClassName {
        PType::Ref(self.view.backing_class.extract_class_from_constant_pool_name(self.view.backing_class.interfaces[self.i])).unwrap_class_type()
    }
}


impl<'l> Iterator for InterfaceIterator<'l> {
    type Item = InterfaceView<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.view.num_interfaces() {
            return None;
        }
        let res = InterfaceView::from(self.view, self.i);
        self.i += 1;
        Some(res)
    }
}
