use crate::view::{HasAccessFlags, ClassView};
use rust_jvm_common::classfile::{Code, MethodInfo};
use crate::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};

pub struct MethodView<'cl> {
    pub(crate) class_view: &'cl ClassView,
    pub(crate) method_i: usize,
}

impl Clone for MethodView<'_>{
    fn clone(&self) -> Self {
        Self {
            class_view: self.class_view,
            method_i: self.method_i
        }
    }
}

impl HasAccessFlags for MethodView<'_> {
    fn access_flags(&self) -> u16 {
        self.class_view.backing_class.methods[self.method_i].access_flags
    }
}

impl MethodView<'_> {
    fn from(c: &ClassView, i: usize) -> MethodView {
        MethodView { class_view: c, method_i: i }
    }

    pub fn classview(&self) -> &ClassView{
        self.class_view
    }

    //todo shouldn't be public but needs to be
    pub fn method_info(&self) -> &MethodInfo{
        &self.class_view.backing_class.methods[self.method_i]
    }

    pub fn name(&self) -> String {
        self.method_info().method_name(&self.class_view.backing_class)
    }

    pub fn desc_str(&self) -> String {
        self.method_info().descriptor_str(&self.class_view.backing_class)
    }

    pub fn desc(&self) -> MethodDescriptor {
        parse_method_descriptor( self.desc_str().as_str()).unwrap()
    }

    pub fn code_attribute(&self) -> Option<&Code>{
        self.method_info().code_attribute()//todo get a Code view
    }

    pub fn is_signature_polymorphic(&self) -> bool{
        // from the spec:
        // A method is signature polymorphic if all of the following are true:
        // •  It is declared in the java.lang.invoke.MethodHandle class.
        // •  It has a single formal parameter of type Object[].
        // •  It has a return type of Object.
        // •  It has the ACC_VARARGS and ACC_NATIVE flags set.
        self.class_view.name() == ClassName::method_handle() &&
            self.desc().parameter_types.len()  == 1 &&
            self.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
            self.desc().return_type == PTypeView::object().to_ptype() &&
            self.is_varargs() &&
            self.is_native()
    }

    //todo this shouldn't be public but needs to be atm.
    pub fn method_i(&self)-> usize {
        self.method_i
    }
}



pub struct MethodIterator<'l> {
    //todo create a from and remove pub(crate)
    pub(crate) class_view: &'l ClassView,
    pub(crate) i: usize,
}

impl <'cl> Iterator for MethodIterator<'cl> {
    type Item = MethodView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.class_view.num_methods() {
            return None;
        }
        let res = MethodView::from(self.class_view, self.i);
        self.i += 1;
        Some(res)
    }
}