use crate::view::ClassView;

pub struct InterfaceView<'l> {
    view: &'l ClassView,
    i: usize,
}


pub struct InterfaceIterator<'l> {
    //todo create a from and remove pub(crate)
    pub(crate) view: &'l ClassView,
    pub(crate) i: usize,
}


impl<'l> InterfaceView<'l> {
    fn from(c: &ClassView, i: usize) -> InterfaceView {
        InterfaceView { view: c, i }
    }
    pub fn interface_name(&self) -> String {
        self.view.backing_class.extract_class_from_constant_pool_name(self.view.backing_class.interfaces[self.i])
    }
}


impl <'l> Iterator for InterfaceIterator<'l> {
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
