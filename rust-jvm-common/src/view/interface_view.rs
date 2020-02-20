use crate::classfile::Classfile;
use std::sync::Arc;
use crate::view::ClassView;

pub struct InterfaceView {
    backing_class: Arc<Classfile>,
    i: usize,
}


pub struct InterfaceIterator<'l> {
    //todo create a from and remove pub(crate)
    pub(crate) backing_class: &'l ClassView,
    pub(crate) i: usize,
}


impl InterfaceView {
    fn from(c: &ClassView, i: usize) -> InterfaceView {
        InterfaceView { backing_class: c.backing_class.clone(), i }
    }
    pub fn interface_name(&self) -> String {
        self.backing_class.extract_class_from_constant_pool_name(self.backing_class.interfaces[self.i])
    }
}


impl Iterator for InterfaceIterator<'_> {
    type Item = InterfaceView;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.backing_class.num_interfaces() {
            return None;
        }
        let res = InterfaceView::from(self.backing_class, self.i);
        self.i += 1;
        Some(res)
    }
}
