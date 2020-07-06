use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;

use crate::view::ClassView;
use crate::view::ptype_view::ReferenceTypeView;

pub trait LivePoolGetter {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView;
}

#[derive(Debug)]
pub enum ClassLoadingError {
    ClassNotFoundException,
}

impl Display for ClassLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ClassNotFoundException")
    }
}

impl std::error::Error for ClassLoadingError {}


#[derive(Debug)]
#[derive(Hash, Eq)]
pub enum LoaderName {
    Str(String),
    Class(ClassName),
    BootstrapLoader,
}

impl PartialEq for LoaderName {
    fn eq(&self, other: &LoaderName) -> bool {
        match self {
            LoaderName::Str(s1) => match other {
                LoaderName::Str(s2) => s1 == s2,
                LoaderName::BootstrapLoader => false,
                LoaderName::Class(_) => false
            },
            LoaderName::BootstrapLoader => match other {
                LoaderName::Str(_) => false,
                LoaderName::BootstrapLoader => true,
                LoaderName::Class(_) => false
            },
            LoaderName::Class(c1) => match other {
                LoaderName::Str(_) => false,
                LoaderName::Class(c2) => c1 == c2,
                LoaderName::BootstrapLoader => false,
            }
        }
    }
}

impl Display for LoaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoaderName::Str(_) => unimplemented!(),
            LoaderName::BootstrapLoader => {
                write!(f, "<bl>")
            }
            LoaderName::Class(_) => unimplemented!()
        }
    }
}

pub trait Loader {
    fn find_loaded_class(&self, name: &ClassName) -> Option<Arc<ClassView>>;
    fn initiating_loader_of(&self, class: &ClassName) -> bool;
    //todo File will have to be a much more general array of bytes
    fn find_representation_of(&self, class: &ClassName) -> Result<File, ClassLoadingError>;
    fn load_class(&self, self_arc: LoaderArc, class: &ClassName, bl: LoaderArc, live_pool_getter: Arc<dyn LivePoolGetter>) -> Result<Arc<ClassView>, ClassLoadingError>;
    fn name(&self) -> LoaderName;


    //pre loading parses the class file but does not verify
    fn pre_load(&self, name: &ClassName) -> Result<Arc<ClassView>, ClassLoadingError>;
    fn add_pre_loaded(&self, name: &ClassName, classfile: &Arc<Classfile>);
}

//todo Loading Constraints

pub const BOOTSTRAP_LOADER_NAME: &str = "bl";

pub struct EmptyLoader {}

pub type LoaderArc = Arc<dyn Loader + Sync + Send>;

impl Loader for EmptyLoader {
    fn find_loaded_class(&self, _name: &ClassName) -> Option<Arc<ClassView>> {
        unimplemented!()
    }

    fn initiating_loader_of(&self, _class: &ClassName) -> bool {
        unimplemented!()
    }

    fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, _self_arc: LoaderArc, _class: &ClassName, _bl: LoaderArc, _live_pool_getter: Arc<dyn LivePoolGetter>) -> Result<Arc<ClassView>, ClassLoadingError> {
        unimplemented!()
    }

    fn name(&self) -> LoaderName {
        unimplemented!()
    }

    fn pre_load(&self, _name: &ClassName) -> Result<Arc<ClassView>, ClassLoadingError> {
        unimplemented!()
    }

    fn add_pre_loaded(&self, _name: &ClassName, _classfile: &Arc<Classfile>) {
        unimplemented!()
    }
}


pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: LoaderArc,
}

impl Hash for ClassWithLoader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
        self.loader.name().hash(state);
    }
}

impl PartialEq for ClassWithLoader {
    fn eq(&self, other: &ClassWithLoader) -> bool {
        self.class_name == other.class_name &&
            Arc::ptr_eq(&self.loader, &other.loader)
    }
}

impl Clone for ClassWithLoader {
    fn clone(&self) -> Self {
        ClassWithLoader { class_name: self.class_name.clone(), loader: self.loader.clone() }
    }
}

impl Eq for ClassWithLoader {}


impl Debug for ClassWithLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{},{}>", &self.class_name.get_referred_name(), self.loader.name())
    }
}
