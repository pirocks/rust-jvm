use crate::view::ClassView;
use crate::classfile::{AttributeType, BootstrapMethod};

pub struct BootstrapMethodIterator{
    pub(crate) view: BootstrapMethodsView,
    pub(crate) i :usize
}

impl Iterator for  BootstrapMethodIterator{
    type Item = BootstrapMethodView;

    fn next(&mut self) -> Option<Self::Item> {
        if i >= self.view.get_bootstrap_methods_raw().len(){
            return None
        }
        let res = BootstrapMethodView { backing: self.clone(), i: self.i };
        self.i += 1;
        res.into()
    }
}

#[derive(Clone)]
pub struct BootstrapMethodsView{
    pub(crate) backing_class : ClassView,
    pub(crate) attr_i : usize
}

impl BootstrapMethodsView{
    fn get_bootstrap_methods_raw(&self) -> &Vec<BootstrapMethod>{
        match &self.backing_class.backing_class.attributes[self.attr_i].attribute_type{
            AttributeType::BootstrapMethods(bm) => {
                &bm.bootstrap_methods
            },
            _ => panic!()
        }
    }

    pub fn bootstrap_methods(&self) -> BootstrapMethodIterator{
        unimplemented!()
    }
}

pub struct BootstrapMethodView{
    pub backing: BootstrapMethodsView,
    pub i: usize
}

impl BootstrapMethodView{
    fn get_raw(&self) -> &BootstrapMethod{
        &self.backing.get_bootstrap_methods_raw()[i]
    }

    pub fn bootstrap_args(&self) -> BootstrapArgViewIterator{
        self.get_raw().bootstrap_arguments
    }

}

pub struct BootstrapArgViewIterator{

}